//! Orientation names declarations in the actual facade; rustc resolves exports.
use std::{collections::BTreeSet, fs, path::Path};
use syn::{Item, UseTree, Visibility};

pub(crate) fn facade_exports(crate_root: &Path) -> Result<Vec<String>, String> {
    let source = [
        crate_root.join("src/facade.rs"),
        crate_root.join("src/facade/mod.rs"),
    ]
    .into_iter()
    .find(|p| p.is_file())
    .ok_or_else(|| format!("{} has no facade source", crate_root.display()))?;
    let text = fs::read_to_string(&source).map_err(|e| e.to_string())?;
    let file = syn::parse_file(&text).map_err(|e| format!("{}: {e}", source.display()))?;
    let mut names = BTreeSet::new();
    for item in file.items {
        match item {
            Item::Use(item) if matches!(item.vis, Visibility::Public(_)) => {
                imports(&item.tree, "", &mut names)
            }
            Item::Mod(item) if matches!(item.vis, Visibility::Public(_)) => {
                names.insert(item.ident.to_string());
            }
            _ => {}
        }
    }
    Ok(names.into_iter().collect())
}
fn imports(tree: &UseTree, prefix: &str, names: &mut BTreeSet<String>) {
    match tree {
        UseTree::Path(p) => imports(&p.tree, &format!("{prefix}{}::", p.ident), names),
        UseTree::Name(n) => {
            names.insert(n.ident.to_string());
        }
        UseTree::Rename(n) => {
            names.insert(n.rename.to_string());
        }
        UseTree::Group(g) => {
            for t in &g.items {
                imports(t, prefix, names);
            }
        }
        UseTree::Glob(_) => {
            names.insert(format!("{prefix}*"));
        }
    }
}
