//! Iteration laws: leaf courts, narrow testkits and deterministic topology.
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

mod category_paths;
mod configuration;
mod construction_factories;
mod court_manifest;
mod diff_scope;
mod production_dependencies;
mod runtime_roots;
mod selectors;
mod source_baseline;
mod source_construction;
mod structure_limits;
mod targets;
mod workspace_members;

const CONFIG: &str = "tools/boundary-check/config/test-workspaces.txt";
const EXCEPTIONS: &str = "tools/boundary-check/config/test-binary-exceptions.txt";

pub(crate) fn validate(root: &Path) -> Vec<Diagnostic> {
    validate_configured(root, None).unwrap_or_else(|error| vec![failure(CONFIG, error)])
}

pub(crate) fn validate_diff(root: &Path) -> Vec<Diagnostic> {
    let result = (|| {
        let root = std::fs::canonicalize(root).map_err(|e| e.to_string())?;
        let paths = diff_scope::changed(&root)?;
        validate_configured(&root, paths.as_ref())
    })();
    result.unwrap_or_else(|error| vec![failure(CONFIG, error)])
}

fn owners(
    root: &Path,
) -> Result<BTreeMap<std::path::PathBuf, (String, workspace_members::Member)>, String> {
    let mut manifests = BTreeMap::new();
    if let Some(config) = configuration::load(root)? {
        for workspace in config.workspaces {
            for member in workspace_members::manifests(&root.join(&workspace.path))? {
                manifests.insert(member.manifest.clone(), (workspace.stem.clone(), member));
            }
        }
    }
    Ok(manifests)
}

pub(crate) fn source_report(root: &Path) -> Result<serde_json::Value, String> {
    let root = std::fs::canonicalize(root).map_err(|e| e.to_string())?;
    let roots = runtime_roots::Registry::load(&root)?;
    let mut observations = BTreeSet::new();
    for (manifest, (_, _)) in owners(&root)? {
        let value = manifest_value(&manifest)?;
        let owner = owner(&root, &manifest, &value)?;
        if !owner.package.contains("-court-") && !owner.package.contains("-testkit") {
            observations.extend(source_baseline::observed(&root, &owner, &roots)?);
        }
    }
    serde_json::to_value(source_baseline::grouped(observations)).map_err(|e| e.to_string())
}

fn manifest_value(path: &Path) -> Result<toml::Value, String> {
    let text = crate::source_documents::read_to_string(path).map_err(|e| e.to_string())?;
    toml::from_str(&text).map_err(|e| e.to_string())
}
fn owner(
    root: &Path,
    manifest: &Path,
    value: &toml::Value,
) -> Result<crate::source_rules::GovernedCrate, String> {
    let name = value
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or("member has no package name")?;
    Ok(crate::source_rules::GovernedCrate {
        package: name.into(),
        crate_root: manifest.parent().unwrap().into(),
        relative_crate_root: manifest
            .parent()
            .unwrap()
            .strip_prefix(root)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/"),
    })
}
fn validate_configured(
    root: &Path,
    changed: Option<&BTreeSet<std::path::PathBuf>>,
) -> Result<Vec<Diagnostic>, String> {
    let root = std::fs::canonicalize(root).map_err(|e| e.to_string())?;
    let Some(config) = configuration::load(&root)? else {
        return Ok(Vec::new());
    };
    let manifests = owners(&root)?;
    let stems: Vec<_> = config.workspaces.iter().map(|w| w.stem.as_str()).collect();
    let mut diagnostics = selectors::validate(&root);
    let mut source_debt: Option<(source_baseline::Baseline, runtime_roots::Registry)> = None;
    for (manifest, (stem, member)) in &manifests {
        let value = manifest_value(manifest)?;
        let owner = owner(&root, manifest, &value)?;
        if !diff_scope::includes(changed, &owner.crate_root) {
            continue;
        }
        let relative = format!("{}/Cargo.toml", owner.relative_crate_root);
        let worth = value
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("worth"));
        let court =
            owner.package.contains("-court-") || worth.and_then(|w| w.get("court")).is_some();
        let testkit =
            owner.package.contains("-testkit") || worth.and_then(|w| w.get("testkit")).is_some();
        if court {
            diagnostics.extend(court_manifest::product_code(&owner, &value));
            if let Err(error) = court_manifest::surface(&owner, &value, stem) {
                diagnostics.push(error);
            }
            diagnostics.extend(structure_limits::validate(&owner));
            let targets = targets::discover(&owner.crate_root, &value)?;
            let canonical = owner.package.replace('-', "_");
            if targets.len() != 1 {
                diagnostics.push(failure(&relative, "court must have exactly one integration-test target; executable exceptions are retired by convention v3"));
            }
            for target in targets {
                if !target.tested || !target.harness {
                    diagnostics.push(failure(
                        &relative,
                        "court target must execute tests with the standard harness",
                    ));
                }
                if !target.explicit
                    || target.name != canonical
                    || target.path != owner.crate_root.join(format!("tests/{canonical}.rs"))
                {
                    diagnostics.push(failure(
                        &relative,
                        format!(
                            "canonical explicit integration target must be tests/{canonical}.rs"
                        ),
                    ));
                }
                diagnostics.extend(category_paths::validate(&owner, &target.path));
            }
        } else {
            if owner.crate_root.join("tests").exists() {
                diagnostics.push(failure(
                    &relative,
                    "product crate must not contain a tests/ directory",
                ));
            }
            if value.get("test").is_some() {
                diagnostics.push(failure(
                    &relative,
                    "product crate cannot declare integration targets",
                ));
            }
            if testkit {
                let name = &owner.package;
                if !(name == &format!("{stem}-testkit")
                    || name
                        .strip_prefix(&format!("{stem}-testkit-"))
                        .is_some_and(|s| {
                            !s.is_empty()
                                && s.bytes().all(|b| {
                                    b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'
                                })
                        }))
                {
                    diagnostics.push(failure(&relative, "misnamed testkit"));
                }
                diagnostics.extend(structure_limits::validate(&owner));
            } else {
                diagnostics.extend(production_dependencies::validate(
                    &owner,
                    &value,
                    &stems,
                    member.production_dependencies.as_deref(),
                ));
                if source_debt.is_none() {
                    source_debt = Some((
                        source_baseline::load(&root)?,
                        runtime_roots::Registry::load(&root)?,
                    ));
                }
                diagnostics.extend(source_baseline::validate(
                    &root,
                    &owner,
                    &source_debt.as_ref().unwrap().0,
                    &source_debt.as_ref().unwrap().1,
                ));
            }
        }
        // Even dev dependencies would invalidate the leaf property.
        for dependency in &member.all_dependencies {
            if stems
                .iter()
                .any(|s| dependency.starts_with(&format!("{s}-court-")))
            {
                diagnostics.push(failure(
                    &relative,
                    format!("court `{dependency}` must be a leaf; no package may depend on it"),
                ));
            }
        }
    }
    if !config.exceptions.is_empty() {
        diagnostics.push(failure(
            EXCEPTIONS,
            "convention v3 permits exactly one target, with no executable exceptions",
        ));
    }
    Ok(diagnostics)
}

fn failure(path: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Bc9002TestTopology, path, message)
}

#[cfg(test)]
mod tests;
