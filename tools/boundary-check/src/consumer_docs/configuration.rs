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
    #[serde(default)]
    pub compiled_sources: Vec<CompiledSource>,
    #[serde(default)]
    pub facts: Vec<FactRegion>,
    #[serde(default)]
    pub absence_claims: Vec<AbsenceClaim>,
    #[serde(default)]
    pub navigation: Option<Navigation>,
    #[serde(default)]
    pub derive_skills: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FactRegion {
    pub id: String,
    pub manifest: String,
    pub item: String,
    pub document: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AbsenceClaim {
    pub id: String,
    pub manifest: String,
    pub container: String,
    pub name: String,
    pub kind: crate::facade_docs::resolution::SelectorKind,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Navigation {
    pub index: String,
    pub guides: Vec<String>,
    pub crate_roots: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DoctestRoot {
    pub manifest: String,
    pub packages: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CompiledSource {
    pub name: String,
    pub crate_root: String,
    pub target: String,
    pub source: String,
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
        if !confined_path(path) || !documents.insert(path.clone()) || !root.join(path).is_file() {
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
    let mut names = BTreeSet::new();
    for binding in &config.compiled_sources {
        if ![
            &binding.name,
            &binding.crate_root,
            &binding.target,
            &binding.source,
        ]
        .into_iter()
        .all(|path| confined_path(path))
            || !names.insert(&binding.name)
        {
            return Err("invalid or duplicate compiled source binding".into());
        }
        let owner = root.join(&binding.crate_root);
        let manifest: toml::Value = toml::from_str(
            &crate::source_documents::read_to_string(&owner.join("Cargo.toml"))
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        let registered = binding.target == "src/main.rs"
            && manifest["package"]
                .get("autobins")
                .and_then(toml::Value::as_bool)
                != Some(false)
            || ["bin", "example"].into_iter().any(|kind| {
                manifest
                    .get(kind)
                    .and_then(toml::Value::as_array)
                    .is_some_and(|targets| {
                        targets
                            .iter()
                            .any(|target| target["path"].as_str() == Some(&binding.target))
                    })
            });
        if !registered
            || !owner.join(&binding.target).is_file()
            || !owner.join(&binding.source).is_file()
        {
            return Err(format!(
                "compiled source target is not registered: {}",
                binding.name
            ));
        }
    }
    validate_guidance(root, &config, &documents)?;
    Ok(config)
}

pub(super) fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

pub(super) fn confined_existing(root: &Path, name: &str) -> Result<std::path::PathBuf, String> {
    if !confined_path(name) {
        return Err("path must be repository-relative".into());
    }
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let path = root.join(name).canonicalize().map_err(|e| e.to_string())?;
    if !path.starts_with(root) {
        return Err("path escapes repository".into());
    }
    Ok(path)
}

fn validate_guidance(
    root: &Path,
    config: &Configuration,
    documents: &BTreeSet<String>,
) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    for fact in &config.facts {
        if !valid_id(&fact.id)
            || !ids.insert(&fact.id)
            || !documents.contains(&fact.document)
            || !document_output(&fact.document)
            || !confined_path(&fact.manifest)
            || !fact.manifest.ends_with("Cargo.toml")
        {
            return Err(format!("invalid fact binding: {}", fact.id));
        }
        super::guidance_write::inspect(root, &root.join(&fact.document), false)?;
        confined_existing(root, &fact.manifest)?;
    }
    let mut claims = BTreeSet::new();
    for claim in &config.absence_claims {
        if !valid_id(&claim.id) || !claims.insert(&claim.id) || !confined_path(&claim.manifest) {
            return Err(format!("invalid absence binding: {}", claim.id));
        }
        confined_existing(root, &claim.manifest)?;
    }
    if let Some(nav) = &config.navigation {
        if !documents.contains(&nav.index)
            || nav.guides.iter().any(|p| !documents.contains(p))
            || nav
                .crate_roots
                .iter()
                .any(|p| !confined_path(p) || !root.join(p).is_file())
        {
            return Err("invalid guidance navigation paths".into());
        }
    }
    Ok(())
}

// Markdown consumer guidance only. Configuration membership still binds each target;
// this class excludes snapshots, Rust and authoritative/generated skill sources.
pub(super) fn document_output(name: &str) -> bool {
    confined_path(name)
        && name.ends_with(".md")
        && (name.starts_with("docs/")
            || name.starts_with("workspaces/worth-query/crates/worth-query/docs/")
            || name == "workspaces/worth-query/README.md"
            || name == "workspaces/worth-query/crates/worth-query-host/README.md")
}
