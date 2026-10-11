//! Query and its runtime owners order declared identity, never Rust build identity.
mod address_order;
mod expression_identity;
mod inventory;
#[cfg(test)]
mod tests;

use super::crate_modules::{GovernedCrate, ModuleGraph};
use crate::diagnostics::{Diagnostic, DiagnosticCode};

const CRATES: &[&str] = &[
    "worth-query-execution",
    "worth-query-installation",
    "worth-runtime-world",
    "worth-execution",
    "worth-runtime-bridge",
];
const ADDRESS_CRATES: &[&str] = &[
    "worth-query-execution",
    "worth-runtime-world",
    "worth-execution",
    "worth-runtime-bridge",
    "worth-signal",
    "worth-relational",
];

pub(super) fn validate(root: &std::path::Path) -> Result<Vec<Diagnostic>, String> {
    let mut diagnostics = Vec::new();
    for workspace in ["", "workspaces/worth-query"] {
        for governed in super::workspace_crates::discover_workspace_crates(root, workspace)? {
            if in_scope(&governed.package) {
                let graph = super::crate_modules::parse_crate_modules(&governed)?;
                diagnostics.extend(enforce(&governed, &graph));
            }
        }
    }
    Ok(diagnostics)
}

fn enforce(governed: &GovernedCrate, graph: &ModuleGraph) -> Vec<Diagnostic> {
    if !in_scope(&governed.package) {
        return Vec::new();
    }
    let files: Vec<_> = graph
        .modules
        .values()
        .map(|module| (module.relative_source.as_str(), module.items.as_slice()))
        .collect();
    let address_files: Vec<_> = graph
        .modules
        .iter()
        .filter(|(path, _)| !super::authority_import_explicitness::module_is_test_only(graph, path))
        .map(|(_, module)| (module.relative_source.as_str(), module.items.as_slice()))
        .collect();
    collect_findings(&files, &address_files)
        .into_iter()
        .filter(|finding| {
            if address_order::is_finding(finding.reason) {
                ADDRESS_CRATES.contains(&governed.package.as_str())
            } else {
                CRATES.contains(&governed.package.as_str())
            }
        })
        .map(|finding| {
            Diagnostic::new(
                DiagnosticCode::Bc7009CanonicalOrder,
                format!(
                    "{}/{}:{}",
                    governed.relative_crate_root, finding.source, finding.line
                ),
                format!(
                    "{}: {}; order must compare declared identity",
                    finding.item, finding.reason
                ),
            )
        })
        .collect()
}

fn in_scope(package: &str) -> bool {
    CRATES.contains(&package) || ADDRESS_CRATES.contains(&package)
}

fn collect_findings(
    files: &[(&str, &[syn::Item])],
    address_files: &[(&str, &[syn::Item])],
) -> Vec<inventory::Finding> {
    let mut found = inventory::scan(files);
    found.extend(address_order::scan(address_files));
    found
}

#[cfg(test)]
fn findings(files: &[(&str, &[syn::Item])]) -> Vec<inventory::Finding> {
    collect_findings(files, files)
}
