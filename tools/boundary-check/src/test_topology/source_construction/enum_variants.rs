//! Resolve declared enum variants to their owning type, never their value spelling.
use crate::source_rules::ModuleGraph;
use std::collections::BTreeMap;

pub(super) fn owners(graph: &ModuleGraph) -> BTreeMap<String, String> {
    let mut owners = BTreeMap::new();
    for (scope, node) in &graph.modules {
        for item in &node.items {
            if let syn::Item::Enum(item) = item {
                let owner = std::iter::once("crate".to_owned())
                    .chain(scope.iter().cloned())
                    .chain([item.ident.to_string()])
                    .collect::<Vec<_>>()
                    .join("::");
                for variant in &item.variants {
                    owners.insert(format!("{owner}::{}", variant.ident), owner.clone());
                }
            }
        }
    }
    owners
}
