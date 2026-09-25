//! Raw coordinate representations stay at declared edges.
//!
//! Milestone 3.16.2 requirement 1 gives presented geometry sealed types that
//! carry their truth status. A bare `[f32; 2]`, `[f64; 4]`, or `(f32, f32)`
//! written in production source of a covered crate is one of those values with
//! its status erased, so it is legal only inside an edge the configuration
//! declares: serialization, platform event, GPU upload, and the few owners the
//! milestone reviews record. The rule reads written types, so every raw value
//! that crosses a function, field, or binding annotation is seen; an edge that
//! no longer holds a raw form is stale and is reported too.

#[cfg(test)]
mod tests;

use super::crate_modules::{
    parse_additional_production_target_graphs, parse_crate_modules, GovernedCrate, ModuleGraph,
};
use super::production_cfg::{compiled_out_of_production, module_compiled_out_of_production};
use crate::cargo_graph::package_name_from_manifest;
use crate::config::{RawGeometryDenialConfig, RawGeometryEdgeConfig};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::collections::BTreeSet;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Attribute, Expr, ImplItem, Item, Lit, TraitItem, Type};

/// Check the production source of every configured crate, library and
/// binaries alike, for raw coordinates.
pub(crate) fn enforce_raw_geometry_denials(
    root: &Path,
    rules: &[RawGeometryDenialConfig],
) -> Vec<Diagnostic> {
    rules
        .iter()
        .flat_map(|rule| match production_graphs(root, rule) {
            Ok(graphs) => check_graphs(&graphs, rule),
            Err(error) => vec![Diagnostic::new(
                DiagnosticCode::Bc7004RawGeometryRepresentation,
                &rule.crate_root,
                format!("covered crate could not be read for raw coordinates: {error}"),
            )],
        })
        .collect()
}

fn production_graphs(
    root: &Path,
    rule: &RawGeometryDenialConfig,
) -> Result<Vec<ModuleGraph>, String> {
    let crate_root = root.join(&rule.crate_root);
    let covered = GovernedCrate {
        package: package_name_from_manifest(&crate_root.join("Cargo.toml"))?,
        crate_root,
        relative_crate_root: rule.crate_root.clone(),
    };
    let mut graphs = vec![parse_crate_modules(&covered)?];
    graphs.extend(parse_additional_production_target_graphs(&covered)?);
    Ok(graphs)
}

fn check_graphs(graphs: &[ModuleGraph], rule: &RawGeometryDenialConfig) -> Vec<Diagnostic> {
    let mut used = vec![false; rule.edges.len()];
    // A source two targets both compile is reported once.
    let mut outside = BTreeSet::new();
    for graph in graphs {
        for (module_path, node) in &graph.modules {
            if module_compiled_out_of_production(graph, module_path)
                || declared_inline(graph, module_path)
            {
                continue;
            }
            let mut visitor = RawCoordinates::default();
            for item in &node.items {
                visitor.visit_item(item);
            }
            let sources: Vec<&str> = node.relative_source.split(';').collect();
            for found in visitor.found {
                let edge = rule.edges.iter().position(|edge| {
                    sources
                        .iter()
                        .any(|source| edge_holds(edge, source, &found.item))
                });
                match edge {
                    Some(index) => used[index] = true,
                    None => {
                        outside.insert((node.relative_source.clone(), found));
                    }
                }
            }
        }
    }
    let mut diagnostics: Vec<_> = outside
        .into_iter()
        .map(|(source, found)| {
            Diagnostic::new(
                DiagnosticCode::Bc7004RawGeometryRepresentation,
                format!("{}/{}:{}", rule.crate_root, source, found.line),
                format!(
                    "raw coordinate representation `{}` in `{}` sits outside every declared edge: {}",
                    found.shape, found.item, rule.guidance
                ),
            )
        })
        .collect();
    for (edge, used) in rule.edges.iter().zip(used) {
        if !used {
            diagnostics.push(Diagnostic::new(
                DiagnosticCode::Bc7004RawGeometryRepresentation,
                format!("{}/{}", rule.crate_root, edge.path),
                format!(
                    "declared {} edge {:?} ({}) holds no raw coordinate representation; remove it",
                    edge.kind.as_str(),
                    edge.items,
                    edge.reason
                ),
            ));
        }
    }
    diagnostics
}

/// An inline module is read with the file that declares it, so its item paths
/// name the module the way an edge written against that file does.
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

