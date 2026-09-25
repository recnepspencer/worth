//! The production source of one covered crate, read item by item.
//!
//! Rules that admit a raw form or a construction only at declared sites read a
//! crate the same way: every production target, each source once, and every
//! item named by its path within its file, so a site declared against a file
//! and item matches exactly what the rule saw.

use super::crate_modules::{
    parse_additional_production_target_graphs, parse_crate_modules, GovernedCrate, ModuleGraph,
};
use super::production_cfg::{compiled_out_of_production, module_compiled_out_of_production};
use crate::cargo_graph::package_name_from_manifest;
use std::collections::BTreeSet;
use std::path::Path;
use syn::visit::{self, Visit};
use syn::{Attribute, ImplItem, Item, TraitItem, Type};

/// The library and binary module graphs of the crate at `crate_root`.
pub(super) fn production_graphs(root: &Path, crate_root: &str) -> Result<Vec<ModuleGraph>, String> {
    let absolute_root = root.join(crate_root);
    let covered = GovernedCrate {
        package: package_name_from_manifest(&absolute_root.join("Cargo.toml"))?,
        crate_root: absolute_root,
        relative_crate_root: crate_root.to_owned(),
    };
    let mut graphs = vec![parse_crate_modules(&covered)?];
    graphs.extend(parse_additional_production_target_graphs(&covered)?);
    Ok(graphs)
}

/// One production module: the sources it is read from and its items.
pub(super) struct ProductionModule<'a> {
    pub(super) relative_source: &'a str,
    /// A file module's inner attributes.
    pub(super) attributes: &'a [Attribute],
    pub(super) items: &'a [Item],
}

impl ProductionModule<'_> {
    pub(super) fn sources(&self) -> impl Iterator<Item = &str> {
        self.relative_source.split(';')
    }
}

/// Every production module once. A source two targets both compile is read
/// once, and an inline module is read with the file that declares it, so its
/// items are named the way a site written against that file names them.
pub(super) fn production_modules(graphs: &[ModuleGraph]) -> Vec<ProductionModule<'_>> {
    let mut read = BTreeSet::new();
    let mut modules = Vec::new();
    for graph in graphs {
        for (module_path, node) in &graph.modules {
            if module_compiled_out_of_production(graph, module_path)
                || declared_inline(graph, module_path)
                || !read.insert(node.relative_source.as_str())
            {
                continue;
            }
            modules.push(ProductionModule {
                relative_source: &node.relative_source,
                attributes: &node.attributes,
                items: &node.items,
            });
        }
    }
    modules
}

fn declared_inline(graph: &ModuleGraph, module_path: &[String]) -> bool {
    let Some((name, parent)) = module_path.split_last() else {
        return false;
    };
    graph.modules.get(parent).is_some_and(|parent| {
        parent.items.iter().any(|item| {
            matches!(item, Item::Mod(item_mod) if item_mod.ident == name && item_mod.content.is_some())
        })
    })
}

/// Whether a declared site, a source path relative to the crate root and
/// optionally item paths within it, holds `item` in `source`. A path holds
/// that file, or a directory and everything under it; an item holds itself
/// and the items nested in it.
pub(super) fn site_holds(path: &str, items: &[String], source: &str, item: &str) -> bool {
    let path = path.trim_end_matches('/');
    let in_source = source == path
        || source
            .strip_prefix(path)
            .is_some_and(|rest| rest.starts_with('/'));
    in_source
        && (items.is_empty()
            || items.iter().any(|declared| {
                item == declared
                    || item
                        .strip_prefix(declared.as_str())
                        .is_some_and(|rest| rest.starts_with("::"))
            }))
}

/// A visitor that names the item it stands in and skips source a production
/// build never compiles.
pub(super) trait ScopedVisitor {
    fn scope(&mut self) -> &mut Vec<String>;
}

