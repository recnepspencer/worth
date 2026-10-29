//! Testkits and courts cannot become normal product authority providers.
use super::failure;
use crate::{diagnostics::Diagnostic, source_rules::GovernedCrate};

pub(super) fn validate(
    owner: &GovernedCrate,
    manifest: &toml::Value,
    stems: &[&str],
    resolved: Option<&[String]>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let is_test = |package: &str| {
        stems.iter().any(|stem| {
            package.starts_with(&format!("{stem}-testkit"))
                || package.starts_with(&format!("{stem}-court-"))
        })
    };
    let refused = |package: &str| {
        failure(&owner.relative_crate_root, format!("product dependency `{package}` must be in dev-dependencies; testkits are test-only and courts are leaves"))
    };
    if let Some(packages) = resolved {
        return packages
            .iter()
            .filter(|package| is_test(package))
            .map(|package| refused(package))
            .collect();
    }
    let mut tables = vec![manifest];
    if let Some(targets) = manifest.get("target").and_then(|v| v.as_table()) {
        tables.extend(targets.values());
    }
    for table in tables {
        for kind in ["dependencies", "build-dependencies"] {
            if let Some(dependencies) = table.get(kind).and_then(|v| v.as_table()) {
                for (alias, declaration) in dependencies {
                    let package = declaration
                        .get("package")
                        .and_then(|v| v.as_str())
                        .unwrap_or(alias);
                    if is_test(package) {
                        diagnostics.push(refused(package));
                    }
                }
            }
        }
    }
    diagnostics
}
