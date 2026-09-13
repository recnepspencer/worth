//! BC7001 requires named imports and reexports on governed authority surfaces.

use super::crate_modules::{GovernedCrate, ModuleGraph};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::collections::BTreeSet;
use syn::{Item, UseTree};

const EXPLICIT_IMPORT_LAW: &str =
    "Authority-governed source must use named imports and reexports. \
Glob imports and glob reexports are denied because they hide the exact public authority surface; \
name each imported or reexported module, type, and operation explicitly.";

pub(super) fn enforce_explicit_imports(
    governed: &GovernedCrate,
    graph: &ModuleGraph,
) -> Vec<Diagnostic> {
    let mut diagnostics = production_glob_sources(graph)
        .into_iter()
        .map(|relative_source| {
            Diagnostic::new(
                DiagnosticCode::Bc7001AuthoritySealing,
                format!("{}::{relative_source}", governed.package),
                EXPLICIT_IMPORT_LAW,
            )
        })
        .collect::<Vec<_>>();
    diagnostics.sort_by(Diagnostic::compare_subject_message);
    diagnostics.dedup_by(|left, right| left.has_same_subject_message(right));
    diagnostics
}

pub(super) fn production_glob_sources(graph: &ModuleGraph) -> BTreeSet<String> {
    let mut sources = BTreeSet::new();
    for (module_path, node) in &graph.modules {
        if module_is_test_only(graph, module_path) {
            continue;
        }
        for item in &node.items {
            let Item::Use(item_use) = item else {
                continue;
            };
            if !attributes_are_test_only(&item_use.attrs) && use_tree_contains_glob(&item_use.tree)
            {
                sources.insert(node.relative_source.clone());
            }
        }
    }
    sources
}

pub(super) fn module_is_test_only(graph: &ModuleGraph, module_path: &[String]) -> bool {
    for depth in 1..=module_path.len() {
        let parent_path = &module_path[..depth - 1];
        let Some(parent) = graph.modules.get(parent_path) else {
            return false;
        };
        let name = &module_path[depth - 1];
        let declarations = parent.items.iter().filter_map(|item| match item {
            Item::Mod(item_mod) if item_mod.ident == name => Some(item_mod),
            _ => None,
        });
        let declarations = declarations.collect::<Vec<_>>();
        if !declarations.is_empty()
            && declarations
                .iter()
                .all(|item_mod| attributes_are_test_only(&item_mod.attrs))
        {
            return true;
        }
    }
    false
}

fn attributes_are_test_only(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("cfg")
            && attribute
                .parse_args::<syn::Path>()
                .is_ok_and(|path| path.is_ident("test"))
    })
}

fn use_tree_contains_glob(tree: &UseTree) -> bool {
    match tree {
        UseTree::Path(path) => use_tree_contains_glob(&path.tree),
        UseTree::Group(group) => group.items.iter().any(use_tree_contains_glob),
        UseTree::Glob(_) => true,
        UseTree::Name(_) | UseTree::Rename(_) => false,
    }
}