fn edge_holds(edge: &RawGeometryEdgeConfig, source: &str, item: &str) -> bool {
    let path = edge.path.trim_end_matches('/');
    let in_source = source == path
        || source
            .strip_prefix(path)
            .is_some_and(|rest| rest.starts_with('/'));
    in_source
        && (edge.items.is_empty()
            || edge.items.iter().any(|declared| {
                item == declared
                    || item
                        .strip_prefix(declared.as_str())
                        .is_some_and(|rest| rest.starts_with("::"))
            }))
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
struct Found {
    item: String,
    shape: String,
    line: usize,
}

#[derive(Default)]
struct RawCoordinates {
    scope: Vec<String>,
    found: Vec<Found>,
}

impl RawCoordinates {
    fn scoped(
        &mut self,
        attributes: &[Attribute],
        name: Option<String>,
        visit: impl FnOnce(&mut Self),
    ) {
        if compiled_out_of_production(attributes) {
            return;
        }
        let pushed = name.is_some();
        if let Some(name) = name {
            self.scope.push(name);
        }
        visit(self);
        if pushed {
            self.scope.pop();
        }
    }

    fn record(&mut self, shape: String, line: usize) {
        self.found.push(Found {
            item: self.scope.join("::"),
            shape,
            line,
        });
    }
}

impl<'ast> Visit<'ast> for RawCoordinates {
    fn visit_item(&mut self, item: &'ast Item) {
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
            _ => (&[], None),
        };
        self.scoped(attributes, name, |visitor| visit::visit_item(visitor, item));
    }

    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        let (attributes, name): (&[Attribute], Option<String>) = match item {
            ImplItem::Fn(function) => (&function.attrs, Some(function.sig.ident.to_string())),
            ImplItem::Const(item) => (&item.attrs, Some(item.ident.to_string())),
            ImplItem::Type(item) => (&item.attrs, Some(item.ident.to_string())),
            _ => (&[], None),
        };
        self.scoped(attributes, name, |visitor| {
            visit::visit_impl_item(visitor, item)
        });
    }

    fn visit_trait_item(&mut self, item: &'ast TraitItem) {
        let (attributes, name): (&[Attribute], Option<String>) = match item {
            TraitItem::Fn(function) => (&function.attrs, Some(function.sig.ident.to_string())),
            TraitItem::Const(item) => (&item.attrs, Some(item.ident.to_string())),
            TraitItem::Type(item) => (&item.attrs, Some(item.ident.to_string())),
            _ => (&[], None),
        };
        self.scoped(attributes, name, |visitor| {
            visit::visit_trait_item(visitor, item)
        });
    }

    fn visit_field(&mut self, field: &'ast syn::Field) {
        self.scoped(&field.attrs, None, |visitor| {
            visit::visit_field(visitor, field)
        });
    }

    fn visit_variant(&mut self, variant: &'ast syn::Variant) {
        self.scoped(&variant.attrs, None, |visitor| {
            visit::visit_variant(visitor, variant)
        });
    }

    fn visit_type_array(&mut self, array: &'ast syn::TypeArray) {
        if let (Some(scalar), Some(length)) =
            (float_scalar(&array.elem), coordinate_length(&array.len))
        {
            self.record(format!("[{scalar}; {length}]"), array.span().start().line);
        }
        visit::visit_type_array(self, array);
    }

    fn visit_type_tuple(&mut self, tuple: &'ast syn::TypeTuple) {
        let scalars: Vec<_> = tuple.elems.iter().map(float_scalar).collect();
        if let [Some(first), Some(second)] = scalars.as_slice() {
            self.record(format!("({first}, {second})"), tuple.span().start().line);
        }
        visit::visit_type_tuple(self, tuple);
    }
}

fn self_type_name(self_type: &Type) -> String {
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

fn float_scalar(element: &Type) -> Option<&'static str> {
    let Type::Path(path) = element else {
        return None;
    };
    if path.path.is_ident("f32") {
        Some("f32")
    } else if path.path.is_ident("f64") {
        Some("f64")
    } else {
        None
    }
}

/// A point or offset has two coordinates and a rectangle four.
fn coordinate_length(length: &Expr) -> Option<usize> {
    let Expr::Lit(literal) = length else {
        return None;
    };
    let Lit::Int(integer) = &literal.lit else {
        return None;
    };
    integer
        .base10_parse::<usize>()
        .ok()
        .filter(|length| matches!(length, 2 | 4))
}
