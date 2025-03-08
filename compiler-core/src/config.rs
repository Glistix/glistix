use crate::error::{FileIoAction, FileKind};
use crate::io::FileSystemReader;
use crate::manifest::Manifest;
use crate::requirement::Requirement;
use crate::version::COMPILER_VERSION;
use crate::{Error, Result};
use camino::{Utf8Path, Utf8PathBuf};
use ecow::EcoString;
use globset::{Glob, GlobSetBuilder};
use hexpm::version::{self, Version};
use http::Uri;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fmt::{self};
use std::marker::PhantomData;

#[cfg(test)]
use crate::manifest::ManifestPackage;

use crate::build::{Mode, Runtime, Target};

fn default_version() -> Version {
    Version::parse("0.1.0").expect("default version")
}

fn erlang_target() -> Target {
    Target::Erlang
}

fn default_javascript_runtime() -> Runtime {
    Runtime::NodeJs
}

pub type Dependencies = HashMap<EcoString, Requirement>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpdxLicense {
    pub licence: String,
}

impl ToString for SpdxLicense {
    fn to_string(&self) -> String {
        String::from(&self.licence)
    }
}

impl<'de> Deserialize<'de> for SpdxLicense {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        match spdx::license_id(s) {
            None => Err(serde::de::Error::custom(format!(
                "{s} is not a valid SPDX License ID"
            ))),
            Some(_) => Ok(SpdxLicense {
                licence: String::from(s),
            }),
        }
    }
}

impl AsRef<str> for SpdxLicense {
    fn as_ref(&self) -> &str {
        self.licence.as_str()
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct PackageConfig {
    #[serde(with = "package_name")]
    pub name: EcoString,
    #[serde(default = "default_version")]
    pub version: Version,
    #[serde(
        default,
        rename = "gleam",
        serialize_with = "serialise_range",
        deserialize_with = "deserialise_range"
    )]
    pub gleam_version: Option<pubgrub::range::Range<Version>>,
    #[serde(default, alias = "licenses")]
    pub licences: Vec<SpdxLicense>,
    #[serde(default)]
    pub description: EcoString,
    #[serde(default, alias = "docs")]
    pub documentation: Docs,
    #[serde(default)]
    pub dependencies: Dependencies,
    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: Dependencies,
    #[serde(default)]
    pub repository: Repository,
    #[serde(default)]
    pub links: Vec<Link>,
    #[serde(default)]
    pub erlang: ErlangConfig,
    #[serde(default)]
    pub javascript: JavaScriptConfig,
    #[serde(default = "erlang_target")]
    pub target: Target,
    #[serde(default)]
    pub internal_modules: Option<Vec<Glob>>,
    #[serde(default)]
    pub glistix: GlistixConfig,
}

pub fn serialise_range<S>(
    range: Option<pubgrub::range::Range<Version>>,
    serialiser: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match range {
        Some(range) => serialiser.serialize_some(&range.to_string()),
        None => serialiser.serialize_none(),
    }
}

pub fn deserialise_range<'de, D>(
    deserialiser: D,
) -> Result<Option<pubgrub::range::Range<Version>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Deserialize::deserialize(deserialiser)? {
        Some(range_string) => Ok(Some(
            version::Range::new(range_string)
                .to_pubgrub()
                .map_err(serde::de::Error::custom)?,
        )),
        None => Ok(None),
    }
}

impl PackageConfig {
    pub fn dependencies_for(&self, mode: Mode) -> Result<Dependencies> {
        match mode {
            Mode::Dev | Mode::Lsp => self.all_direct_dependencies(),
            Mode::Prod => Ok(self.dependencies.clone()),
        }
    }

    // Return all the dependencies listed in the configuration, that is, all the
    // direct dependencies, both in the `dependencies` and `dev-dependencies`.
    pub fn all_direct_dependencies(&self) -> Result<Dependencies> {
        let mut deps =
            HashMap::with_capacity(self.dependencies.len() + self.dev_dependencies.len());
        for (name, requirement) in self.dependencies.iter().chain(&self.dev_dependencies) {
            let already_inserted = deps.insert(name.clone(), requirement.clone()).is_some();
            if already_inserted {
                return Err(Error::DuplicateDependency(name.clone()));
            }
        }
        Ok(deps)
    }

    pub fn read<FS: FileSystemReader, P: AsRef<Utf8Path>>(
        path: P,
        fs: &FS,
    ) -> Result<PackageConfig, Error> {
        let toml = fs.read(path.as_ref())?;
        let config: PackageConfig = toml::from_str(&toml).map_err(|e| Error::FileIo {
            action: FileIoAction::Parse,
            kind: FileKind::File,
            path: path.as_ref().to_path_buf(),
            err: Some(e.to_string()),
        })?;
        Ok(config)
    }

    /// Get the locked packages for the current config and a given (optional)
    /// manifest of previously locked packages.
    ///
    /// If a package is removed or the specified required version range for it
    /// changes then it is not considered locked. This also goes for any child
    /// packages of the package which have no other parents.
    ///
    /// This function should be used each time resolution is performed so that
    /// outdated deps are removed from the manifest and not locked to the
    /// previously selected versions.
    ///
    pub fn locked(&self, manifest: Option<&Manifest>) -> Result<HashMap<EcoString, Version>> {
        match manifest {
            None => Ok(HashMap::new()),
            Some(manifest) => StalePackageRemover::fresh_and_locked(
                &self.all_direct_dependencies()?,
                manifest,
                &self.glistix.preview.patch,
            ),
        }
    }

    /// Determines whether the given module should be hidden in the docs or not
    ///
    /// The developer can specify a list of glob patterns in the gleam.toml file
    /// to determine modules that should not be shown in the package's documentation
    pub fn is_internal_module(&self, module: &str) -> bool {
        let package = &self.name;
        match &self.internal_modules {
            Some(globs) => {
                let mut builder = GlobSetBuilder::new();
                for glob in globs {
                    _ = builder.add(glob.clone());
                }
                builder.build()
            }

            // If no patterns were specified in the config then we use a default value
            None => GlobSetBuilder::new()
                .add(Glob::new(&format!("{package}/internal")).expect("internal module glob"))
                .add(Glob::new(&format!("{package}/internal/*")).expect("internal module glob"))
                .build(),
        }
        .expect("internal module globs")
        .is_match(module)
    }

