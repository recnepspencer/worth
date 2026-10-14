//! Exact consumer document inventory and Cargo doctest selections.
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Component, Path};

pub(super) const CONFIG: &str = "tools/boundary-check/config/consumer-docs.toml";
pub(super) const DEBT: &str = "tools/boundary-check/snapshots/consumer-doc-fence-debt.toml";
pub(super) const SOURCE: &str = "workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Configuration {
    pub documents: Vec<String>,
    pub doctests: Vec<DoctestRoot>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DoctestRoot {
    pub manifest: String,
    pub packages: Vec<String>,
}

pub(super) fn confined_path(value: &str) -> bool {
    !value.is_empty()
        && !value.contains('\\')
        && Path::new(value)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

pub(super) fn load(root: &Path) -> Result<Configuration, String> {
    let text = crate::source_documents::read_to_string(&root.join(CONFIG))
        .map_err(|error| format!("{CONFIG}: {error}"))?;
    let config: Configuration = toml::from_str(&text).map_err(|error| error.to_string())?;
    let mut documents = BTreeSet::new();
    for path in &config.documents {
        if !confined_path(path) || !documents.insert(path) || !root.join(path).is_file() {
            return Err(format!(
                "invalid, duplicate or missing consumer document: {path}"
            ));
        }
    }
    let mut packages = BTreeSet::new();
    let mut manifests = BTreeSet::new();
    for selection in &config.doctests {
        if !confined_path(&selection.manifest)
            || !root.join(&selection.manifest).is_file()
            || !manifests.insert(&selection.manifest)
            || selection.packages.is_empty()
        {
            return Err(format!("invalid doctest root: {}", selection.manifest));
        }
        for package in &selection.packages {
            if package.is_empty()
                || !package
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || "_-".contains(character))
                || !packages.insert(package)
            {
                return Err(format!("duplicate doctest package: {package}"));
            }
        }
    }
    Ok(config)
}
