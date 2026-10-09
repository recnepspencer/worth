use crate::config::SubworkspaceConfig;
use crate::config::{QueryAudienceContract, SnapshotDependencyPackagesConfig};
use crate::manifest_types::{CargoMetadata, CargoMetadataPackage, Road1Package, WorkspaceManifest};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) fn parse_workspace_manifest(path: &Path) -> Result<WorkspaceManifest, String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("read manifest {}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("parse manifest {}: {e}", path.display()))
}

pub(crate) fn cargo_metadata(root: &Path, manifest_path: &Path) -> Result<CargoMetadata, String> {
    let cargo_manifest_path = cargo_compatible_path(manifest_path);
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .arg("--manifest-path")
        .arg(&cargo_manifest_path)
        .current_dir(root)
        .output()
        .map_err(|e| format!("spawn cargo metadata for {}: {e}", manifest_path.display()))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed for {}: {}",
            manifest_path.display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|e| {
        format!(
            "parse cargo metadata output for {}: {e}",
            manifest_path.display()
        )
    })
}

#[cfg(windows)]
fn cargo_compatible_path(path: &Path) -> PathBuf {
    let rendered = path.to_string_lossy();
    if let Some(path) = rendered.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{path}"));
    }
    rendered
        .strip_prefix(r"\\?\")
        .map_or_else(|| path.to_path_buf(), PathBuf::from)
}

#[cfg(not(windows))]
fn cargo_compatible_path(path: &Path) -> PathBuf {
    path.to_path_buf()
}

pub(crate) fn package_name_from_manifest(path: &Path) -> Result<String, String> {
    let manifest = parse_package_manifest(path)?;
    Ok(manifest.name)
}

pub(crate) fn parse_package_manifest(path: &Path) -> Result<Road1Package, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("read crate manifest {}: {e}", path.display()))?;
    let value: toml::Value = toml::from_str(&text)
        .map_err(|e| format!("parse crate manifest {}: {e}", path.display()))?;
    let name = value
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(|name| name.as_str())
        .ok_or_else(|| format!("crate manifest {} missing package.name", path.display()))?
        .to_owned();

    Ok(Road1Package {
        name,
        dependencies: Vec::new(),
        manifest_path: normalize_path(path),
    })
}

pub(crate) fn discover_road1_packages(
    root: &Path,
    subworkspaces: &[SubworkspaceConfig],
) -> Result<Vec<Road1Package>, String> {
    let mut packages = BTreeMap::<String, Road1Package>::new();

    for subworkspace in subworkspaces {
        let manifest_path = root.join(&subworkspace.path).join("Cargo.toml");
        let workspace_root = root.join(&subworkspace.path);
        let workspace_manifest = parse_workspace_manifest(&manifest_path)?;
        let has_members = workspace_manifest
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.members.as_ref())
            .map(|members| !members.is_empty())
            .unwrap_or(false);
        if !has_members {
            continue;
        }
        let metadata = cargo_metadata(root, &manifest_path)?;
        for package in &metadata.packages {
            if !is_workspace_package(&workspace_root, package) {
                continue;
            }
            let dependencies = package
                .dependencies
                .iter()
                .map(|dependency| dependency.name.clone())
                .collect::<Vec<_>>();

            packages.insert(
                package.name.clone(),
                Road1Package {
                    name: package.name.clone(),
                    dependencies,
                    manifest_path: package.manifest_path.clone(),
                },
            );
        }
    }

    Ok(packages.into_values().collect())
}

/// Extra workspace-owned rows whose exact dependency direction is governed by
/// the committed DAG snapshot without applying Road 1 naming rules to them.
pub(crate) fn discover_snapshot_dependency_packages(
    root: &Path,
    groups: &[SnapshotDependencyPackagesConfig],
) -> Result<Vec<Road1Package>, String> {
    let mut selected = BTreeMap::<String, Road1Package>::new();
    for group in groups {
        let manifest = root.join(&group.workspace_manifest);
        let workspace_root = manifest
            .parent()
            .ok_or_else(|| format!("snapshot manifest has no parent: {}", manifest.display()))?;
        let expected = group.packages.iter().cloned().collect::<BTreeSet<_>>();
        if expected.is_empty() || expected.len() != group.packages.len() {
            return Err(format!(
                "snapshot package list for {} is empty or duplicated",
                group.workspace_manifest
            ));
        }
        let metadata = cargo_metadata(root, &manifest)?;
        let mut found = BTreeSet::new();
        for package in metadata.packages {
            if expected.contains(&package.name) && is_workspace_package(workspace_root, &package) {
                let name = package.name.clone();
                let dependencies = package
                    .dependencies
                    .into_iter()
                    .map(|dependency| dependency.name)
                    .collect();
                found.insert(name.clone());
                if selected
                    .insert(
                        name.clone(),
                        Road1Package {
                            name: package.name,
                            dependencies,
                            manifest_path: package.manifest_path,
                        },
                    )
                    .is_some()
                {
                    return Err(format!("duplicate snapshot package {name}"));
                }
            }
        }
        if let Some(name) = expected.difference(&found).next() {
            return Err(format!(
                "snapshot package {name} is absent from {}",
                group.workspace_manifest
            ));
        }
        validate_required_snapshot_edges(group, &selected, &found)?;
    }
    Ok(selected.into_values().collect())
}

