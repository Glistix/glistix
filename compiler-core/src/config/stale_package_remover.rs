use crate::manifest::Manifest;
use crate::requirement::Requirement;
use ecow::EcoString;
use hexpm::version::Version;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct StalePackageRemover<'a> {
    // These are the packages for which the requirement or their parents
    // requirement has not changed.
    fresh: HashSet<&'a str>,
    locked: HashMap<EcoString, &'a Vec<EcoString>>,
}

impl<'a> StalePackageRemover<'a> {
    pub fn fresh_and_locked(
        requirements: &'a HashMap<EcoString, Requirement>,
        manifest: &'a Manifest,
        glistix_patches: &'a super::GlistixPatches,
    ) -> HashMap<EcoString, Version> {
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
        glistix_patches: &'a super::GlistixPatches,
    ) -> HashMap<EcoString, Version> {
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
            self.record_tree_fresh(name, &glistix_newly_patched_packages);
        }

        // Return all the previously resolved packages that have not been
        // recorded as fresh
        manifest
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
            .collect()
    }

    fn record_tree_fresh(&mut self, name: &'a str, glistix_newly_patched_packages: &[&'a str]) {
        // Record the top level package
        let _ = self.fresh.insert(name);

        let Some(deps) = self.locked.get(name) else {
            // If the package is not in the manifest then it means that the package is an optional
            // dependency that has not been included. That or someone has been editing the manifest
            // and broken it, but let's hope that's not the case.
            return;
        };

        // Record each of its deps recursively
        for package in *deps {
            if glistix_newly_patched_packages.contains(&&**package) {
                // Dep was affected by a patch
                continue;
            }

            self.record_tree_fresh(package, glistix_newly_patched_packages);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Base16Checksum, Manifest, ManifestPackage, ManifestPackageSource};
    use crate::requirement::Requirement;
    use hexpm::version::{Range, Version};
    use std::collections::HashMap;

    // https://github.com/gleam-lang/gleam/issues/4152
    #[test]
    fn optional_package_not_in_manifest() {
        let requirements = HashMap::from_iter([(
            "required_package".into(),
            Requirement::Hex {
                version: Range::new("1.0.0".into()),
            },
        )]);
        let manifest = Manifest {
            requirements: requirements.clone(),
            packages: vec![ManifestPackage {
                name: "required_package".into(),
                version: Version::new(1, 0, 0),
                build_tools: vec!["gleam".into()],
                otp_app: None,
                requirements: vec![
                    // NOTE: this package isn't in the manifest. This will have been because it is
                    // an optional dep of `required_package`.
                    "optional_package".into(),
                ],
                source: ManifestPackageSource::Hex {
                    outer_checksum: Base16Checksum(vec![]),
                },
            }],
            glistix: Default::default(),
        };

        assert_eq!(
            StalePackageRemover::fresh_and_locked(&requirements, &manifest, &Default::default()),
            HashMap::from_iter([("required_package".into(), Version::new(1, 0, 0))])
        );
    }
}
