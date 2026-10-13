//! Rustc-compatible module source selection for the sealing inventory.
//!
//! `#[path]` is resolved from the containing source file's physical directory;
//! inline module nesting appends its virtual components to that base.
//! Every selectable `path` / `cfg_attr(..., path = ...)` branch is collected
//! fail-closed so conditional path selection cannot hide a hostile body.

use std::path::{Component, Path, PathBuf};
use syn::{Attribute, Lit, Meta};

/// Virtual directory of a module for resolving child `#[path]` and out-of-line files.
pub(super) fn child_module_dir(parent_dir: &Path, child_name: &str) -> PathBuf {
    parent_dir.join(child_name)
}

/// Directory used for nested resolution after loading a source file.
pub(super) fn directory_after_loading_file(source_path: &Path) -> PathBuf {
    if source_path.file_name().and_then(|s| s.to_str()) == Some("mod.rs") {
        source_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    } else {
        // `foo.rs` → virtual directory `foo/`
        source_path.with_extension("")
    }
}

/// Base directory rustc uses for `#[path]` inside the current source module.
///
/// Conventional `foo.rs` children live under `foo/`, but a path attribute in
/// `foo.rs` is relative to the physical directory containing `foo.rs`. Inline
/// module nesting is retained on top of that physical directory.
pub(super) fn path_attribute_dir(parent_source: &Path, module_dir: &Path) -> PathBuf {
    let conventional_base = directory_after_loading_file(parent_source);
    let Some(physical_parent) = parent_source.parent() else {
        return module_dir.to_path_buf();
    };
    match module_dir.strip_prefix(&conventional_base) {
        Ok(inline_suffix) => physical_parent.join(inline_suffix),
        Err(_) => module_dir.to_path_buf(),
    }
}

/// Inline module directories are virtual. Collapse their lexical `..` before
/// consulting the filesystem, which otherwise requires the virtual directory
/// to exist on Unix even when its parent names a real source file.
pub(super) fn path_attribute_source(parent_dir: &Path, selector: &str) -> PathBuf {
    let joined = parent_dir.join(selector);
    let mut resolved = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => match resolved.components().next_back() {
                Some(Component::Normal(_)) => {
                    resolved.pop();
                }
                Some(Component::RootDir | Component::Prefix(_)) => {}
                Some(Component::ParentDir | Component::CurDir) | None => resolved.push(".."),
            },
            other => resolved.push(other.as_os_str()),
        }
    }
    resolved
}

/// All module path selectors on a `mod` item: direct `#[path]` and nested
/// `cfg_attr(..., path = "...")` branches (fail-closed inventory of every branch).
pub(super) fn all_path_selectors(attrs: &[Attribute]) -> Vec<String> {
    let mut out = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("path") {
            if let Some(value) = path_attr_string(attr) {
                out.push(value);
            }
            continue;
        }
        if attr.path().is_ident("cfg_attr") {
            collect_cfg_attr_paths(attr, &mut out);
        }
    }
    out.sort();
    out.dedup();
    out
}

fn path_attr_string(attr: &Attribute) -> Option<String> {
    match &attr.meta {
        Meta::NameValue(nv) => {
            if let syn::Expr::Lit(expr_lit) = &nv.value {
                if let Lit::Str(s) = &expr_lit.lit {
                    return Some(s.value());
                }
            }
            None
        }
        Meta::List(list) => syn::parse2::<syn::LitStr>(list.tokens.clone())
            .ok()
            .map(|s| s.value()),
        _ => None,
    }
}

fn collect_cfg_attr_paths(attr: &Attribute, out: &mut Vec<String>) {
    let Meta::List(list) = &attr.meta else {
        return;
    };
    let groups = split_top_level_comma_groups(list.tokens.clone());
    // Skip predicate (first group); remaining groups are nested attributes.
    for nested in groups.into_iter().skip(1) {
        collect_path_from_nested_tokens(nested, out);
    }
}

