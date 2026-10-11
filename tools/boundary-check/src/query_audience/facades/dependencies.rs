use super::relative_to_root;
use crate::config::QueryAudienceContract;
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::path::Path;

pub(super) fn validate_cold_certification_leaf(
    root: &Path,
    query_crates_root: &Path,
    certification_package: &str,
    contract: &QueryAudienceContract,
) -> Result<Vec<Diagnostic>, String> {
    let certification_root = query_crates_root.join(certification_package);
    let certification_relative = relative_to_root(root, &certification_root);
    let mut diagnostics = Vec::new();

    if !certification_root.is_dir() {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc3003QueryAudienceFacadeContract,
            certification_relative,
            format!(
                "configured Query certification package `{certification_package}` is missing from the Query workspace"
            ),
        ));
        return Ok(diagnostics);
    }

    let certification_manifest = certification_root.join("Cargo.toml");
    let certification_text = crate::source_documents::read_to_string(&certification_manifest)
        .map_err(|e| format!("read {}: {e}", certification_manifest.display()))?;
    let certification_value: toml::Value = toml::from_str(&certification_text)
        .map_err(|e| format!("parse {}: {e}", certification_manifest.display()))?;
    let certification_dependencies = direct_dependency_package_names(&certification_value);
    let mut expected_dependencies = contract.certification_authority_packages.clone();
    expected_dependencies.sort();
    if certification_dependencies != expected_dependencies {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc3003QueryAudienceFacadeContract,
            format!("{certification_relative}/Cargo.toml"),
            format!(
                "Query certification must depend on exactly its configured authority facades {}; found normal dependencies: {}",
                expected_dependencies.join(", "),
                if certification_dependencies.is_empty() {
                    "none".to_owned()
                } else {
                    certification_dependencies.join(", ")
                }
            ),
        ));
    }

    reject_ordinary_certification_dependencies(
        root,
        query_crates_root,
        certification_package,
        contract,
        &mut diagnostics,
    )?;
    Ok(diagnostics)
}

fn reject_ordinary_certification_dependencies(
    root: &Path,
    query_crates_root: &Path,
    certification_package: &str,
    contract: &QueryAudienceContract,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), String> {
    let mut ordinary_packages = vec![contract.engine_package.as_str()];
    ordinary_packages.extend(
        contract
            .audiences
            .iter()
            .map(|audience| audience.package.as_str()),
    );
    ordinary_packages.extend(contract.internal_packages.iter().map(String::as_str));
    for ordinary_package in ordinary_packages {
        let manifest_path = query_crates_root.join(ordinary_package).join("Cargo.toml");
        let text = crate::source_documents::read_to_string(&manifest_path)
            .map_err(|e| format!("read {}: {e}", manifest_path.display()))?;
        let value: toml::Value =
            toml::from_str(&text).map_err(|e| format!("parse {}: {e}", manifest_path.display()))?;
        if all_direct_dependency_package_names(&value)
            .iter()
            .any(|dependency| dependency == certification_package)
        {
            diagnostics.push(Diagnostic::new(
                DiagnosticCode::Bc3003QueryAudienceFacadeContract,
                relative_to_root(root, &manifest_path),
                format!(
                    "ordinary Query package `{ordinary_package}` must not depend on cold certification `{certification_package}`"
                ),
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_authority_dependencies(
    crate_root: &Path,
    relative: &str,
    authority_packages: &[String],
    audience_packages: &[&str],
) -> Result<Vec<Diagnostic>, String> {
    let manifest_path = crate_root.join("Cargo.toml");
    let text = crate::source_documents::read_to_string(&manifest_path)
        .map_err(|e| format!("read {}: {e}", manifest_path.display()))?;
    let value: toml::Value =
        toml::from_str(&text).map_err(|e| format!("parse {}: {e}", manifest_path.display()))?;
    let deps = direct_dependency_package_names(&value);
    let mut diagnostics = Vec::new();

    let mut expected = authority_packages.to_vec();
    expected.sort();
    if deps != expected {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc3003QueryAudienceFacadeContract,
            format!("{relative}/Cargo.toml"),
            format!(
                "Query audience facade must depend on exactly {}; found dependencies: {}",
                expected.join(", "),
                if deps.is_empty() {
                    "none".to_owned()
                } else {
                    deps.join(", ")
                }
            ),
        ));
    }

    for dep in &deps {
        if audience_packages.iter().any(|package| package == dep) {
            diagnostics.push(Diagnostic::new(
                DiagnosticCode::Bc3003QueryAudienceFacadeContract,
                format!("{relative}/Cargo.toml"),
                format!("Query audience facade must not depend on another audience facade `{dep}`"),
            ));
        }
    }

    Ok(diagnostics)
}

fn direct_dependency_package_names(manifest: &toml::Value) -> Vec<String> {
    let Some(deps) = manifest
        .get("dependencies")
        .and_then(|value| value.as_table())
    else {
        return Vec::new();
    };
    let mut names = deps
        .iter()
        .map(|(key, spec)| {
            if let Some(table) = spec.as_table() {
                table
                    .get("package")
                    .and_then(|package| package.as_str())
                    .unwrap_or(key)
                    .to_owned()
            } else {
                key.clone()
            }
        })
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
}

fn all_direct_dependency_package_names(manifest: &toml::Value) -> Vec<String> {
    let mut names = Vec::new();
    for table_name in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let Some(dependencies) = manifest.get(table_name).and_then(toml::Value::as_table) else {
            continue;
        };
        names.extend(dependencies.iter().map(|(key, spec)| {
            spec.as_table()
                .and_then(|table| table.get("package"))
                .and_then(toml::Value::as_str)
                .unwrap_or(key)
                .to_owned()
        }));
    }
    names.sort();
    names.dedup();
    names
}