fn validate_required_snapshot_edges(
    group: &SnapshotDependencyPackagesConfig,
    selected: &BTreeMap<String, Road1Package>,
    found: &BTreeSet<String>,
) -> Result<(), String> {
    for edge in &group.required_edges {
        let Some(source) = selected.get(&edge.source) else {
            return Err(format!(
                "required snapshot edge has no source {}",
                edge.source
            ));
        };
        if !found.contains(&edge.source) || !source.dependencies.contains(&edge.target) {
            return Err(format!(
                "required snapshot edge {} -> {} is absent from {}",
                edge.source, edge.target, group.workspace_manifest
            ));
        }
    }
    Ok(())
}

pub(crate) fn discover_query_audience_packages(
    root: &Path,
    contract: &QueryAudienceContract,
) -> Result<Vec<Road1Package>, String> {
    let mut package_names = contract
        .audiences
        .iter()
        .map(|audience| audience.package.as_str())
        .collect::<Vec<_>>();
    if let Some(certification_package) = contract.certification_package.as_deref() {
        package_names.push(certification_package);
    }
    package_names
        .into_iter()
        .map(|package_name| {
            let manifest_path = root
                .join(&contract.workspace)
                .join("crates")
                .join(package_name)
                .join("Cargo.toml");
            let canonical_manifest_path = fs::canonicalize(&manifest_path).map_err(|error| {
                format!(
                    "canonicalize configured audience manifest {}: {error}",
                    manifest_path.display()
                )
            })?;
            let metadata = cargo_metadata(root, &manifest_path)?;
            let package = metadata
                .packages
                .iter()
                .find(|package| {
                    cargo_compatible_path(Path::new(&package.manifest_path))
                        == cargo_compatible_path(&canonical_manifest_path)
                })
                .ok_or_else(|| {
                    format!(
                        "cargo metadata omitted configured audience {}",
                        package_name
                    )
                })?;
            let dependencies = package
                .dependencies
                .iter()
                .map(|dependency| dependency.name.clone())
                .collect();
            Ok(Road1Package {
                name: package.name.clone(),
                dependencies,
                manifest_path: package.manifest_path.clone(),
            })
        })
        .collect()
}

fn is_workspace_package(workspace_root: &Path, package: &CargoMetadataPackage) -> bool {
    cargo_compatible_path(Path::new(&package.manifest_path))
        .starts_with(cargo_compatible_path(workspace_root))
}

pub(crate) fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub(crate) fn normalize_str(path: &str) -> String {
    path.replace('\\', "/")
}

#[cfg(test)]
mod snapshot_dependency_tests {
    use super::*;
    use crate::config::SnapshotDependencyEdgeConfig;

    #[test]
    fn required_store_edge_cannot_be_removed_even_with_a_new_snapshot() {
        let group = SnapshotDependencyPackagesConfig {
            workspace_manifest: "workspaces/worth-store/Cargo.toml".into(),
            packages: vec!["worth-store".into()],
            required_edges: vec![SnapshotDependencyEdgeConfig {
                source: "worth-store".into(),
                target: "worth-store-blob-chunks".into(),
            }],
        };
        let found = BTreeSet::from(["worth-store".into()]);
        let selected = BTreeMap::from([(
            "worth-store".into(),
            Road1Package {
                name: "worth-store".into(),
                dependencies: vec![],
                manifest_path: "unused/Cargo.toml".into(),
            },
        )]);
        assert!(validate_required_snapshot_edges(&group, &selected, &found).is_err());
    }
}
