use super::workspace_source_inventory::WorkspaceSourceInventory;
use std::path::Path;
use syn::{Item, ItemMod, ItemUse, UseTree, Visibility};

const FACADE_ROOTS: &[&str] = &[
    "crates/worth-ui/src/facade",
    "crates/worth-ui-runtime/src/facade",
];

pub fn audit_product_lifecycle_facade(inventory: &WorkspaceSourceInventory) -> Vec<String> {
    let mut violations = Vec::new();
    for facade_root in FACADE_ROOTS {
        for source in inventory.rust_files_under(facade_root) {
            violations.extend(facade_source_violations(
                source.text(),
                source.relative_path(),
                facade_root,
            ));
        }
    }
    violations.sort();
    violations.dedup();
    violations
}

fn facade_source_violations(source: &str, path: &Path, facade_root: &str) -> Vec<String> {
    let syntax = match syn::parse_file(source) {
        Ok(syntax) => syntax,
        Err(error) => {
            return vec![format!(
                "{} could not be parsed for facade authority audit: {error}",
                path.display()
            )]
        }
    };
    let mut violations = Vec::new();
    for item in &syntax.items {
        match item {
            Item::Use(item_use) if is_public(&item_use.vis) => {
                audit_public_use(item_use, path, facade_root, &mut violations)
            }
            Item::Mod(item_mod) if is_public(&item_mod.vis) => {
                audit_public_module(item_mod, path, &mut violations)
            }
            _ => {}
        }
    }
    violations
}

fn audit_public_use(
    item_use: &ItemUse,
    path: &Path,
    facade_root: &str,
    violations: &mut Vec<String>,
) {
    let mut leaves = Vec::new();
    flatten_use_tree(&item_use.tree, Vec::new(), &mut leaves, violations, path);
    for leaf in leaves {
        let rendered = leaf.join("::");
        if leaf.iter().any(|segment| {
            matches!(
                segment.as_str(),
                "runtime_exports" | "compat_modules" | "compatibility"
            )
        }) {
            violations.push(format!(
                "{} publicly routes through compatibility or aggregate exports: `{rendered}`",
                path.display()
            ));
        }
        if let Some(name) = leaf.last()
            && let Some(reason) = forbidden_authority_family(name)
        {
            violations.push(format!(
                "{} exposes `{name}` through a public facade; {reason}",
                path.display()
            ));
        }

        let runtime_facade_root =
            facade_root.ends_with("worth-ui-runtime/src/facade") && path.ends_with("facade/mod.rs");
        if runtime_facade_root && leaf.windows(2).any(|pair| pair == ["crate", "runtime"]) {
            violations.push(format!(
                "{} exports runtime internals from the facade root; route `{rendered}` through a named lifecycle facade",
                path.display()
            ));
        }
    }
}

fn flatten_use_tree(
    tree: &UseTree,
    prefix: Vec<String>,
    leaves: &mut Vec<Vec<String>>,
    violations: &mut Vec<String>,
    path: &Path,
) {
    match tree {
        UseTree::Path(branch) => {
            let mut next = prefix;
            next.push(branch.ident.to_string());
            flatten_use_tree(&branch.tree, next, leaves, violations, path);
        }
        UseTree::Name(name) => {
            let mut leaf = prefix;
            leaf.push(name.ident.to_string());
            leaves.push(leaf);
        }
        UseTree::Rename(rename) => {
            let mut leaf = prefix;
            leaf.push(rename.ident.to_string());
            leaves.push(leaf);
        }
        UseTree::Group(group) => {
            for item in &group.items {
                flatten_use_tree(item, prefix.clone(), leaves, violations, path);
            }
        }
        UseTree::Glob(_) => violations.push(format!(
            "{} contains a public wildcard export; facade membership must be reviewed item by item",
            path.display()
        )),
    }
}

fn audit_public_module(item_mod: &ItemMod, path: &Path, violations: &mut Vec<String>) {
    let name = item_mod.ident.to_string();
    if matches!(
        name.as_str(),
        "runtime_exports" | "compat_modules" | "compatibility"
    ) {
        violations.push(format!(
            "{} publishes compatibility or aggregate facade module `{name}`",
            path.display()
        ));
    }
}

fn forbidden_authority_family(name: &str) -> Option<&'static str> {
    let raw_plan = name.contains("ExecutionPlanInput")
        || name.contains("PlanningLaneInput")
        || name.contains("ExecutionLaneInput")
        || (name.contains("Candidate") && name.contains("ExecutionPlan"));
    if raw_plan {
        return Some("callers must operate on active-session capabilities, not raw plan input");
    }
    if name.contains("LoweringAuthority") || name.contains("LoweringBasis") {
        return Some("lowering authority and its proof basis must remain behind preparation");
    }
    if name.contains("FrameExecutor") || name.contains("PlanExecutor") {
        return Some("executor injection would create a second execution authority");
    }
    if name.contains("ExecutionPlanDigest") || name.contains("PlanNodeDigest") {
        return Some("digests are evidence inputs, not public equivalence authority");
    }
    None
}

fn is_public(visibility: &Visibility) -> bool {
    matches!(visibility, Visibility::Public(_))
}

#[cfg(test)]
mod tests {
    use super::facade_source_violations;
    use std::path::Path;

    #[test]
    fn audit_rejects_authority_shapes_and_aggregate_routing() {
        let source = r#"
            pub mod runtime_exports;
            pub use crate::runtime::exports::*;
            pub use crate::runtime::{
                AcmeCandidateExecutionPlan,
                AcmeExecutionPlanLoweringAuthority,
                AcmeLaneFrameExecutor,
                AcmeExecutionPlanDigest,
            };
        "#;
        let violations = facade_source_violations(
            source,
            Path::new("crates/worth-ui-runtime/src/facade/mod.rs"),
            "crates/worth-ui-runtime/src/facade",
        );
        assert!(violations.iter().any(|row| row.contains("wildcard")));
        assert!(violations.iter().any(|row| row.contains("raw plan input")));
        assert!(violations
            .iter()
            .any(|row| row.contains("lowering authority")));
        assert!(violations
            .iter()
            .any(|row| row.contains("executor injection")));
        assert!(violations
            .iter()
            .any(|row| row.contains("equivalence authority")));
        assert!(violations
            .iter()
            .any(|row| row.contains("aggregate facade module")));
    }

    #[test]
    fn named_lifecycle_facade_may_curate_runtime_owned_outcomes() {
        let source = r#"
            pub use crate::runtime::{WorthUiRuntime, WorthUiFrameworkTurnCompletion};
        "#;
        assert!(facade_source_violations(
            source,
            Path::new("crates/worth-ui-runtime/src/facade/runtime_handoff/mod.rs"),
            "crates/worth-ui-runtime/src/facade",
        )
        .is_empty());
    }
}
