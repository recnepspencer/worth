//! Fresh def-site (`mixed_site`) identifiers for `#[derive(WorthQueryProjection)]` / `WorthQueryRow`.
use super::grammar::{ProjectionDeclaration, SlotKind};
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
use std::collections::BTreeSet;
use syn::{DeriveInput, Ident};

pub(super) struct ProjectionHygiene {
    pub(super) schema_alias: Ident,
    pub(super) root_alias: Ident,
    pub(super) q_ty: Ident,
    pub(super) b_ty: Ident,
    pub(super) row_local: Ident,
    pub(super) related_local: Ident,
}

impl ProjectionHygiene {
    pub(super) fn new(input: &DeriveInput, projection: &ProjectionDeclaration) -> Self {
        let mut used = collect_used_idents(input, projection);
        let mixed = Span::mixed_site();
        Self {
            schema_alias: fresh_ident("__WorthProjSchema", mixed, &mut used),
            root_alias: fresh_ident("__WorthProjRoot", mixed, &mut used),
            q_ty: fresh_ident("__WorthProjQ", mixed, &mut used),
            b_ty: fresh_ident("__WorthProjB", mixed, &mut used),
            row_local: fresh_ident("__worth_proj_row", mixed, &mut used),
            related_local: fresh_ident("__worth_proj_related", mixed, &mut used),
        }
    }
}

fn collect_used_idents(
    input: &DeriveInput,
    projection: &ProjectionDeclaration,
) -> BTreeSet<String> {
    let mut used = BTreeSet::new();
    collect_stream(input.ident.to_token_stream(), &mut used);
    collect_stream(projection.schema.to_token_stream(), &mut used);
    collect_stream(projection.root.to_token_stream(), &mut used);
    if let Some(ns) = &projection.namespace {
        collect_stream(ns.to_token_stream(), &mut used);
    }
    for slot in &projection.slots {
        collect_stream(slot.member.to_token_stream(), &mut used);
        collect_stream(slot.selector.to_token_stream(), &mut used);
        collect_stream(slot.marker.to_token_stream(), &mut used);
        collect_stream(slot.value.to_token_stream(), &mut used);
        match &slot.kind {
            SlotKind::Field { selection, .. } => {
                collect_stream(selection.to_token_stream(), &mut used);
            }
            SlotKind::Relation(tr) => {
                collect_stream(tr.relation.to_token_stream(), &mut used);
                collect_stream(tr.child.to_token_stream(), &mut used);
                collect_stream(tr.nested.to_token_stream(), &mut used);
                if let Some(rev) = &tr.reverse_from {
                    collect_stream(rev.to_token_stream(), &mut used);
                }
            }
        }
    }
    for name in &projection.names {
        collect_stream(name.member.to_token_stream(), &mut used);
        collect_stream(name.value.to_token_stream(), &mut used);
    }
    used
}

fn collect_stream(stream: TokenStream, used: &mut BTreeSet<String>) {
    for tt in stream {
        match tt {
            TokenTree::Ident(ident) => {
                let raw = ident.to_string();
                let normalized = raw.strip_prefix("r#").unwrap_or(&raw).to_owned();
                used.insert(normalized);
            }
            TokenTree::Group(group) => collect_stream(group.stream(), used),
            TokenTree::Punct(_) | TokenTree::Literal(_) => {}
        }
    }
}

fn fresh_ident(base: &str, span: Span, used: &mut BTreeSet<String>) -> Ident {
    let mut candidate = base.to_owned();
    let mut suffix = 0usize;
    while used.contains(&candidate) {
        suffix += 1;
        candidate = format!("{base}_{suffix}");
    }
    used.insert(candidate.clone());
    Ident::new(&candidate, span)
}