fn collect_path_from_nested_tokens(tokens: proc_macro2::TokenStream, out: &mut Vec<String>) {
    let Ok(meta) = syn::parse2::<Meta>(tokens) else {
        return;
    };
    match &meta {
        Meta::Path(path) if path.is_ident("path") => {}
        Meta::NameValue(nv) if nv.path.is_ident("path") => {
            if let syn::Expr::Lit(expr_lit) = &nv.value {
                if let Lit::Str(s) = &expr_lit.lit {
                    out.push(s.value());
                }
            }
        }
        Meta::List(list) if list.path.is_ident("path") => {
            if let Ok(s) = syn::parse2::<syn::LitStr>(list.tokens.clone()) {
                out.push(s.value());
            }
        }
        Meta::List(list) if list.path.is_ident("cfg_attr") => {
            let groups = split_top_level_comma_groups(list.tokens.clone());
            for nested in groups.into_iter().skip(1) {
                collect_path_from_nested_tokens(nested, out);
            }
        }
        _ => {}
    }
}

fn split_top_level_comma_groups(tokens: proc_macro2::TokenStream) -> Vec<proc_macro2::TokenStream> {
    use proc_macro2::TokenTree;
    let mut groups = Vec::new();
    let mut current = Vec::new();
    for tree in tokens {
        match &tree {
            TokenTree::Punct(punct) if punct.as_char() == ',' => {
                groups.push(current.into_iter().collect());
                current = Vec::new();
            }
            _ => current.push(tree),
        }
    }
    if !current.is_empty() {
        groups.push(current.into_iter().collect());
    }
    groups
}

/// Resolve out-of-line module sources for a child `mod` under its two rustc
/// bases: the conventional module directory and the path-attribute directory.
///
/// Returns every concrete file that may be compiled: all path-attribute targets
/// plus conventional candidates when present (covers conditional `cfg_attr(path)`).
pub(super) fn resolve_child_sources(
    conventional_parent_dir: &Path,
    path_attribute_parent_dir: &Path,
    child_name: &str,
    attrs: &[Attribute],
) -> Result<Vec<PathBuf>, String> {
    let mut sources = Vec::new();
    let selectors = all_path_selectors(attrs);
    for rel in &selectors {
        let resolved = path_attribute_source(path_attribute_parent_dir, rel);
        if resolved.is_file() {
            sources.push(resolved);
        } else {
            return Err(format!(
                "unresolved #[path = \"{rel}\"] for module `{child_name}` under {}",
                path_attribute_parent_dir.display()
            ));
        }
    }

    // Conventional locations always inventoried when present so a cfg_attr(path)
    // that is inactive still cannot hide behind a decoy-only conventional scan,
    // and so path-less modules resolve normally.
    let conventional = [
        conventional_parent_dir.join(child_name).join("mod.rs"),
        conventional_parent_dir.join(format!("{child_name}.rs")),
    ];
    for candidate in conventional {
        if candidate.is_file() && !sources.iter().any(|s| s == &candidate) {
            sources.push(candidate);
        }
    }

    if sources.is_empty() {
        return Err(format!(
            "unresolved module `{child_name}` under {}; expected path attribute or conventional file",
            conventional_parent_dir.display()
        ));
    }
    Ok(sources)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_selector_resolves_without_a_physical_inline_directory() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/source_rules/analysis");
        let virtual_directory = directory.join("unmaterialized-inline-module");
        assert!(!virtual_directory.exists());
        let attributes = [syn::parse_quote!(#[path = "../module_source.rs"])];
        let sources = resolve_child_sources(
            &virtual_directory,
            &virtual_directory,
            "selected_source",
            &attributes,
        )
        .unwrap();
        assert_eq!(sources, [directory.join("module_source.rs")]);
    }

    #[test]
    fn parent_selector_still_refuses_a_missing_source() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/source_rules/analysis/unmaterialized-inline-module");
        let attributes = [syn::parse_quote!(#[path = "../unmaterialized-source.rs"])];
        assert!(resolve_child_sources(&directory, &directory, "missing", &attributes).is_err());
        assert_eq!(
            path_attribute_source(Path::new("one"), "../../file.rs"),
            Path::new("../file.rs")
        );
    }
}
