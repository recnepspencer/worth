use std::fs;
use std::path::{Path, PathBuf};
use syn::{Item, Visibility};

pub(super) fn validate_facade_only_surface(manifest: &Path) -> Result<PathBuf, String> {
    let surface = crate::source_rules::observe_compiled_library_surface(manifest)?;
    validate_lib(&surface.library_source)?;
    if let Some(source) = surface.exported_macro_sources.first() {
        return Err(format!(
            "governed public macros must not bypass facade.rs in {source}"
        ));
    }
    let source_directory = surface.library_source.parent().ok_or_else(|| {
        format!(
            "{} has no source directory",
            surface.library_source.display()
        )
    })?;
    Ok(source_directory.join("facade.rs"))
}

fn validate_lib(path: &Path) -> Result<(), String> {
    let syntax = parse(path)?;
    let mut facade_bindings = 0usize;
    for item in syntax.items {
        let allowed_facade = match &item {
            Item::Mod(module)
                if matches!(module.vis, Visibility::Public(_)) && module.ident == "facade" =>
            {
                facade_bindings += 1;
                module.content.is_none() && module.attrs.is_empty()
            }
            _ => false,
        };
        if item_is_public(&item) && !allowed_facade {
            return Err(format!(
                "governed crate root may expose only exactly-bound `pub mod facade;` without attributes in {}",
                path.display()
            ));
        }
    }
    if facade_bindings == 1 {
        Ok(())
    } else {
        Err(format!(
            "governed crate root must bind exactly one `pub mod facade;` in {}",
            path.display()
        ))
    }
}

fn parse(path: &Path) -> Result<syn::File, String> {
    let text =
        fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    crate::source_syntax::parse_file(&text)
        .map_err(|error| format!("parse {}: {error}", path.display()))
}

fn item_is_public(item: &Item) -> bool {
    match item {
        Item::Const(value) => matches!(value.vis, Visibility::Public(_)),
        Item::Enum(value) => matches!(value.vis, Visibility::Public(_)),
        Item::ExternCrate(value) => matches!(value.vis, Visibility::Public(_)),
        Item::Fn(value) => matches!(value.vis, Visibility::Public(_)),
        Item::Mod(value) => matches!(value.vis, Visibility::Public(_)),
        Item::Static(value) => matches!(value.vis, Visibility::Public(_)),
        Item::Struct(value) => matches!(value.vis, Visibility::Public(_)),
        Item::Trait(value) => matches!(value.vis, Visibility::Public(_)),
        Item::TraitAlias(value) => matches!(value.vis, Visibility::Public(_)),
        Item::Type(value) => matches!(value.vis, Visibility::Public(_)),
        Item::Union(value) => matches!(value.vis, Visibility::Public(_)),
        Item::Use(value) => matches!(value.vis, Visibility::Public(_)),
        _ => false,
    }
}
