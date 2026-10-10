//! Query Execution orders declared identity, never Rust build identity.
mod inventory;
#[cfg(test)]
mod tests;

use super::crate_modules::{GovernedCrate, ModuleGraph};
use crate::diagnostics::{Diagnostic, DiagnosticCode};

const CRATE: &str = "worth-query-execution";

/// Named exceptions are membership contracts, never permission to iterate for
/// output, selection, or cost. Every exception names its membership consumer.
const MEMBERSHIP_ONLY: &[(&str, &str, &str)] = &[
    (
        "src/domain_computation/primary_graph/output_lineage/family_selection/checkpoint_priors.rs",
        "checkpoint_prior_outputs",
        "membership only: installed bindings are queried by equality to validate checkpoint roles",
    ),
    (
        "src/domain_computation/primary_graph/application_installation/program/output_roots.rs",
        "append_required_bindings",
        "membership only: public root vocabulary accumulates source binding membership",
    ),
    (
        "src/domain_computation/primary_graph/application_installation/program/construction.rs",
        "require_rostered_binding_membership",
        "membership only: installation checks complete declared binding membership",
    ),
];

pub(super) fn validate(root: &std::path::Path) -> Result<Vec<Diagnostic>, String> {
    let mut diagnostics = Vec::new();
    for governed in
        super::workspace_crates::discover_workspace_crates(root, "workspaces/worth-query")?
    {
        if governed.package == CRATE {
            let graph = super::crate_modules::parse_crate_modules(&governed)?;
            diagnostics.extend(enforce(&governed, &graph));
        }
    }
    Ok(diagnostics)
}

fn enforce(governed: &GovernedCrate, graph: &ModuleGraph) -> Vec<Diagnostic> {
    if governed.package != CRATE {
        return Vec::new();
    }
    let files: Vec<_> = graph
        .modules
        .values()
        .map(|module| (module.relative_source.as_str(), module.items.as_slice()))
        .collect();
    findings(&files)
        .into_iter()
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

fn findings(files: &[(&str, &[syn::Item])]) -> Vec<inventory::Finding> {
    inventory::scan(files)
        .into_iter()
        .filter(|finding| {
            finding.reason != "ordered container keyed by TypeId"
                || !MEMBERSHIP_ONLY.iter().any(|(source, item, reason)| {
                    *source == finding.source
                        && *item == finding.item
                        && reason.starts_with("membership only:")
                })
        })
        .collect()
}
