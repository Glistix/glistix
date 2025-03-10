use std::collections::HashMap;

use crate::io::{make_relative, ordered_map};
use crate::requirement::Requirement;
use crate::Result;
use camino::{Utf8Path, Utf8PathBuf};
use ecow::EcoString;
use hexpm::version::Version;
use itertools::Itertools;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Manifest {
    #[serde(serialize_with = "ordered_map")]
    pub requirements: HashMap<EcoString, Requirement>,
    #[serde(serialize_with = "sorted_vec")]
    pub packages: Vec<ManifestPackage>,
    #[serde(default)]
    pub glistix: GlistixManifest,
}

impl Manifest {
    // Rather than using the toml library to do serialization we implement it
    // manually so that we can control the formatting.
    // We want to keep entries on a single line each so that they are more
    // resistant to merge conflicts and are easier to fix when it does happen.
    pub fn to_toml(&self, root_path: &Utf8Path) -> String {
        let mut buffer = String::new();
        let Self {
            requirements,
            packages,
            glistix,
        } = self;

        buffer.push_str(
            "# This file was generated by Gleam
# You typically do not need to edit this file

",
        );

        // Packages
        buffer.push_str("packages = [\n");
        for ManifestPackage {
            name,
            source,
            version,
            otp_app,
            build_tools,
            requirements,
        } in packages.iter().sorted_by(|a, b| a.name.cmp(&b.name))
        {
            buffer.push_str(r#"  {"#);
            buffer.push_str(r#" name = ""#);
            buffer.push_str(name);
            buffer.push_str(r#"", version = ""#);
            buffer.push_str(&version.to_string());
            buffer.push_str(r#"", build_tools = ["#);
            for (i, tool) in build_tools.iter().enumerate() {
                if i != 0 {
                    buffer.push_str(", ");
                }
                buffer.push('"');
                buffer.push_str(tool);
                buffer.push('"');
            }

            buffer.push_str("], requirements = [");
            for (i, package) in requirements.iter().sorted_by(|a, b| a.cmp(b)).enumerate() {
                if i != 0 {
                    buffer.push_str(", ");
                }
                buffer.push('"');
                buffer.push_str(package);
                buffer.push('"');
            }
            buffer.push(']');

            if let Some(app) = otp_app {
                buffer.push_str(", otp_app = \"");
                buffer.push_str(app);
                buffer.push('"');
            }

            match source {
                ManifestPackageSource::Hex { outer_checksum } => {
                    buffer.push_str(r#", source = "hex", outer_checksum = ""#);
                    buffer.push_str(&outer_checksum.to_string());
                    buffer.push('"');
                }
                ManifestPackageSource::Git { repo, commit } => {
                    buffer.push_str(r#", source = "git", repo = ""#);
                    buffer.push_str(repo);
                    buffer.push_str(r#"", commit = ""#);
                    buffer.push_str(commit);
                    buffer.push('"');
                }
                ManifestPackageSource::Local { path } => {
                    buffer.push_str(r#", source = "local", path = ""#);
                    buffer.push_str(&make_relative(root_path, path).as_str().replace('\\', "/"));
                    buffer.push('"');
                }
            };

            buffer.push_str(" },\n");
        }
        buffer.push_str("]\n\n");

        // Requirements
        buffer.push_str("[requirements]\n");
        for (name, requirement) in requirements.iter().sorted_by(|a, b| a.0.cmp(b.0)) {
            buffer.push_str(name);
            buffer.push_str(" = ");
            buffer.push_str(&requirement.to_toml(root_path));
            buffer.push('\n');
        }

        if !glistix.preview.patch.0.is_empty() {
            buffer.push_str("\n[glistix.preview.patch]\n");
        }
        for (name, patch) in glistix
            .preview
            .patch
            .0
            .iter()
            .sorted_by(|a, b| a.0.cmp(b.0))
        {
            buffer.push_str(name);
            buffer.push_str(" = ");
            if let Some(new_name) = &patch.name {
                buffer.push_str("{ name = \"");
                buffer.push_str(new_name);
                buffer.push_str("\", ");
                buffer.push_str(patch.source.to_toml(root_path).trim_start_matches("{ "));
            } else {
                buffer.push_str(&patch.source.to_toml(root_path));
            }
            buffer.push('\n');
        }

        buffer
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Base16Checksum(pub Vec<u8>);

impl ToString for Base16Checksum {
    fn to_string(&self) -> String {
        base16::encode_upper(&self.0)
    }
}

impl serde::Serialize for Base16Checksum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&base16::encode_upper(&self.0))
    }
}

impl<'de> serde::Deserialize<'de> for Base16Checksum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::de::Deserialize::deserialize(deserializer)?;
        base16::decode(s)
            .map(Base16Checksum)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct ManifestPackage {
    pub name: EcoString,
    pub version: Version,
    pub build_tools: Vec<EcoString>,
    #[serde(default)]
    pub otp_app: Option<EcoString>,
    #[serde(serialize_with = "sorted_vec")]
    pub requirements: Vec<EcoString>,
    #[serde(flatten)]
    pub source: ManifestPackageSource,
}

impl ManifestPackage {
    pub fn with_build_tools(mut self, build_tools: &'static [&'static str]) -> Self {
        self.build_tools = build_tools.iter().map(|s| (*s).into()).collect();
        self
    }

    pub fn application_name(&self) -> &EcoString {
        match self.otp_app {
            Some(ref app) => app,
            None => &self.name,
        }
    }

    #[inline]
    pub fn is_hex(&self) -> bool {
        matches!(self.source, ManifestPackageSource::Hex { .. })
    }

    #[inline]
    pub fn is_local(&self) -> bool {
        matches!(self.source, ManifestPackageSource::Local { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(tag = "source")]
pub enum ManifestPackageSource {
    #[serde(rename = "hex")]
    Hex { outer_checksum: Base16Checksum },
    #[serde(rename = "git")]
    Git { repo: EcoString, commit: EcoString },
    #[serde(rename = "local")]
    Local { path: Utf8PathBuf }, // should be the canonical path
}

fn sorted_vec<S, T>(value: &[T], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: serde::Serialize + Ord,
{
    use serde::Serialize;
    let mut value: Vec<&T> = value.iter().collect();
    value.sort();
    value.serialize(serializer)
}

/// Glistix manifest information.
///
/// This will record patches. When patches change, we need to update the
/// manifest.
///
/// Can be omitted from the manifest for compatibility with existing Gleam
/// manifests. That will indicate there were no patches when the manifest was
/// recorded.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct GlistixManifest {
    #[serde(default)]
    pub preview: GlistixPreviewManifest,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct GlistixPreviewManifest {
    #[serde(default, serialize_with = "ordered_glistix_patches")]
    pub patch: crate::config::GlistixPatches,
}

impl GlistixManifest {
    pub fn with_patches(patch: crate::config::GlistixPatches) -> Self {
        Self {
            preview: GlistixPreviewManifest { patch },
        }
    }
}

fn ordered_glistix_patches<S>(
    value: &crate::config::GlistixPatches,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    ordered_map(&value.0, serializer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    const HOME: &'static str = "C:\\home\\louis\\packages\\some_folder";

    #[cfg(windows)]
    const PACKAGE: &'static str = "C:\\home\\louis\\packages\\path\\to\\package";

    #[cfg(windows)]
    const PACKAGE_WITH_UNC: &'static str = "\\\\?\\C:\\home\\louis\\packages\\path\\to\\package";

    #[cfg(not(windows))]
    const HOME: &str = "/home/louis/packages/some_folder";

    #[cfg(not(windows))]
    const PACKAGE: &str = "/home/louis/packages/path/to/package";

    #[test]
    fn manifest_toml_format() {
        let manifest = Manifest {
            requirements: [
                ("zzz".into(), Requirement::hex("> 0.0.0")),
                ("aaa".into(), Requirement::hex("> 0.0.0")),
                (
                    "awsome_local2".into(),
                    Requirement::git("https://github.com/gleam-lang/gleam.git"),
                ),
                (
                    "awsome_local1".into(),
                    Requirement::path("../path/to/package"),
                ),
                ("gleam_stdlib".into(), Requirement::hex("~> 0.17")),
                ("gleeunit".into(), Requirement::hex("~> 0.1")),
            ]
            .into(),
            packages: vec![
                ManifestPackage {
                    name: "gleam_stdlib".into(),
                    version: Version::new(0, 17, 1),
                    build_tools: ["gleam".into()].into(),
                    otp_app: None,
                    requirements: vec![],
                    source: ManifestPackageSource::Hex {
                        outer_checksum: Base16Checksum(vec![1, 22]),
                    },
                },
                ManifestPackage {
                    name: "aaa".into(),
                    version: Version::new(0, 4, 0),
                    build_tools: ["rebar3".into(), "make".into()].into(),
                    otp_app: Some("aaa_app".into()),
                    requirements: vec!["zzz".into(), "gleam_stdlib".into()],
                    source: ManifestPackageSource::Hex {
                        outer_checksum: Base16Checksum(vec![3, 22]),
                    },
                },
                ManifestPackage {
                    name: "zzz".into(),
                    version: Version::new(0, 4, 0),
                    build_tools: ["mix".into()].into(),
                    otp_app: None,
                    requirements: vec![],
                    source: ManifestPackageSource::Hex {
                        outer_checksum: Base16Checksum(vec![3, 22]),
                    },
                },
                ManifestPackage {
                    name: "awsome_local2".into(),
                    version: Version::new(1, 2, 3),
                    build_tools: ["gleam".into()].into(),
                    otp_app: None,
                    requirements: vec![],
                    source: ManifestPackageSource::Git {
                        repo: "https://github.com/gleam-lang/gleam.git".into(),
                        commit: "bd9fe02f72250e6a136967917bcb1bdccaffa3c8".into(),
                    },
                },
                ManifestPackage {
                    name: "awsome_local1".into(),
                    version: Version::new(1, 2, 3),
                    build_tools: ["gleam".into()].into(),
                    otp_app: None,
                    requirements: vec![],
                    source: ManifestPackageSource::Local {
                        path: PACKAGE.into(),
                    },
                },
                ManifestPackage {
                    name: "gleeunit".into(),
                    version: Version::new(0, 4, 0),
                    build_tools: ["gleam".into()].into(),
                    otp_app: None,
                    requirements: vec!["gleam_stdlib".into()],
                    source: ManifestPackageSource::Hex {
                        outer_checksum: Base16Checksum(vec![3, 46]),
                    },
                },
            ],
            glistix: Default::default(),
        };

        let buffer = manifest.to_toml(HOME.into());
        assert_eq!(
            buffer,
            r#"# This file was generated by Gleam
# You typically do not need to edit this file

packages = [
  { name = "aaa", version = "0.4.0", build_tools = ["rebar3", "make"], requirements = ["gleam_stdlib", "zzz"], otp_app = "aaa_app", source = "hex", outer_checksum = "0316" },
  { name = "awsome_local1", version = "1.2.3", build_tools = ["gleam"], requirements = [], source = "local", path = "../path/to/package" },
  { name = "awsome_local2", version = "1.2.3", build_tools = ["gleam"], requirements = [], source = "git", repo = "https://github.com/gleam-lang/gleam.git", commit = "bd9fe02f72250e6a136967917bcb1bdccaffa3c8" },
  { name = "gleam_stdlib", version = "0.17.1", build_tools = ["gleam"], requirements = [], source = "hex", outer_checksum = "0116" },
  { name = "gleeunit", version = "0.4.0", build_tools = ["gleam"], requirements = ["gleam_stdlib"], source = "hex", outer_checksum = "032E" },
  { name = "zzz", version = "0.4.0", build_tools = ["mix"], requirements = [], source = "hex", outer_checksum = "0316" },
]

[requirements]
aaa = { version = "> 0.0.0" }
awsome_local1 = { path = "../path/to/package" }
awsome_local2 = { git = "https://github.com/gleam-lang/gleam.git" }
gleam_stdlib = { version = "~> 0.17" }
gleeunit = { version = "~> 0.1" }
zzz = { version = "> 0.0.0" }
"#
        );
    }

    #[cfg(windows)]
    #[test]
    fn manifest_toml_format_with_unc() {
        let manifest = Manifest {
            requirements: [
                ("zzz".into(), Requirement::hex("> 0.0.0")),
                ("aaa".into(), Requirement::hex("> 0.0.0")),
                (
                    "awsome_local2".into(),
                    Requirement::git("https://github.com/gleam-lang/gleam.git"),
                ),
                (
                    "awsome_local1".into(),
                    Requirement::path("../path/to/package"),
                ),
                ("gleam_stdlib".into(), Requirement::hex("~> 0.17")),
                ("gleeunit".into(), Requirement::hex("~> 0.1")),
            ]
            .into(),
            packages: vec![
                ManifestPackage {
                    name: "gleam_stdlib".into(),
                    version: Version::new(0, 17, 1),
                    build_tools: ["gleam".into()].into(),
                    otp_app: None,
                    requirements: vec![],
                    source: ManifestPackageSource::Hex {
                        outer_checksum: Base16Checksum(vec![1, 22]),
                    },
                },
                ManifestPackage {
                    name: "aaa".into(),
                    version: Version::new(0, 4, 0),
                    build_tools: ["rebar3".into(), "make".into()].into(),
                    otp_app: Some("aaa_app".into()),
                    requirements: vec!["zzz".into(), "gleam_stdlib".into()],
                    source: ManifestPackageSource::Hex {
                        outer_checksum: Base16Checksum(vec![3, 22]),
                    },
                },
                ManifestPackage {
                    name: "zzz".into(),
                    version: Version::new(0, 4, 0),
                    build_tools: ["mix".into()].into(),
                    otp_app: None,
                    requirements: vec![],
                    source: ManifestPackageSource::Hex {
                        outer_checksum: Base16Checksum(vec![3, 22]),
                    },
                },
                ManifestPackage {
                    name: "awsome_local2".into(),
                    version: Version::new(1, 2, 3),
                    build_tools: ["gleam".into()].into(),
                    otp_app: None,
                    requirements: vec![],
                    source: ManifestPackageSource::Git {
                        repo: "https://github.com/gleam-lang/gleam.git".into(),
                        commit: "bd9fe02f72250e6a136967917bcb1bdccaffa3c8".into(),
                    },
                },
                ManifestPackage {
                    name: "awsome_local1".into(),
                    version: Version::new(1, 2, 3),
                    build_tools: ["gleam".into()].into(),
                    otp_app: None,
                    requirements: vec![],
                    source: ManifestPackageSource::Local {
                        path: PACKAGE_WITH_UNC.into(),
                    },
                },
                ManifestPackage {
                    name: "gleeunit".into(),
                    version: Version::new(0, 4, 0),
                    build_tools: ["gleam".into()].into(),
                    otp_app: None,
                    requirements: vec!["gleam_stdlib".into()],
                    source: ManifestPackageSource::Hex {
                        outer_checksum: Base16Checksum(vec![3, 46]),
                    },
                },
            ],
            glistix: Default::default(),
        };

        let buffer = manifest.to_toml(HOME.into());
        assert_eq!(
            buffer,
            r#"# This file was generated by Gleam
# You typically do not need to edit this file

packages = [
  { name = "aaa", version = "0.4.0", build_tools = ["rebar3", "make"], requirements = ["gleam_stdlib", "zzz"], otp_app = "aaa_app", source = "hex", outer_checksum = "0316" },
  { name = "awsome_local1", version = "1.2.3", build_tools = ["gleam"], requirements = [], source = "local", path = "../path/to/package" },
  { name = "awsome_local2", version = "1.2.3", build_tools = ["gleam"], requirements = [], source = "git", repo = "https://github.com/gleam-lang/gleam.git", commit = "bd9fe02f72250e6a136967917bcb1bdccaffa3c8" },
  { name = "gleam_stdlib", version = "0.17.1", build_tools = ["gleam"], requirements = [], source = "hex", outer_checksum = "0116" },
  { name = "gleeunit", version = "0.4.0", build_tools = ["gleam"], requirements = ["gleam_stdlib"], source = "hex", outer_checksum = "032E" },
  { name = "zzz", version = "0.4.0", build_tools = ["mix"], requirements = [], source = "hex", outer_checksum = "0316" },
]

[requirements]
aaa = { version = "> 0.0.0" }
awsome_local1 = { path = "../path/to/package" }
awsome_local2 = { git = "https://github.com/gleam-lang/gleam.git" }
gleam_stdlib = { version = "~> 0.17" }
gleeunit = { version = "~> 0.1" }
zzz = { version = "> 0.0.0" }
"#
        );
    }

    #[test]
    fn glistix_manifest_toml_format_with_patches() {
        let manifest = Manifest {
            requirements: [
                ("zzz".into(), Requirement::hex("> 0.0.0")),
                ("aaa".into(), Requirement::hex("> 0.0.0")),
                (
                    "awsome_local2".into(),
                    Requirement::git("https://github.com/gleam-lang/gleam.git"),
                ),
                (
                    "awsome_local1".into(),
                    Requirement::path("../path/to/package"),
                ),
                ("gleam_stdlib".into(), Requirement::hex("~> 0.17")),
                ("gleeunit".into(), Requirement::hex("~> 0.1")),
            ]
            .into(),
            packages: vec![
                ManifestPackage {
                    name: "gleam_stdlib".into(),
                    version: Version::new(0, 17, 1),
                    build_tools: ["gleam".into()].into(),
                    otp_app: None,
                    requirements: vec![],
                    source: ManifestPackageSource::Hex {
                        outer_checksum: Base16Checksum(vec![1, 22]),
                    },
                },
                ManifestPackage {
                    name: "aaa".into(),
                    version: Version::new(0, 4, 0),
                    build_tools: ["rebar3".into(), "make".into()].into(),
                    otp_app: Some("aaa_app".into()),
                    requirements: vec!["zzz".into(), "gleam_stdlib".into()],
                    source: ManifestPackageSource::Hex {
                        outer_checksum: Base16Checksum(vec![3, 22]),
                    },
                },
                ManifestPackage {
                    name: "zzz".into(),
                    version: Version::new(0, 4, 0),
                    build_tools: ["mix".into()].into(),
                    otp_app: None,
                    requirements: vec![],
                    source: ManifestPackageSource::Hex {
                        outer_checksum: Base16Checksum(vec![3, 22]),
                    },
                },
                ManifestPackage {
                    name: "awsome_local2".into(),
                    version: Version::new(1, 2, 3),
                    build_tools: ["gleam".into()].into(),
                    otp_app: None,
                    requirements: vec![],
                    source: ManifestPackageSource::Git {
                        repo: "https://github.com/gleam-lang/gleam.git".into(),
                        commit: "bd9fe02f72250e6a136967917bcb1bdccaffa3c8".into(),
                    },
                },
                ManifestPackage {
                    name: "awsome_local1".into(),
                    version: Version::new(1, 2, 3),
                    build_tools: ["gleam".into()].into(),
                    otp_app: None,
                    requirements: vec![],
                    source: ManifestPackageSource::Local {
                        path: PACKAGE.into(),
                    },
                },
                ManifestPackage {
                    name: "gleeunit".into(),
                    version: Version::new(0, 4, 0),
                    build_tools: ["gleam".into()].into(),
                    otp_app: None,
                    requirements: vec!["gleam_stdlib".into()],
                    source: ManifestPackageSource::Hex {
                        outer_checksum: Base16Checksum(vec![3, 46]),
                    },
                },
            ],
            glistix: GlistixManifest {
                preview: GlistixPreviewManifest {
                    patch: crate::config::GlistixPatches({
                        let mut patches = HashMap::new();
                        _ = patches.insert(
                            ecow::eco_format!("first_package"),
                            crate::config::GlistixPatch {
                                name: Some(ecow::eco_format!("renamed")),
                                source: Requirement::hex(">= 0.1.0"),
                            },
                        );
                        _ = patches.insert(
                            ecow::eco_format!("second_package"),
                            crate::config::GlistixPatch {
                                name: None,
                                source: Requirement::hex(">= 0.2.0"),
                            },
                        );
                        _ = patches.insert(
                            ecow::eco_format!("third_package"),
                            crate::config::GlistixPatch {
                                name: Some(ecow::eco_format!("renamed")),
                                source: Requirement::path("./abc/def"),
                            },
                        );
                        _ = patches.insert(
                            ecow::eco_format!("fourth_package"),
                            crate::config::GlistixPatch {
                                name: None,
                                source: Requirement::path("./abc/def"),
                            },
                        );
                        patches
                    }),
                },
            },
        };

        let buffer = manifest.to_toml(HOME.into());
        assert_eq!(
            buffer,
            r#"# This file was generated by Gleam
# You typically do not need to edit this file

packages = [
  { name = "aaa", version = "0.4.0", build_tools = ["rebar3", "make"], requirements = ["gleam_stdlib", "zzz"], otp_app = "aaa_app", source = "hex", outer_checksum = "0316" },
  { name = "awsome_local1", version = "1.2.3", build_tools = ["gleam"], requirements = [], source = "local", path = "../path/to/package" },
  { name = "awsome_local2", version = "1.2.3", build_tools = ["gleam"], requirements = [], source = "git", repo = "https://github.com/gleam-lang/gleam.git", commit = "bd9fe02f72250e6a136967917bcb1bdccaffa3c8" },
  { name = "gleam_stdlib", version = "0.17.1", build_tools = ["gleam"], requirements = [], source = "hex", outer_checksum = "0116" },
  { name = "gleeunit", version = "0.4.0", build_tools = ["gleam"], requirements = ["gleam_stdlib"], source = "hex", outer_checksum = "032E" },
  { name = "zzz", version = "0.4.0", build_tools = ["mix"], requirements = [], source = "hex", outer_checksum = "0316" },
]

[requirements]
aaa = { version = "> 0.0.0" }
awsome_local1 = { path = "../path/to/package" }
awsome_local2 = { git = "https://github.com/gleam-lang/gleam.git" }
gleam_stdlib = { version = "~> 0.17" }
gleeunit = { version = "~> 0.1" }
zzz = { version = "> 0.0.0" }

[glistix.preview.patch]
first_package = { name = "renamed", version = ">= 0.1.0" }
fourth_package = { path = "./abc/def" }
second_package = { version = ">= 0.2.0" }
third_package = { name = "renamed", path = "./abc/def" }
"#
        );
    }

    impl Default for ManifestPackage {
        fn default() -> Self {
            Self {
                name: Default::default(),
                build_tools: Default::default(),
                otp_app: Default::default(),
                requirements: Default::default(),
                version: Version::new(1, 0, 0),
                source: ManifestPackageSource::Hex {
                    outer_checksum: Base16Checksum(vec![]),
                },
            }
        }
    }
}