    // Checks to see if the gleam version specified in the config is compatible
    // with the current compiler version
    pub fn check_gleam_compatibility(&self) -> Result<(), Error> {
        if let Some(range) = &self.gleam_version {
            let compiler_version =
                Version::parse(COMPILER_VERSION).expect("Parse compiler semantic version");

            // We ignore the pre-release and build metadata when checking compatibility
            let mut version_without_pre = compiler_version.clone();
            version_without_pre.pre = vec![];
            version_without_pre.build = None;
            if !range.contains(&version_without_pre) {
                return Err(Error::IncompatibleCompilerVersion {
                    package: self.name.to_string(),
                    required_version: range.to_string(),
                    gleam_version: COMPILER_VERSION.to_string(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct StalePackageRemover<'a> {
    // These are the packages for which the requirement or their parents
    // requirement has not changed.
    fresh: HashSet<&'a str>,
    locked: HashMap<EcoString, &'a Vec<EcoString>>,
}

impl<'a> StalePackageRemover<'a> {
    pub fn fresh_and_locked(
        requirements: &'a HashMap<EcoString, Requirement>,
        manifest: &'a Manifest,
        glistix_patches: &'a GlistixPatches,
    ) -> Result<HashMap<EcoString, Version>> {
        let locked = manifest
            .packages
            .iter()
            .map(|p| (p.name.clone(), &p.requirements))
            .collect();

        Self {
            fresh: HashSet::new(),
            locked,
        }
        .run(requirements, manifest, glistix_patches)
    }

    fn run(
        &mut self,
        requirements: &'a HashMap<EcoString, Requirement>,
        manifest: &'a Manifest,
        glistix_patches: &'a GlistixPatches,
    ) -> Result<HashMap<EcoString, Version>> {
        // TODO: Don't unlock dependents of newly-patched packages when only the
        // version changed, but not any names
        // Track packages from removed patches so they are re-fetched or removed
        let glistix_packages_from_removed_patches = manifest
            .glistix
            .preview
            .patch
            .0
            .iter()
            .filter(|(old_name, _)| !glistix_patches.0.contains_key(*old_name))
            .flat_map(|(old_name, patch)| {
                // Include both 'old_name' and 'new_name' as packages
                // potentially affected by the patch changes.
                Some(&**old_name).into_iter().chain(patch.name.as_deref())
            });

        // Track packages from new or modified patches so they are re-fetched
        // or removed
        let glistix_newly_patched_packages = glistix_patches
            .0
            .iter()
            .filter(|(old_name, patch)| {
                manifest.glistix.preview.patch.0.get(*old_name) != Some(patch)
            })
            .flat_map(|(old_name, patch)| {
                // Include both 'old_name' and 'new_name' as packages
                // potentially affected by the patch changes.
                Some(&**old_name)
                    .into_iter()
                    .chain(patch.name.as_deref())
                    .chain(
                        // Also unlock the previous renamed-to name as it is no
                        // longer receiving a patch, so requirements will have
                        // to be updated.
                        manifest
                            .glistix
                            .preview
                            .patch
                            .0
                            .get(&**old_name)
                            .filter(|p| p.name != patch.name)
                            .and_then(|p| p.name.as_deref()),
                    )
            });

        // Join packages from removed patches with added and motified patches
        let glistix_newly_patched_packages: Vec<&'a str> = glistix_packages_from_removed_patches
            .chain(glistix_newly_patched_packages)
            .collect::<Vec<_>>();

        // Record all the requirements that have not changed
        for (name, requirement) in requirements {
            if manifest.requirements.get(name) != Some(requirement)
                || glistix_newly_patched_packages.contains(&&**name)
            {
                continue; // This package has changed, don't record it
            }

            // Recursively record the package and its deps as being fresh
            self.record_tree_fresh(name, &glistix_newly_patched_packages)?;
        }

        // Return all the previously resolved packages that have not been
        // recorded as fresh
        Ok(manifest
            .packages
            .iter()
            .filter(|package| {
                // If any requirement was patched, it might have been renamed to something else,
                // so we force the package to be re-fetched.
                let glistix_depends_on_newly_patched_packages = !glistix_newly_patched_packages
                    .is_empty()
                    && package
                        .requirements
                        .iter()
                        .any(|r| glistix_newly_patched_packages.contains(&&**r));

                let new = requirements.contains_key(package.name.as_str())
                    && !manifest.requirements.contains_key(package.name.as_str());
                let fresh = self.fresh.contains(package.name.as_str());
                let locked = !glistix_depends_on_newly_patched_packages && !new && fresh;
                if !locked {
                    tracing::info!(name = package.name.as_str(), "unlocking_stale_package");
                }
                locked
            })
            .map(|package| (package.name.clone(), package.version.clone()))
            .collect())
    }

    fn record_tree_fresh(
        &mut self,
        name: &'a str,
        glistix_newly_patched_packages: &[&'a str],
    ) -> Result<()> {
        // Record the top level package
        let _ = self.fresh.insert(name);

        let deps = self.locked.get(name).ok_or(Error::CorruptManifest)?;
        // Record each of its deps recursively
        for package in *deps {
            if glistix_newly_patched_packages.contains(&&**package) {
                // Dep was affected by a patch
                continue;
            }

            self.record_tree_fresh(package, glistix_newly_patched_packages)?;
        }
        Ok(())
    }
}

#[test]
fn locked_no_manifest() {
    let mut config = PackageConfig::default();
    config.dependencies = [
        ("prod1".into(), Requirement::hex("~> 1.0")),
        ("prod2".into(), Requirement::hex("~> 2.0")),
    ]
    .into();
    config.dev_dependencies = [
        ("dev1".into(), Requirement::hex("~> 1.0")),
        ("dev2".into(), Requirement::hex("~> 2.0")),
    ]
    .into();
    assert_eq!(config.locked(None).unwrap(), [].into());
}

#[test]
fn locked_no_changes() {
    let mut config = PackageConfig::default();
    config.dependencies = [
        ("prod1".into(), Requirement::hex("~> 1.0")),
        ("prod2".into(), Requirement::hex("~> 2.0")),
    ]
    .into();
    config.dev_dependencies = [
        ("dev1".into(), Requirement::hex("~> 1.0")),
        ("dev2".into(), Requirement::hex("~> 2.0")),
    ]
    .into();
    let manifest = Manifest {
        requirements: config.all_direct_dependencies().unwrap(),
        packages: vec![
            manifest_package("prod1", "1.1.0", &[]),
            manifest_package("prod2", "1.2.0", &[]),
            manifest_package("dev1", "1.1.0", &[]),
            manifest_package("dev2", "1.2.0", &[]),
        ],
        glistix: Default::default(),
    };
    assert_eq!(
        config.locked(Some(&manifest)).unwrap(),
        [
            locked_version("prod1", "1.1.0"),
            locked_version("prod2", "1.2.0"),
            locked_version("dev1", "1.1.0"),
            locked_version("dev2", "1.2.0"),
        ]
        .into()
    );
}

#[test]
fn locked_some_removed() {
    let mut config = PackageConfig::default();
    config.dependencies = [("prod1".into(), Requirement::hex("~> 1.0"))].into();
    config.dev_dependencies = [("dev2".into(), Requirement::hex("~> 2.0"))].into();
    let manifest = Manifest {
        requirements: config.all_direct_dependencies().unwrap(),
        packages: vec![
            manifest_package("prod1", "1.1.0", &[]),
            manifest_package("prod2", "1.2.0", &[]), // Not in config
            manifest_package("dev1", "1.1.0", &[]),  // Not in config
            manifest_package("dev2", "1.2.0", &[]),
        ],
        glistix: Default::default(),
    };
    assert_eq!(
        config.locked(Some(&manifest)).unwrap(),
        [
            // prod2 removed
            // dev1 removed
            locked_version("prod1", "1.1.0"),
            locked_version("dev2", "1.2.0"),
        ]
        .into()
    );
}

#[test]
fn locked_some_changed() {
    let mut config = PackageConfig::default();
    config.dependencies = [
        ("prod1".into(), Requirement::hex("~> 3.0")), // Does not match manifest
        ("prod2".into(), Requirement::hex("~> 2.0")),
    ]
    .into();
    config.dev_dependencies = [
        ("dev1".into(), Requirement::hex("~> 3.0")), // Does not match manifest
        ("dev2".into(), Requirement::hex("~> 2.0")),
    ]
    .into();
    let manifest = Manifest {
        requirements: [
            ("prod1".into(), Requirement::hex("~> 1.0")),
            ("prod2".into(), Requirement::hex("~> 2.0")),
            ("dev1".into(), Requirement::hex("~> 1.0")),
            ("dev2".into(), Requirement::hex("~> 2.0")),
        ]
        .into(),
        packages: vec![
            manifest_package("prod1", "1.1.0", &[]),
            manifest_package("prod2", "1.2.0", &[]),
            manifest_package("dev1", "1.1.0", &[]),
            manifest_package("dev2", "1.2.0", &[]),
        ],
        glistix: Default::default(),
    };
    assert_eq!(
        config.locked(Some(&manifest)).unwrap(),
        [
            // prod1 removed
            // dev1 removed
            locked_version("prod2", "1.2.0"),
            locked_version("dev2", "1.2.0"),
        ]
        .into()
    );
}

#[test]
fn locked_nested_are_removed_too() {
    let mut config = PackageConfig::default();
    config.dependencies = [
        ("1".into(), Requirement::hex("~> 2.0")), // Does not match manifest
        ("2".into(), Requirement::hex("~> 1.0")),
    ]
    .into();
    config.dev_dependencies = [].into();
    let manifest = Manifest {
        requirements: [
            ("1".into(), Requirement::hex("~> 1.0")),
            ("2".into(), Requirement::hex("~> 1.0")),
        ]
        .into(),
        packages: vec![
            manifest_package("1", "1.1.0", &["1.1", "1.2"]),
            manifest_package("1.1", "1.1.0", &["1.1.1", "1.1.2"]),
            manifest_package("1.1.1", "1.1.0", &["shared"]),
            manifest_package("1.1.2", "1.1.0", &[]),
            manifest_package("1.2", "1.1.0", &["1.2.1", "1.2.2"]),
            manifest_package("1.2.1", "1.1.0", &[]),
            manifest_package("1.2.2", "1.1.0", &[]),
            manifest_package("2", "2.1.0", &["2.1", "2.2"]),
            manifest_package("2.1", "2.1.0", &["2.1.1", "2.1.2"]),
            manifest_package("2.1.1", "2.1.0", &[]),
            manifest_package("2.1.2", "2.1.0", &[]),
            manifest_package("2.2", "2.1.0", &["2.2.1", "2.2.2", "shared"]),
            manifest_package("2.2.1", "2.1.0", &[]),
            manifest_package("2.2.2", "2.1.0", &[]),
            manifest_package("shared", "2.1.0", &[]),
        ],
        glistix: Default::default(),
    };
    assert_eq!(
        config.locked(Some(&manifest)).unwrap(),
        [
            // 1* removed
            locked_version("2", "2.1.0"),
            locked_version("2.1", "2.1.0"),
            locked_version("2.1.1", "2.1.0"),
            locked_version("2.1.2", "2.1.0"),
            locked_version("2.2", "2.1.0"),
            locked_version("2.2.1", "2.1.0"),
            locked_version("2.2.2", "2.1.0"),
            locked_version("shared", "2.1.0"),
        ]
        .into()
    );
}

// https://github.com/gleam-lang/gleam/issues/1754
#[test]
fn locked_unlock_new() {
    let mut config = PackageConfig::default();
    config.dependencies = [
        ("1".into(), Requirement::hex("~> 1.0")),
        ("2".into(), Requirement::hex("~> 1.0")),
        ("3".into(), Requirement::hex("~> 3.0")), // Does not match manifest
    ]
    .into();
    config.dev_dependencies = [].into();
    let manifest = Manifest {
        requirements: [
            ("1".into(), Requirement::hex("~> 1.0")),
            ("2".into(), Requirement::hex("~> 1.0")),
        ]
        .into(),
        packages: vec![
            manifest_package("1", "1.1.0", &["3"]),
            manifest_package("2", "1.1.0", &["3"]),
            manifest_package("3", "1.1.0", &[]),
        ],
        glistix: Default::default(),
    };
    assert_eq!(
        config.locked(Some(&manifest)).unwrap(),
        [locked_version("1", "1.1.0"), locked_version("2", "1.1.0"),].into()
    )
}

#[cfg(test)]
fn generate_glistix_patches(
    patches: impl IntoIterator<Item = (&'static str, Option<&'static str>, Requirement)>,
) -> GlistixPatches {
    let patches = patches
        .into_iter()
        .map(|(old_name, new_name, source)| {
            (
                EcoString::from(old_name),
                GlistixPatch {
                    name: new_name.map(EcoString::from),
                    source,
                },
            )
        })
        .collect::<HashMap<_, _>>();

    GlistixPatches(patches)
}

#[test]
fn glistix_locked_top_level_are_removed_with_new_patches() {
    let mut config = PackageConfig::default();
    config.dependencies = [
        ("a".into(), Requirement::hex("== 0.1.0")),
        ("d2".into(), Requirement::hex("== 0.1.0")),
        ("something_else".into(), Requirement::hex("== 0.1.0")),
        ("f".into(), Requirement::hex("== 0.1.0")),
    ]
    .into();
    config.dev_dependencies = [].into();
    config.glistix.preview.patch = generate_glistix_patches([
        ("unexistent", Some("a"), Requirement::hex("== 0.1.0")),
        ("d1", Some("d2"), Requirement::hex("== 0.1.0")),
        ("e", Some("something_else"), Requirement::hex("== 0.1.0")),
        ("f", None, Requirement::hex("== 0.1.0")),
    ]);
    let manifest = Manifest {
        requirements: config.dependencies.clone(),
        packages: vec![
            manifest_package("a", "0.1.0", &["b", "c"]),
            manifest_package("b", "0.1.0", &[]),
            manifest_package("c", "0.1.0", &[]),
            manifest_package("d1", "0.1.0", &[]),
            manifest_package("d2", "0.1.0", &[]),
            manifest_package("e", "0.1.0", &[]),
            manifest_package("f", "0.1.0", &[]),
        ],
        glistix: crate::manifest::GlistixManifest::with_patches(generate_glistix_patches([(
            "unexistent",
            Some("a"),
            Requirement::hex("== 0.1.0"),
        )])),
    };
    assert_eq!(
        config.locked(Some(&manifest)).unwrap(),
        [
            // a was patched but the patch didn't change, so it was kept
            // d1, d2, e, f are the new targets of patches, removed
            locked_version("a", "0.1.0"),
            locked_version("b", "0.1.0"),
            locked_version("c", "0.1.0"),
        ]
        .into()
    );
}

#[test]
fn glistix_locked_top_level_are_removed_with_patches_removed() {
    let mut config = PackageConfig::default();
    config.dependencies = [
        ("a".into(), Requirement::hex("== 0.1.0")),
        ("d2".into(), Requirement::hex("== 0.1.0")),
        ("something_else".into(), Requirement::hex("== 0.1.0")),
        ("f".into(), Requirement::hex("== 0.1.0")),
    ]
    .into();
    config.dev_dependencies = [].into();
    config.glistix.preview.patch =
        generate_glistix_patches([("unexistent", Some("a"), Requirement::hex("== 0.1.0"))]);
    let manifest = Manifest {
        requirements: config.dependencies.clone(),
        packages: vec![
            manifest_package("a", "0.1.0", &["b", "c"]),
            manifest_package("b", "0.1.0", &[]),
            manifest_package("c", "0.1.0", &[]),
            manifest_package("d1", "0.1.0", &[]),
            manifest_package("d2", "0.1.0", &[]),
            manifest_package("e", "0.1.0", &[]),
            manifest_package("f", "0.1.0", &[]),
        ],
        glistix: crate::manifest::GlistixManifest::with_patches(generate_glistix_patches([
            ("unexistent", Some("a"), Requirement::hex("== 0.1.0")),
            ("d1", Some("d2"), Requirement::hex("== 0.1.0")),
            ("e", Some("something_else"), Requirement::hex("== 0.1.0")),
            ("f", None, Requirement::hex("== 0.1.0")),
        ])),
    };
    assert_eq!(
        config.locked(Some(&manifest)).unwrap(),
        [
            // a was patched but the patch didn't change, so it was kept
            // d1 was the original target of a patch, removed
            // d2 was the rename target of a patch, removed
            // e was the original target of a patch, removed
            // f was the original target of a patch, removed
            locked_version("a", "0.1.0"),
            locked_version("b", "0.1.0"),
            locked_version("c", "0.1.0"),
        ]
        .into()
    );
}

#[test]
fn glistix_locked_top_level_patches_added() {
    let mut config = PackageConfig::default();
    config.dependencies = [
        ("a".into(), Requirement::hex("== 0.1.0")),
        ("b".into(), Requirement::hex("== 0.1.0")),
        ("c".into(), Requirement::hex("== 0.1.0")),
        ("d".into(), Requirement::hex("== 0.1.0")),
    ]
    .into();
    config.dev_dependencies = [].into();
    config.glistix.preview.patch =
        generate_glistix_patches([("c", Some("d"), Requirement::hex("== 0.1.0"))]);
    let manifest = Manifest {
        requirements: config.dependencies.clone(),
        packages: vec![
            manifest_package("a", "0.1.0", &[]),
            manifest_package("b", "0.1.0", &[]),
            manifest_package("c", "0.1.0", &[]),
            manifest_package("d", "0.1.0", &[]),
        ],
        glistix: crate::manifest::GlistixManifest::with_patches(generate_glistix_patches([])),
    };
    assert_eq!(
        config.locked(Some(&manifest)).unwrap(),
        [
            // c and d participating in patch, removed
            locked_version("a", "0.1.0"),
            locked_version("b", "0.1.0"),
        ]
        .into()
    );
}

#[test]
fn glistix_locked_top_level_patch_removed() {
    let mut config = PackageConfig::default();
    config.dependencies = [
        ("a".into(), Requirement::hex("== 0.1.0")),
        ("b".into(), Requirement::hex("== 0.1.0")),
        ("c".into(), Requirement::hex("== 0.1.0")),
        ("d".into(), Requirement::hex("== 0.1.0")),
    ]
    .into();
    config.dev_dependencies = [].into();
    config.glistix.preview.patch = generate_glistix_patches([]);
    let manifest = Manifest {
        requirements: config.dependencies.clone(),
        packages: vec![
            manifest_package("a", "0.1.0", &[]),
            manifest_package("b", "0.1.0", &[]),
            manifest_package("c", "0.1.0", &[]),
            manifest_package("d", "0.1.0", &[]),
        ],
        glistix: crate::manifest::GlistixManifest::with_patches(generate_glistix_patches([(
            "c",
            Some("d"),
            Requirement::hex("== 0.1.0"),
        )])),
    };
    assert_eq!(
        config.locked(Some(&manifest)).unwrap(),
        [
            // c and d participating in patch, removed
            locked_version("a", "0.1.0"),
            locked_version("b", "0.1.0"),
        ]
        .into()
    );
}

#[test]
fn glistix_locked_top_level_rename_patch_modified() {
    let mut config = PackageConfig::default();
    config.dependencies = [
        ("a".into(), Requirement::hex("== 0.1.0")),
        ("b".into(), Requirement::hex("== 0.1.0")),
        ("c".into(), Requirement::hex("== 0.1.0")),
        ("d".into(), Requirement::hex("== 0.1.0")),
    ]
    .into();
    config.dev_dependencies = [].into();
    config.glistix.preview.patch =
        generate_glistix_patches([("c", Some("d"), Requirement::hex("== 0.1.0"))]);
    let manifest = Manifest {
        requirements: config.dependencies.clone(),
        packages: vec![
            manifest_package("a", "0.1.0", &[]),
            manifest_package("b", "0.1.0", &[]),
            manifest_package("c", "0.1.0", &[]),
            manifest_package("d", "0.1.0", &[]),
        ],
        glistix: crate::manifest::GlistixManifest::with_patches(generate_glistix_patches([(
            "c",
            Some("b"),
            Requirement::hex("== 0.1.0"),
        )])),
    };
    assert_eq!(
        config.locked(Some(&manifest)).unwrap(),
        [
            // b, c and d participating in old and new patches, removed
            locked_version("a", "0.1.0"),
        ]
        .into()
    );
}

#[test]
fn glistix_locked_top_level_non_rename_patch_modified() {
    let mut config = PackageConfig::default();
    config.dependencies = [
        ("a".into(), Requirement::hex("== 0.1.0")),
        ("b".into(), Requirement::hex("== 0.1.0")),
        ("c".into(), Requirement::hex("== 0.1.0")),
        ("d".into(), Requirement::hex("== 0.1.0")),
    ]
    .into();
    config.dev_dependencies = [].into();
    config.glistix.preview.patch =
        generate_glistix_patches([("c", None, Requirement::hex("== 0.2.0"))]);
    let manifest = Manifest {
        requirements: config.dependencies.clone(),
        packages: vec![
            manifest_package("a", "0.1.0", &[]),
            manifest_package("b", "0.1.0", &[]),
            manifest_package("c", "0.1.0", &[]),
            manifest_package("d", "0.1.0", &[]),
        ],
        glistix: crate::manifest::GlistixManifest::with_patches(generate_glistix_patches([(
            "c",
            None,
            Requirement::hex("== 0.1.0"),
        )])),
    };
    assert_eq!(
        config.locked(Some(&manifest)).unwrap(),
        [
            // c patch changed, unlocked
            locked_version("a", "0.1.0"),
            locked_version("b", "0.1.0"),
            locked_version("d", "0.1.0"),
        ]
        .into()
    );
}

#[test]
fn glistix_locked_nested_patches_added_unlocks_dependent() {
    let mut config = PackageConfig::default();
    config.dependencies = [
        ("a".into(), Requirement::hex("== 0.1.0")),
        ("d".into(), Requirement::hex("== 0.1.0")),
        ("h".into(), Requirement::hex("== 0.1.0")),
    ]
    .into();
    config.dev_dependencies = [].into();
    config.glistix.preview.patch = generate_glistix_patches([
        ("m", Some("d"), Requirement::hex("== 0.1.0")),
        ("b", Some("h"), Requirement::hex("== 0.1.0")),
        ("f", None, Requirement::hex("== 0.1.0")),
    ]);
    let manifest = Manifest {
        requirements: config.dependencies.clone(),
        packages: vec![
            manifest_package("a", "0.1.0", &["b", "c"]),
            manifest_package("b", "0.1.0", &[]),
            manifest_package("c", "0.1.0", &[]),
            manifest_package("d", "0.1.0", &["e"]),
            manifest_package("e", "0.1.0", &["f"]),
            manifest_package("f", "0.1.0", &[]),
            manifest_package("h", "0.1.0", &["g"]),
            manifest_package("g", "0.1.0", &[]),
        ],
        glistix: crate::manifest::GlistixManifest::with_patches(generate_glistix_patches([(
            "m",
            Some("d"),
            Requirement::hex("== 0.1.0"),
        )])),
    };
    assert_eq!(
        config.locked(Some(&manifest)).unwrap(),
        [
            // "a" depends on "b" which is no longer patched, so it is unlocked
            // with "b"
            // "h" was what "b" used to be patched to so it is also unlocked
            // "e" depends on "f" which used to be patched so both are unlocked,
            // however "d" can still depend on "e" since "e" itself wasn't
            // affected
            // "d" is also kept as its existing patch wasn't changed
            // "g" is unlocked as it was simply a dependency of "h"
            locked_version("c", "0.1.0"),
            locked_version("d", "0.1.0"),
        ]
        .into()
    );
}

#[test]
fn glistix_locked_nested_patches_removed_unlocks_dependent() {
    let mut config = PackageConfig::default();
    config.dependencies = [
        ("a".into(), Requirement::hex("== 0.1.0")),
        ("d".into(), Requirement::hex("== 0.1.0")),
        ("g".into(), Requirement::hex("== 0.1.0")),
    ]
    .into();
    config.dev_dependencies = [].into();
    config.glistix.preview.patch = generate_glistix_patches([
        // This one isn't modified
        ("m", Some("d"), Requirement::hex("== 0.1.0")),
    ]);
    let manifest = Manifest {
        requirements: config.dependencies.clone(),
        packages: vec![
            manifest_package("a", "0.1.0", &["b", "c"]),
            manifest_package("b", "0.1.0", &[]),
            manifest_package("c", "0.1.0", &[]),
            manifest_package("d", "0.1.0", &["e"]),
            manifest_package("e", "0.1.0", &["f"]),
            manifest_package("f", "0.1.0", &[]),
            manifest_package("g", "0.1.0", &["h"]),
            manifest_package("h", "0.1.0", &["i"]),
            manifest_package("i", "0.1.0", &[]),
        ],
        glistix: crate::manifest::GlistixManifest::with_patches(generate_glistix_patches([
            ("m", Some("d"), Requirement::hex("== 0.1.0")),
            ("b", Some("h"), Requirement::hex("== 0.1.0")),
            ("f", None, Requirement::hex("== 0.1.0")),
        ])),
    };
    assert_eq!(
        config.locked(Some(&manifest)).unwrap(),
        [
            // Should be the opposite as the previous situation:
            // Removed patches to b affecting h, and to f
            // So all three should be unlocked
            // However, patch to "d" is kept
            locked_version("c", "0.1.0"),
            locked_version("d", "0.1.0"),
        ]
        .into()
    );
}

#[test]
fn default_internal_modules() {
    // When no internal modules are specified then we default to
    // `["$package/internal", "$package/internal/*"]`
    let mut config = PackageConfig::default();
    config.name = "my_package".into();
    config.internal_modules = None;

    assert!(config.is_internal_module("my_package/internal"));
    assert!(config.is_internal_module("my_package/internal/wibble"));
    assert!(config.is_internal_module("my_package/internal/wibble/wobble"));
    assert!(!config.is_internal_module("my_package/internallll"));
    assert!(!config.is_internal_module("my_package/other"));
    assert!(!config.is_internal_module("my_package/other/wibble"));
    assert!(!config.is_internal_module("other/internal"));
}

#[test]
fn no_internal_modules() {
    // When no internal modules are specified then we default to
    // `["$package/internal", "$package/internal/*"]`
    let mut config = PackageConfig::default();
    config.name = "my_package".into();
    config.internal_modules = Some(vec![]);

    assert!(!config.is_internal_module("my_package/internal"));
    assert!(!config.is_internal_module("my_package/internal/wibble"));
    assert!(!config.is_internal_module("my_package/internal/wibble/wobble"));
    assert!(!config.is_internal_module("my_package/internallll"));
    assert!(!config.is_internal_module("my_package/other"));
    assert!(!config.is_internal_module("my_package/other/wibble"));
    assert!(!config.is_internal_module("other/internal"));
}

#[test]
fn hidden_a_directory_from_docs() {
    let mut config = PackageConfig::default();
    config.internal_modules = Some(vec![Glob::new("package/internal/*").expect("")]);

    let mod1 = "package/internal";
    let mod2 = "package/internal/module";

    assert_eq!(config.is_internal_module(mod1), false);
    assert_eq!(config.is_internal_module(mod2), true);
}

#[test]
fn hidden_two_directories_from_docs() {
    let mut config = PackageConfig::default();
    config.internal_modules = Some(vec![
        Glob::new("package/internal1/*").expect(""),
        Glob::new("package/internal2/*").expect(""),
    ]);

    let mod1 = "package/internal1";
    let mod2 = "package/internal1/module";
    let mod3 = "package/internal2";
    let mod4 = "package/internal2/module";

    assert_eq!(config.is_internal_module(mod1), false);
    assert_eq!(config.is_internal_module(mod2), true);
    assert_eq!(config.is_internal_module(mod3), false);
    assert_eq!(config.is_internal_module(mod4), true);
}

#[test]
fn hidden_a_directory_and_a_file_from_docs() {
    let mut config = PackageConfig::default();
    config.internal_modules = Some(vec![
        Glob::new("package/internal1/*").expect(""),
        Glob::new("package/module").expect(""),
    ]);

    let mod1 = "package/internal1";
    let mod2 = "package/internal1/module";
    let mod3 = "package/module";
    let mod4 = "package/module/inner";

    assert_eq!(config.is_internal_module(mod1), false);
    assert_eq!(config.is_internal_module(mod2), true);
    assert_eq!(config.is_internal_module(mod3), true);
    assert_eq!(config.is_internal_module(mod4), false);
}

#[test]
fn hidden_a_file_in_all_directories_from_docs() {
    let mut config = PackageConfig::default();
    config.internal_modules = Some(vec![Glob::new("package/*/module1").expect("")]);

    let mod1 = "package/internal1/module1";
    let mod2 = "package/internal2/module1";
    let mod3 = "package/internal2/module2";
    let mod4 = "package/module";

    assert_eq!(config.is_internal_module(mod1), true);
    assert_eq!(config.is_internal_module(mod2), true);
    assert_eq!(config.is_internal_module(mod3), false);
    assert_eq!(config.is_internal_module(mod4), false);
}

#[cfg(test)]
fn manifest_package(
    name: &'static str,
    version: &'static str,
    requirements: &'static [&'static str],
) -> ManifestPackage {
    use crate::manifest::Base16Checksum;

    ManifestPackage {
        name: name.into(),
        version: Version::parse(version).unwrap(),
        build_tools: vec![],
        otp_app: None,
        requirements: requirements.iter().map(|e| (*e).into()).collect(),
        source: crate::manifest::ManifestPackageSource::Hex {
            outer_checksum: Base16Checksum(vec![]),
        },
    }
}

#[cfg(test)]
fn locked_version(name: &'static str, version: &'static str) -> (EcoString, Version) {
    (name.into(), Version::parse(version).unwrap())
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            name: Default::default(),
            version: default_version(),
            gleam_version: Default::default(),
            description: Default::default(),
            documentation: Default::default(),
            dependencies: Default::default(),
            erlang: Default::default(),
            javascript: Default::default(),
            repository: Default::default(),
            dev_dependencies: Default::default(),
            licences: Default::default(),
            links: Default::default(),
            internal_modules: Default::default(),
            glistix: Default::default(),
            target: Target::Erlang,
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct ErlangConfig {
    #[serde(default)]
    pub application_start_module: Option<EcoString>,
    #[serde(default)]
    pub extra_applications: Vec<EcoString>,
}

#[derive(Deserialize, Debug, PartialEq, Default, Clone)]
pub struct JavaScriptConfig {
    #[serde(default)]
    pub typescript_declarations: bool,
    #[serde(default = "default_javascript_runtime")]
    pub runtime: Runtime,
    #[serde(default, rename = "deno")]
    pub deno: DenoConfig,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum DenoFlag {
    AllowAll,
    Allow(Vec<String>),
}

impl Default for DenoFlag {
    fn default() -> Self {
        Self::Allow(Vec::new())
    }
}

fn bool_or_seq_string_to_deno_flag<'de, D>(deserializer: D) -> Result<DenoFlag, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StringOrVec(PhantomData<Vec<String>>);

    impl<'de> serde::de::Visitor<'de> for StringOrVec {
        type Value = DenoFlag;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("bool or list of strings")
        }

        fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if value {
                Ok(DenoFlag::AllowAll)
            } else {
                Ok(DenoFlag::default())
            }
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
        where
            S: serde::de::SeqAccess<'de>,
        {
            let allow: Vec<String> =
                Deserialize::deserialize(serde::de::value::SeqAccessDeserializer::new(visitor))
                    .unwrap_or_default();

            Ok(DenoFlag::Allow(allow))
        }
    }

    deserializer.deserialize_any(StringOrVec(PhantomData))
}

#[derive(Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct DenoConfig {
    #[serde(default, deserialize_with = "bool_or_seq_string_to_deno_flag")]
    pub allow_env: DenoFlag,
    #[serde(default)]
    pub allow_sys: bool,
    #[serde(default)]
    pub allow_hrtime: bool,
    #[serde(default, deserialize_with = "bool_or_seq_string_to_deno_flag")]
    pub allow_net: DenoFlag,
    #[serde(default)]
    pub allow_ffi: bool,
    #[serde(default, deserialize_with = "bool_or_seq_string_to_deno_flag")]
    pub allow_read: DenoFlag,
    #[serde(default, deserialize_with = "bool_or_seq_string_to_deno_flag")]
    pub allow_run: DenoFlag,
    #[serde(default, deserialize_with = "bool_or_seq_string_to_deno_flag")]
    pub allow_write: DenoFlag,
    #[serde(default)]
    pub allow_all: bool,
    #[serde(default)]
    pub unstable: bool,
    #[serde(default, deserialize_with = "uri_serde::deserialize_option")]
    pub location: Option<Uri>,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct GlistixConfig {
    /// Config for the initial beta.
    /// Can change in the future.
    #[serde(default)]
    pub preview: GlistixPreviewConfig,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct GlistixPreviewConfig {
    /// Replaces a package with another hex package,
    /// but only while publishing. Workaround while
    /// proper patches aren't implemented upstream.
    #[serde(default, rename = "hex-patch")]
    pub hex_patch: Dependencies,

    /// List of dependencies which are bound to local paths
    /// and should override other local dependencies to the
    /// same package. Useful when you patch a library
    /// (e.g. stdlib) which is also patched by a local dependency.
    #[serde(default, rename = "local-overrides")]
    pub local_overrides: Vec<EcoString>,

    /// Replaces a package with another recursively.
    #[serde(default)]
    pub patch: GlistixPatches,
}

#[derive(Deserialize, Debug, Default, PartialEq, Eq, Clone)]
pub struct GlistixPatches(pub HashMap<EcoString, GlistixPatch>);

impl PackageConfig {
    /// Apply patches to the root config.
    ///
    /// This is identical to [`GlistixPatches::patch_config`], however it
    /// has to be reimplemented to avoid conflicting borrows on `self` (one
    /// borrowing `GlistixPatches` itself and another borrowing the whole config).
    pub fn apply_glistix_patches(&mut self) {
        self.glistix
            .preview
            .patch
            .patch_req_hash_map(&mut self.dependencies, true);

        self.glistix
            .preview
            .patch
            .patch_req_hash_map(&mut self.dev_dependencies, true);
    }

    /// Apply patches to the root config (owned version).
    pub fn with_glistix_patches_applied(mut self) -> Self {
        self.apply_glistix_patches();
        self
    }
}

impl GlistixPatches {
    /// Replace this package's name with another if necessary.
    /// Otherwise, returns the package's original name.
    #[allow(unused)]
    pub fn replace_name<'s: 'n, 'n>(&'s self, name: &'n str) -> &'n str {
        self.0
            .get(name)
            .and_then(|r| r.name.as_deref())
            .unwrap_or(name)
    }

    /// Replace this package's name with another if necessary (EcoString version).
    /// Otherwise, returns the package's original name.
    pub fn replace_name_ecostring(&self, name: EcoString) -> EcoString {
        self.0
            .get(&name)
            .and_then(|r| r.name.clone())
            .unwrap_or(name)
    }

    /// Replace all packages in a requirements hash map according to this
    /// instance's stored patches.
    pub fn patch_req_hash_map(&self, deps: &mut HashMap<EcoString, Requirement>, is_root: bool) {
        for (old_name, patch) in &self.0 {
            // If the replaced package is present, insert the replacing package.
            // Alternatively, forcefully insert the replacing package as a root
            // dependency so it is provided if it's a local or git package.
            if deps.contains_key(old_name)
                || is_root
                    && matches!(
                        patch.source,
                        Requirement::Path { .. } | Requirement::Git { .. }
                    )
            {
                _ = deps.insert(
                    patch.name.as_ref().unwrap_or(old_name).clone(),
                    patch.source.clone(),
                );
            }

            // If the replaced package is present, remove it.
            if patch
                .name
                .as_ref()
                .is_some_and(|new_name| new_name != old_name)
            {
                _ = deps.remove(old_name);
            }
        }
    }

    /// Patch a hash map of hexpm dependencies according to the specified
    /// patches. Must not correspond to root dependencies.
    pub fn patch_dep_hash_map(&self, deps: &mut HashMap<String, hexpm::Dependency>) {
        for (old_name, patch) in &self.0 {
            if let Some(mut dep) = deps.get(&**old_name).cloned() {
                // If the patched-to package is a local or git package, it is
                // provided and so always overrides hex dependencies, so we
                // don't need to change the dependency version at all since
                // the version will be whichever version is locally available.
                if let Requirement::Hex { version } = &patch.source {
                    dep.requirement = version.clone();
                }

                // Insert replacing dependency with updated name and version
                let new_name = patch.name.as_ref().unwrap_or(old_name).to_string();
                _ = deps.insert(new_name, dep);
            }

            // If the replaced package is present, remove it.
            if patch
                .name
                .as_ref()
                .is_some_and(|new_name| new_name != old_name)
            {
                // Remove replaced dependency
                _ = deps.remove(&**old_name);
            }
        }
    }

    /// Replace all packages in a config according to this instance's stored
    /// patches.
    pub fn patch_config(&self, config: &mut PackageConfig, is_root: bool) {
        self.patch_req_hash_map(&mut config.dependencies, is_root);
        self.patch_req_hash_map(&mut config.dev_dependencies, is_root);
    }

    /// Patch all requirements of a hex package.
    ///
    /// The name of the package itself is assumed to be final and kept unchanged
    /// regardless of patches, especially since this package might just be the
    /// result of an earlier patch, in which case we still need to patch
    /// requirements.
    pub fn patch_hex_package(&self, package: &mut hexpm::Package) {
        for release in &mut package.releases {
            self.patch_dep_hash_map(&mut release.requirements);
        }
    }
}

#[derive(serde::Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct GlistixPatch {
    /// Name of the package that will replace the old one.
    ///
    /// If unspecified, the old package will be kept but will use a different
    /// source (for example, to pin a different Hex version).
    #[serde(default)]
    pub name: Option<EcoString>,

    /// Version or source of the package that will replace the old one.
    ///
    /// This is specified as 'version = ...', 'path = ...' or 'git = ...'
    /// due to flattening, for simplicity.
    #[serde(flatten)]
    pub source: Requirement,
}

#[test]
fn glistix_test_patch_deps() {
    let config = PackageConfig {
        dependencies: [
            (EcoString::from("hex_to_hex"), Requirement::hex(">= 1.0.0")),
            (
                EcoString::from("local_to_hex"),
                Requirement::path("./external/local"),
            ),
            (
                EcoString::from("hex_to_local"),
                Requirement::hex(">= 1.0.0"),
            ),
            (
                EcoString::from("local_to_local"),
                Requirement::path("./external/local"),
            ),
            (EcoString::from("hex_rename"), Requirement::hex(">= 1.0.0")),
            (
                EcoString::from("local_rename"),
                Requirement::path("./external/local"),
            ),
            (
                EcoString::from("hex_to_local_rename"),
                Requirement::hex(">= 1.0.0"),
            ),
            (
                EcoString::from("unpatched_hex"),
                Requirement::hex(">= 1.0.0"),
            ),
            (
                EcoString::from("unpatched_local"),
                Requirement::path("./external/local"),
            ),
        ]
        .into_iter()
        .collect(),
        glistix: GlistixConfig {
            preview: GlistixPreviewConfig {
                patch: GlistixPatches(
                    [
                        (
                            EcoString::from("hex_to_hex"),
                            GlistixPatch {
                                name: None,
                                source: Requirement::hex("== 5.0.0"),
                            },
                        ),
                        (
                            EcoString::from("local_to_hex"),
                            GlistixPatch {
                                name: None,
                                source: Requirement::hex("== 5.0.0"),
                            },
                        ),
                        (
                            EcoString::from("hex_to_local"),
                            GlistixPatch {
                                name: None,
                                source: Requirement::path("./external/patched"),
                            },
                        ),
                        (
                            EcoString::from("local_to_local"),
                            GlistixPatch {
                                name: None,
                                source: Requirement::path("./external/patched"),
                            },
                        ),
                        (
                            EcoString::from("hex_rename"),
                            GlistixPatch {
                                name: Some(EcoString::from("hex_did_rename")),
                                source: Requirement::hex("== 5.0.0"),
                            },
                        ),
                        (
                            EcoString::from("local_rename"),
                            GlistixPatch {
                                name: Some(EcoString::from("local_did_rename")),
                                source: Requirement::path("./external/patched"),
                            },
                        ),
                        (
                            EcoString::from("hex_to_local_rename"),
                            GlistixPatch {
                                name: Some(EcoString::from("hex_to_local_did_rename")),
                                source: Requirement::path("./external/patched"),
                            },
                        ),
                        (
                            EcoString::from("unused_hex_rename"),
                            GlistixPatch {
                                name: Some(EcoString::from("unused_did_hex_rename")),
                                source: Requirement::hex("== 5.0.0"),
                            },
                        ),
                        (
                            EcoString::from("unused_local_rename"),
                            GlistixPatch {
                                name: Some(EcoString::from("unused_did_local_rename")),
                                source: Requirement::path("./external/patched"),
                            },
                        ),
                    ]
                    .into_iter()
                    .collect(),
                ),
                ..Default::default()
            },
        },
        ..Default::default()
    };

    assert_eq!(
        config.with_glistix_patches_applied().dependencies,
        [
            (EcoString::from("hex_to_hex"), Requirement::hex("== 5.0.0")),
            (
                EcoString::from("local_to_hex"),
                Requirement::hex("== 5.0.0")
            ),
            (
                EcoString::from("hex_to_local"),
                Requirement::path("./external/patched")
            ),
            (
                EcoString::from("local_to_local"),
                Requirement::path("./external/patched")
            ),
            (
                EcoString::from("hex_did_rename"),
                Requirement::hex("== 5.0.0")
            ),
            (
                EcoString::from("local_did_rename"),
                Requirement::path("./external/patched")
            ),
            (
                EcoString::from("hex_to_local_did_rename"),
                Requirement::path("./external/patched")
            ),
            (
                EcoString::from("unpatched_hex"),
                Requirement::hex(">= 1.0.0")
            ),
            (
                EcoString::from("unpatched_local"),
                Requirement::path("./external/local")
            ),
            // Any 'to local' patches must become dependencies so they are provided.
            // Unused 'to hex' patches do not, however, as they are fetched on the fly.
            (
                EcoString::from("unused_did_local_rename"),
                Requirement::path("./external/patched")
            ),
        ]
        .into_iter()
        .collect()
    );
}

#[test]
fn glistix_test_patch_dev_deps() {
    let config = PackageConfig {
        dev_dependencies: [
            (EcoString::from("hex_to_hex"), Requirement::hex(">= 1.0.0")),
            (
                EcoString::from("local_to_hex"),
                Requirement::path("./external/local"),
            ),
            (
                EcoString::from("hex_to_local"),
                Requirement::hex(">= 1.0.0"),
            ),
            (
                EcoString::from("local_to_local"),
                Requirement::path("./external/local"),
            ),
            (EcoString::from("hex_rename"), Requirement::hex(">= 1.0.0")),
            (
                EcoString::from("local_rename"),
                Requirement::path("./external/local"),
            ),
            (
                EcoString::from("hex_to_local_rename"),
                Requirement::hex(">= 1.0.0"),
            ),
            (
                EcoString::from("unpatched_hex"),
                Requirement::hex(">= 1.0.0"),
            ),
            (
                EcoString::from("unpatched_local"),
                Requirement::path("./external/local"),
            ),
        ]
        .into_iter()
        .collect(),
        glistix: GlistixConfig {
            preview: GlistixPreviewConfig {
                patch: GlistixPatches(
                    [
                        (
                            EcoString::from("hex_to_hex"),
                            GlistixPatch {
                                name: None,
                                source: Requirement::hex("== 5.0.0"),
                            },
                        ),
                        (
                            EcoString::from("local_to_hex"),
                            GlistixPatch {
                                name: None,
                                source: Requirement::hex("== 5.0.0"),
                            },
                        ),
                        (
                            EcoString::from("hex_to_local"),
                            GlistixPatch {
                                name: None,
                                source: Requirement::path("./external/patched"),
                            },
                        ),
                        (
                            EcoString::from("local_to_local"),
                            GlistixPatch {
                                name: None,
                                source: Requirement::path("./external/patched"),
                            },
                        ),
                        (
                            EcoString::from("hex_rename"),
                            GlistixPatch {
                                name: Some(EcoString::from("hex_did_rename")),
                                source: Requirement::hex("== 5.0.0"),
                            },
                        ),
                        (
                            EcoString::from("local_rename"),
                            GlistixPatch {
                                name: Some(EcoString::from("local_did_rename")),
                                source: Requirement::path("./external/patched"),
                            },
                        ),
                        (
                            EcoString::from("hex_to_local_rename"),
                            GlistixPatch {
                                name: Some(EcoString::from("hex_to_local_did_rename")),
                                source: Requirement::path("./external/patched"),
                            },
                        ),
                        (
                            EcoString::from("unused_hex_rename"),
                            GlistixPatch {
                                name: Some(EcoString::from("unused_did_hex_rename")),
                                source: Requirement::hex("== 5.0.0"),
                            },
                        ),
                        (
                            EcoString::from("unused_local_rename"),
                            GlistixPatch {
                                name: Some(EcoString::from("unused_did_local_rename")),
                                source: Requirement::path("./external/patched"),
                            },
                        ),
                    ]
                    .into_iter()
                    .collect(),
                ),
                ..Default::default()
            },
        },
        ..Default::default()
    };

    assert_eq!(
        config.with_glistix_patches_applied().dev_dependencies,
        [
            (EcoString::from("hex_to_hex"), Requirement::hex("== 5.0.0")),
            (
                EcoString::from("local_to_hex"),
                Requirement::hex("== 5.0.0")
            ),
            (
                EcoString::from("hex_to_local"),
                Requirement::path("./external/patched")
            ),
            (
                EcoString::from("local_to_local"),
                Requirement::path("./external/patched")
            ),
            (
                EcoString::from("hex_did_rename"),
                Requirement::hex("== 5.0.0")
            ),
            (
                EcoString::from("local_did_rename"),
                Requirement::path("./external/patched")
            ),
            (
                EcoString::from("hex_to_local_did_rename"),
                Requirement::path("./external/patched")
            ),
            (
                EcoString::from("unpatched_hex"),
                Requirement::hex(">= 1.0.0")
            ),
            (
                EcoString::from("unpatched_local"),
                Requirement::path("./external/local")
            ),
            // Any 'to local' patches must become dependencies so they are provided.
            // Unused 'to hex' patches do not, however, as they are fetched on the fly.
            (
                EcoString::from("unused_did_local_rename"),
                Requirement::path("./external/patched")
            ),
        ]
        .into_iter()
        .collect()
    );
}
#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Repository {
    GitHub {
        user: String,
        repo: String,
        path: Option<String>,
    },
    GitLab {
        user: String,
        repo: String,
        path: Option<String>,
    },
    BitBucket {
        user: String,
        repo: String,
        path: Option<String>,
    },
    Codeberg {
        user: String,
        repo: String,
        path: Option<String>,
    },
    #[serde(alias = "forgejo")]
    Gitea {
        user: String,
        repo: String,
        path: Option<String>,
        #[serde(with = "uri_serde_default_https")]
        host: Uri,
    },
    SourceHut {
        user: String,
        repo: String,
        path: Option<String>,
    },
    Custom {
        url: String,
    },
    None,
}

impl Repository {
    pub fn url(&self) -> Option<String> {
        match self {
            Repository::GitHub { repo, user, .. } => {
                Some(format!("https://github.com/{user}/{repo}"))
            }
            Repository::GitLab { repo, user, .. } => {
                Some(format!("https://gitlab.com/{user}/{repo}"))
            }
            Repository::BitBucket { repo, user, .. } => {
                Some(format!("https://bitbucket.com/{user}/{repo}"))
            }
            Repository::Codeberg { repo, user, .. } => {
                Some(format!("https://codeberg.org/{user}/{repo}"))
            }
            Repository::SourceHut { repo, user, .. } => {
                Some(format!("https://git.sr.ht/~{user}/{repo}"))
            }
            Repository::Gitea {
                repo, user, host, ..
            } => Some(format!("{host}/{user}/{repo}")),
            Repository::Custom { url } => Some(url.clone()),
            Repository::None => None,
        }
    }

    pub fn path(&self) -> Option<&String> {
        match self {
            Repository::GitHub { path, .. }
            | Repository::GitLab { path, .. }
            | Repository::BitBucket { path, .. }
            | Repository::Codeberg { path, .. }
            | Repository::SourceHut { path, .. }
            | Repository::Gitea { path, .. } => path.as_ref(),

            Repository::Custom { .. } | Repository::None => None,
        }
    }
}

impl Default for Repository {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Deserialize, Default, Debug, PartialEq, Eq, Clone)]
pub struct Docs {
    #[serde(default)]
    pub pages: Vec<DocsPage>,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct DocsPage {
    pub title: String,
    pub path: String,
    pub source: Utf8PathBuf,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Link {
    pub title: String,
    #[serde(with = "uri_serde")]
    pub href: Uri,
}

// Note we don't use http-serde since we also want to validate the scheme and host is set.
mod uri_serde {
    use http::uri::InvalidUri;
    use serde::{de::Error as _, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<http::Uri, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        let uri: http::Uri = string
            .parse()
            .map_err(|err: InvalidUri| D::Error::custom(err.to_string()))?;
        if uri.scheme().is_none() || uri.host().is_none() {
            return Err(D::Error::custom("uri without scheme"));
        }
        Ok(uri)
    }

    pub fn deserialize_option<'de, D>(deserializer: D) -> Result<Option<http::Uri>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string: Option<String> = Option::deserialize(deserializer)?;
        match string {
            Some(s) => {
                let deserializer = serde::de::value::StringDeserializer::new(s);
                deserialize(deserializer).map(Some)
            }
            None => Ok(None),
        }
    }
}

// This prefixes https as a default in the event no scheme was provided
mod uri_serde_default_https {
    use http::uri::InvalidUri;
    use serde::{de::Error as _, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<http::Uri, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        let uri: http::Uri = string
            .parse()
            .map_err(|err: InvalidUri| D::Error::custom(err.to_string()))?;
        if uri.host().is_none() {
            return Err(D::Error::custom("uri without host"));
        }
        match uri.scheme().is_none() {
            true => format!("https://{string}")
                .parse()
                .map_err(|err: InvalidUri| D::Error::custom(err.to_string())),
            false => Ok(uri),
        }
    }
}

mod package_name {
    use ecow::EcoString;
    use regex::Regex;
    use serde::Deserializer;
    use std::sync::OnceLock;

    static PACKAGE_NAME_PATTERN: OnceLock<Regex> = OnceLock::new();

    pub fn deserialize<'de, D>(deserializer: D) -> Result<EcoString, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name: &str = serde::de::Deserialize::deserialize(deserializer)?;
        if PACKAGE_NAME_PATTERN
            .get_or_init(|| Regex::new("^[a-z][a-z0-9_]*$").expect("Package name regex"))
            .is_match(name)
        {
            Ok(name.into())
        } else {
            let error =
                "Package names may only contain lowercase letters, numbers, and underscores";
            Err(serde::de::Error::custom(error))
        }
    }
}

#[test]
fn name_with_dash() {
    let input = r#"
name = "one-two"
"#;
    assert_eq!(
        toml::from_str::<PackageConfig>(input)
            .unwrap_err()
            .to_string(),
        "Package names may only contain lowercase letters, numbers, and underscores for key `name` at line 1 column 1"
    )
}

#[test]
fn name_with_number_start() {
    let input = r#"
name = "1"
"#;
    assert_eq!(
        toml::from_str::<PackageConfig>(input)
            .unwrap_err()
            .to_string(),
        "Package names may only contain lowercase letters, numbers, and underscores for key `name` at line 1 column 1"
    )
}