fn scoped<V: ScopedVisitor>(
    visitor: &mut V,
    attributes: &[Attribute],
    name: Option<String>,
    inner: impl FnOnce(&mut V),
) {
    if compiled_out_of_production(attributes) {
        return;
    }
    let pushed = name.is_some();
    if let Some(name) = name {
        visitor.scope().push(name);
    }
    inner(visitor);
    if pushed {
        visitor.scope().pop();
    }
}

pub(super) fn visit_item_in_scope<'ast, V: ScopedVisitor + Visit<'ast>>(
    visitor: &mut V,
    item: &'ast Item,
) {
    let (attributes, name): (&[Attribute], Option<String>) = match item {
        Item::Fn(function) => (&function.attrs, Some(function.sig.ident.to_string())),
        Item::Struct(item) => (&item.attrs, Some(item.ident.to_string())),
        Item::Enum(item) => (&item.attrs, Some(item.ident.to_string())),
        Item::Union(item) => (&item.attrs, Some(item.ident.to_string())),
        Item::Trait(item) => (&item.attrs, Some(item.ident.to_string())),
        Item::Type(item) => (&item.attrs, Some(item.ident.to_string())),
        Item::Const(item) => (&item.attrs, Some(item.ident.to_string())),
        Item::Static(item) => (&item.attrs, Some(item.ident.to_string())),
        Item::Impl(item) => (&item.attrs, Some(self_type_name(&item.self_ty))),
        // A file module is its own node; only inline content is read here.
        Item::Mod(item) if item.content.is_none() => return,
        Item::Mod(item) => (&item.attrs, Some(item.ident.to_string())),
        Item::Use(item) => (&item.attrs, None),
        Item::Macro(item) => (&item.attrs, None),
        _ => (&[], None),
    };
    scoped(visitor, attributes, name, |visitor| {
        visit::visit_item(visitor, item);
    });
}

pub(super) fn visit_impl_item_in_scope<'ast, V: ScopedVisitor + Visit<'ast>>(
    visitor: &mut V,
    item: &'ast ImplItem,
) {
    let (attributes, name): (&[Attribute], Option<String>) = match item {
        ImplItem::Fn(function) => (&function.attrs, Some(function.sig.ident.to_string())),
        ImplItem::Const(item) => (&item.attrs, Some(item.ident.to_string())),
        ImplItem::Type(item) => (&item.attrs, Some(item.ident.to_string())),
        _ => (&[], None),
    };
    scoped(visitor, attributes, name, |visitor| {
        visit::visit_impl_item(visitor, item);
    });
}

pub(super) fn visit_trait_item_in_scope<'ast, V: ScopedVisitor + Visit<'ast>>(
    visitor: &mut V,
    item: &'ast TraitItem,
) {
    let (attributes, name): (&[Attribute], Option<String>) = match item {
        TraitItem::Fn(function) => (&function.attrs, Some(function.sig.ident.to_string())),
        TraitItem::Const(item) => (&item.attrs, Some(item.ident.to_string())),
        TraitItem::Type(item) => (&item.attrs, Some(item.ident.to_string())),
        _ => (&[], None),
    };
    scoped(visitor, attributes, name, |visitor| {
        visit::visit_trait_item(visitor, item);
    });
}

pub(super) fn visit_field_in_scope<'ast, V: ScopedVisitor + Visit<'ast>>(
    visitor: &mut V,
    field: &'ast syn::Field,
) {
    scoped(visitor, &field.attrs, None, |visitor| {
        visit::visit_field(visitor, field);
    });
}

pub(super) fn visit_variant_in_scope<'ast, V: ScopedVisitor + Visit<'ast>>(
    visitor: &mut V,
    variant: &'ast syn::Variant,
) {
    scoped(visitor, &variant.attrs, None, |visitor| {
        visit::visit_variant(visitor, variant);
    });
}

/// The last path segment of an impl's self type: the name an item path uses.
pub(super) fn self_type_name(self_type: &Type) -> String {
    match self_type {
        Type::Path(path) => path
            .path
            .segments
            .last()
            .map_or_else(String::new, |segment| segment.ident.to_string()),
        Type::Reference(reference) => self_type_name(&reference.elem),
        _ => String::from("_"),
    }
}
