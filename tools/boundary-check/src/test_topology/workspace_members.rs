//! Cargo owns workspace membership, including implicit local dependency members.
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

pub(super) struct Member {
    pub(super) manifest: PathBuf,
    pub(super) production_dependencies: Option<Vec<String>>,
    pub(super) all_dependencies: Vec<String>,
}

pub(super) fn manifests(workspace: &Path) -> Result<Vec<Member>, String> {
    let path = workspace.join("Cargo.toml");
    let text = crate::source_documents::read_to_string(&path).map_err(|e| e.to_string())?;
    let manifest: toml::Value = toml::from_str(&text).map_err(|e| e.to_string())?;
    if manifest.get("workspace").is_some() {
        let metadata = crate::cargo_graph::cargo_metadata(workspace, &path)?;
        let members: BTreeSet<_> = metadata.workspace_members.into_iter().collect();
        return metadata
            .packages
            .into_iter()
            .filter(|package| members.contains(&package.id))
            .map(|package| {
                let manifest =
                    std::fs::canonicalize(package.manifest_path).map_err(|e| e.to_string())?;
                let production_dependencies = Some(
                    package
                        .dependencies
                        .iter()
                        .filter(|dependency| dependency.kind.as_deref() != Some("dev"))
                        .map(|dependency| dependency.name.clone())
                        .collect(),
                );
                Ok(Member {
                    manifest,
                    production_dependencies,
                    all_dependencies: package.dependencies.into_iter().map(|d| d.name).collect(),
                })
            })
            .collect();
    }
    Ok(if manifest.get("package").is_some() {
        vec![Member {
            manifest: path,
            production_dependencies: None,
            all_dependencies: Vec::new(),
        }]
    } else {
        Vec::new()
    })
}
