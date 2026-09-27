use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::fs;
use std::path::Path;
use syn::{Item, ItemMod, UseTree, Visibility};

pub(super) fn validate_facade_only_lib(
    path: &Path,
    relative: &str,
) -> Result<Vec<Diagnostic>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let syntax = crate::source_syntax::parse_file(&text)
        .map_err(|e| format!("parse {}: {e}", path.display()))?;
    let mut diagnostics = Vec::new();
    let mut facade_exports = 0usize;

    for item in syntax.items {
        match item {
            Item::Mod(ItemMod {
                vis: Visibility::Public(_),
                ident,
                content: None,
                ..
            }) if ident == "facade" => {
                facade_exports += 1;
            }
            Item::Mod(ItemMod {
                vis: Visibility::Inherited,
                content: None,
                ..
            }) => {}
            Item::Use(item_use) if matches!(item_use.vis, Visibility::Public(_)) => {
                diagnostics.push(Diagnostic::new(
                    DiagnosticCode::Bc3003QueryAudienceFacadeContract,
                    format!("{relative}/src/lib.rs"),
                    "Query audience lib.rs must export only the facade module".to_owned(),
                ));
            }
            _ => {
                diagnostics.push(Diagnostic::new(
                    DiagnosticCode::Bc3003QueryAudienceFacadeContract,
                    format!("{relative}/src/lib.rs"),
                    "Query audience lib.rs must remain facade-only wiring (no behavior items)"
                        .to_owned(),
                ));
            }
        }
    }

    if facade_exports != 1 {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc3003QueryAudienceFacadeContract,
            format!("{relative}/src/lib.rs"),
            "Query audience lib.rs must declare exactly one public facade module".to_owned(),
        ));
    }

    Ok(diagnostics)
}

pub(super) fn validate_reexport_only_facade(
    path: &Path,
    relative: &str,
    authority_packages: &[String],
    audience_packages: &[&str],
) -> Result<Vec<Diagnostic>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let syntax = crate::source_syntax::parse_file(&text)
        .map_err(|e| format!("parse {}: {e}", path.display()))?;
    let mut diagnostics = Vec::new();
    let authority_crates = authority_packages
        .iter()
        .map(|package| package.replace('-', "_"))
        .collect::<Vec<_>>();

    if syntax.items.is_empty() {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc3003QueryAudienceFacadeContract,
            format!("{relative}/src/facade.rs"),
            "Query audience facade.rs must not be empty".to_owned(),
        ));
        return Ok(diagnostics);
    }

    for item in syntax.items {
        match item {
            Item::Use(item_use) if matches!(item_use.vis, Visibility::Public(_)) => {
                validate_reexport(
                    &item_use.tree,
                    relative,
                    &authority_crates,
                    audience_packages,
                    &mut diagnostics,
                );
            }
            _ => {
                diagnostics.push(Diagnostic::new(
                    DiagnosticCode::Bc3003QueryAudienceFacadeContract,
                    format!("{relative}/src/facade.rs"),
                    "Query audience facade.rs must contain only public re-exports (no wrappers, functions, or types)"
                        .to_owned(),
                ));
            }
        }
    }

    Ok(diagnostics)
}

fn validate_reexport(
    tree: &UseTree,
    relative: &str,
    authority_crates: &[String],
    audience_packages: &[&str],
    diagnostics: &mut Vec<Diagnostic>,
) {
    if use_tree_is_glob(tree) {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc3003QueryAudienceFacadeContract,
            format!("{relative}/src/facade.rs"),
            "Query audience facade.rs must not use glob re-exports".to_owned(),
        ));
    }
    let Some(root) = use_tree_root_ident(tree) else {
        return;
    };
    if !authority_crates.iter().any(|allowed| allowed == &root) {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc3003QueryAudienceFacadeContract,
            format!("{relative}/src/facade.rs"),
            format!(
                "Query audience facade.rs may re-export only from configured authority crates {}; found `{root}`",
                authority_crates.join(", ")
            ),
        ));
    }
    let root_package = root.replace('_', "-");
    if audience_packages.contains(&root_package.as_str()) {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc3003QueryAudienceFacadeContract,
            format!("{relative}/src/facade.rs"),
            format!(
                "Query audience facade must not re-export another audience facade `{root_package}`"
            ),
        ));
    }
}

fn use_tree_is_glob(tree: &UseTree) -> bool {
    match tree {
        UseTree::Glob(_) => true,
        UseTree::Path(path) => use_tree_is_glob(&path.tree),
        UseTree::Group(group) => group.items.iter().any(use_tree_is_glob),
        UseTree::Name(_) | UseTree::Rename(_) => false,
    }
}

fn use_tree_root_ident(tree: &UseTree) -> Option<String> {
    match tree {
        UseTree::Path(path) => Some(path.ident.to_string()),
        UseTree::Name(name) => Some(name.ident.to_string()),
        UseTree::Rename(rename) => Some(rename.ident.to_string()),
        UseTree::Group(group) => group.items.first().and_then(use_tree_root_ident),
        UseTree::Glob(_) => None,
    }
}
