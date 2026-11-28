use crate::{config::*, inputs::Inputs};
use std::collections::BTreeSet;

/// Refuse regular and build edges to dependencies registered as test-only.
pub(super) fn check(input: &Inputs) -> Vec<Diagnostic> {
    let testkits = strings(&input.config, "dev_only_dependencies");
    let mut denied = BTreeSet::new();
    if !testkits.is_empty() {
        for graph in input.graphs.values() {
            for package in &graph.packages {
                for dependency in &package.dependencies {
                    if dependency.kind.as_deref() != Some("dev")
                        && testkits.contains(&dependency.name)
                    {
                        denied.insert((&package.manifest_path, &package.name, &dependency.name));
                    }
                }
            }
        }
    }
    denied
        .into_iter()
        .map(|(path, source, target)| {
            diagnostic(
                Code::Dependency,
                path,
                format!("{source} -> {target}: testkit dependencies must be dev-only"),
            )
        })
        .collect()
}
