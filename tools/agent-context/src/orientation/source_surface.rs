use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use syn::{Item, ItemMod};

pub(crate) fn collect_owned_modules(src_root: &Path) -> Result<Vec<String>, String> {
    let mut modules = BTreeSet::new();
    for entry in fs::read_dir(src_root).map_err(|e| format!("read {}: {e}", src_root.display()))? {
        let entry = entry.map_err(|e| format!("read {} entry: {e}", src_root.display()))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "lib.rs" || name == "facade.rs" {
            continue;
        }
        if path.is_dir() {
            modules.insert(name);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            modules.insert(
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .ok_or_else(|| format!("invalid utf-8 module name in {}", path.display()))?
                    .to_owned(),
            );
        }
    }
    Ok(modules.into_iter().collect())
}

pub(crate) fn ensure_facade_only_public_surface(path: &Path) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let syntax = syn::parse_file(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
    let mut facade_exports = 0usize;
    for item in syntax.items {
        match item {
            Item::Mod(ItemMod {
                vis: syn::Visibility::Public(_),
                ident,
                content: None,
                attrs,
                ..
            }) if ident == "facade" && attrs.is_empty() => {
                facade_exports += 1;
            }
            Item::Mod(ItemMod {
                vis: syn::Visibility::Inherited,
                content: None,
                ..
            }) => {}
            Item::Use(item_use) if matches!(item_use.vis, syn::Visibility::Public(_)) => {
                return Err(facade_only_error(path));
            }
            _ => return Err(facade_only_error(path)),
        }
    }
    if facade_exports == 1 {
        Ok(())
    } else {
        Err(facade_only_error(path))
    }
}

fn facade_only_error(path: &Path) -> String {
    format!(
        "{} no longer exposes a facade-only public surface",
        path.display()
    )
}
