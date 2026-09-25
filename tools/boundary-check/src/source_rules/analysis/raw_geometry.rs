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

use super::crate_modules::ModuleGraph;
use super::production_scope::{
    production_graphs, production_modules, site_holds, visit_field_in_scope,
    visit_impl_item_in_scope, visit_item_in_scope, visit_trait_item_in_scope,
    visit_variant_in_scope, ScopedVisitor,
};
use crate::config::{RawGeometryDenialConfig, RawGeometryEdgeConfig};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::collections::BTreeSet;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ImplItem, Item, Lit, TraitItem, Type};

/// Check the production source of every configured crate, library and
/// binaries alike, for raw coordinates.
pub(crate) fn enforce_raw_geometry_denials(
    root: &Path,
    rules: &[RawGeometryDenialConfig],
) -> Vec<Diagnostic> {
    rules
        .iter()
        .flat_map(|rule| match production_graphs(root, &rule.crate_root) {
            Ok(graphs) => check_graphs(&graphs, rule),
            Err(error) => vec![Diagnostic::new(
                DiagnosticCode::Bc7004RawGeometryRepresentation,
                &rule.crate_root,
                format!("covered crate could not be read for raw coordinates: {error}"),
            )],
        })
        .collect()
}

fn check_graphs(graphs: &[ModuleGraph], rule: &RawGeometryDenialConfig) -> Vec<Diagnostic> {
    let mut used = vec![false; rule.edges.len()];
    // A source two targets both compile is read once.
    let mut outside = BTreeSet::new();
    for module in production_modules(graphs) {
        let mut visitor = RawCoordinates::default();
        for item in module.items {
            visitor.visit_item(item);
        }
        for found in visitor.found {
            let edge = rule.edges.iter().position(|edge| {
                module
                    .sources()
                    .any(|source| edge_holds(edge, source, &found.item))
            });
            match edge {
                Some(index) => used[index] = true,
                None => {
                    outside.insert((module.relative_source.to_owned(), found));
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

fn edge_holds(edge: &RawGeometryEdgeConfig, source: &str, item: &str) -> bool {
    site_holds(&edge.path, &edge.items, source, item)
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

impl ScopedVisitor for RawCoordinates {
    fn scope(&mut self) -> &mut Vec<String> {
        &mut self.scope
    }
}

impl RawCoordinates {
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
        visit_item_in_scope(self, item);
    }

    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        visit_impl_item_in_scope(self, item);
    }

    fn visit_trait_item(&mut self, item: &'ast TraitItem) {
        visit_trait_item_in_scope(self, item);
    }

    fn visit_field(&mut self, field: &'ast syn::Field) {
        visit_field_in_scope(self, field);
    }

    fn visit_variant(&mut self, variant: &'ast syn::Variant) {
        visit_variant_in_scope(self, variant);
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
