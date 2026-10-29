//! Testing law 18: product/unit boundaries and single-class integration courts.
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::{collections::BTreeSet, path::Path};

mod configuration;
mod cost_classes;
mod court_manifest;
mod production_dependencies;
mod targets;
mod workspace_members;

const CONFIG: &str = "tools/boundary-check/config/test-workspaces.txt";
const EXCEPTIONS: &str = "tools/boundary-check/config/test-binary-exceptions.txt";

pub(crate) fn validate(root: &Path) -> Vec<Diagnostic> {
    match validate_configured(root) {
        Ok(diagnostics) => diagnostics,
        Err(error) => vec![failure(CONFIG, error)],
    }
}

fn validate_configured(root: &Path) -> Result<Vec<Diagnostic>, String> {
    let resolved = std::fs::canonicalize(root).map_err(|e| e.to_string())?;
    let root = resolved.as_path();
    let Some(config) = configuration::load(root)? else {
        return Ok(Vec::new());
    };
    let mut diagnostics = Vec::new();
    let mut seen_exceptions = BTreeSet::new();
    let mut manifests = std::collections::BTreeMap::new();
    let stems: Vec<_> = config
        .workspaces
        .iter()
        .map(|workspace| workspace.stem.as_str())
        .collect();
    for workspace in &config.workspaces {
        for member in workspace_members::manifests(&root.join(&workspace.path))? {
            manifests.insert(
                member.manifest,
                (workspace.stem.clone(), member.production_dependencies),
            );
        }
    }
    for (manifest, (stem, production_dependencies)) in manifests {
        let text = crate::source_documents::read_to_string(&manifest)
            .map_err(|error| error.to_string())?;
        let value: toml::Value = toml::from_str(&text).map_err(|error| error.to_string())?;
        let Some(name) = value
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
        else {
            continue;
        };
        let relative = manifest
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let owner = crate::source_rules::GovernedCrate {
            package: name.into(),
            crate_root: manifest.parent().unwrap().into(),
            relative_crate_root: relative.trim_end_matches("/Cargo.toml").into(),
        };
        let court = value
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("worth"))
            .and_then(|w| w.get("court"))
            .is_some()
            || name.contains("-court-");
        if !court {
            if owner.crate_root.join("tests").exists() {
                diagnostics.push(failure(&relative, "product crate must not contain a tests/ directory; integration tests belong in courts"));
            }
            let worlds = value
                .get("package")
                .and_then(|p| p.get("metadata"))
                .and_then(|m| m.get("worth"))
                .and_then(|w| w.get("worlds"))
                .is_some()
                && name.starts_with(&format!("{stem}-worlds"));
            if !worlds {
                diagnostics.extend(production_dependencies::validate(
                    &owner,
                    &value,
                    &stems,
                    production_dependencies.as_deref(),
                ));
            }
            continue;
        }
        diagnostics.extend(court_manifest::product_code(&owner, &value));
        let class = match court_manifest::class(&owner, &value, &stem) {
            Ok(class) => class,
            Err(error) => {
                diagnostics.push(error);
                continue;
            }
        };
        let targets = targets::discover(&owner.crate_root, &value)?;
        let mut ordinary = Vec::new();
        for target in &targets {
            let key = (relative.clone(), target.name.clone());
            if config.exceptions.contains(&key) {
                seen_exceptions.insert(key);
                if !target.explicit {
                    diagnostics.push(failure(
                        &relative,
                        format!("exception `{}` requires an explicit [[test]]", target.name),
                    ));
                }
            } else {
                ordinary.push(target);
                diagnostics.extend(cost_classes::validate(&owner, &target.path, &class));
            }
        }
        let canonical = name.replace('-', "_");
        if ordinary.len() != 1 {
            diagnostics.push(failure(
                &relative,
                "court needs one canonical integration target; a second integration-test target requires a declared executable exception",
            ));
        }
        for target in ordinary {
            if target.name != canonical
                || target.path != owner.crate_root.join(format!("tests/{canonical}.rs"))
            {
                diagnostics.push(failure(
                    &relative,
                    format!("ordinary integration target must be tests/{canonical}.rs"),
                ));
            }
            if !target.harness {
                diagnostics.push(failure(
                    &relative,
                    format!(
                        "harness=false target `{}` requires a declared executable exception",
                        target.name
                    ),
                ));
            }
        }
    }
    for (manifest, target) in config.exceptions.difference(&seen_exceptions) {
        diagnostics.push(failure(
            EXCEPTIONS,
            format!("stale or ungoverned exception: {manifest} | {target}"),
        ));
    }
    Ok(diagnostics)
}

fn failure(path: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Bc9002TestTopology, path, message)
}

#[cfg(test)]
mod tests;
