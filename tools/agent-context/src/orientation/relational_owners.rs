//! Orientation for the workspace packages in the Relational owner map.
use super::{model::CrateOrientation, source_surface::collect_owned_modules};
use serde::Deserialize;
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Deserialize)]
struct OwnerMap {
    owners: BTreeMap<String, Owner>,
}

#[derive(Deserialize)]
struct Owner {
    #[serde(rename = "crate")]
    package: String,
    #[serde(default)]
    physical: bool,
    layer: u8,
    depends: Vec<String>,
}

pub(super) fn orientations(
    root: &Path,
    machine_constitution: &str,
) -> Result<Vec<CrateOrientation>, String> {
    let map = root.join("scripts/ci/relational_dag_owners.toml");
    let source = fs::read_to_string(&map).map_err(|error| format!("{}: {error}", map.display()))?;
    let map: OwnerMap = toml::from_str(&source).map_err(|error| error.to_string())?;
    map.owners
        .iter()
        .filter(|(_, owner)| owner.physical)
        .map(|(domain, owner)| {
            let relative_path = format!("crates/{}", owner.package);
            let allowed_target_bands = owner
                .depends
                .iter()
                .map(|dependency| {
                    map.owners
                        .get(dependency)
                        .map(|owner| owner.package.clone())
                        .ok_or_else(|| format!("{}: unknown owner {dependency}", owner.package))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CrateOrientation {
                crate_name: owner.package.clone(),
                owned_modules: collect_owned_modules(&root.join(&relative_path).join("src"))?,
                relative_path: relative_path.clone(),
                constitutional_class: format!("worth/relational layer {}", owner.layer),
                domain: domain.clone(),
                exemplar_role: format!("Owns the physical Relational {domain} layer."),
                deferred_routes: Vec::new(),
                public_surface: "Consumer code uses worth-relational::facade. The internal_port serves Relational owners.".to_owned(),
                allowed_target_bands,
                facade_exports: Vec::new(),
                machine_fences: vec![
                    "Relational dependencies must follow the Relational owner map.".to_owned(),
                    "Workspace dependency denials seal the package to admitted Relational callers.".to_owned(),
                ],
                skeleton_fence: "The Relational DAG audit checks physical ownership and source placement.".to_owned(),
                machine_constitution: machine_constitution.to_owned(),
            })
        })
        .collect()
}
