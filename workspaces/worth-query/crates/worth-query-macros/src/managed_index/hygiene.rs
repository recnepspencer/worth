//! Fresh def-site (`mixed_site`) identifiers for `#[derive(WorthManagedIndex)]`.
use super::grammar::{ManagedIndexDeclaration, ProbeSpec};
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
use std::collections::BTreeSet;
use syn::{DeriveInput, Ident};

pub(super) struct HygienicNames {
    pub(super) index_alias: Ident,
    pub(super) schema_alias: Ident,
    pub(super) entity_alias: Ident,
    pub(super) source_alias: Ident,
    pub(super) probe_alias: Ident,
    pub(super) source_scope_const: Ident,
    pub(super) structure_const: Ident,
    pub(super) membership_const: Ident,
    pub(super) maintenance_const: Ident,
    pub(super) refinement_const: Ident,
}

impl HygienicNames {
    pub(super) fn new(input: &DeriveInput, spec: &ManagedIndexDeclaration) -> Self {
        let mut used = collect_used_idents(input, spec);
        let mixed = Span::mixed_site();
        Self {
            index_alias: fresh_ident("__WorthManagedIndex", mixed, &mut used),
            schema_alias: fresh_ident("__WorthManagedIndexSchema", mixed, &mut used),
            entity_alias: fresh_ident("__WorthManagedIndexEntity", mixed, &mut used),
            source_alias: fresh_ident("__WorthManagedIndexSource", mixed, &mut used),
            probe_alias: fresh_ident("__WorthManagedIndexProbe", mixed, &mut used),
            source_scope_const: fresh_ident("__WORTH_MANAGED_INDEX_SOURCE_SCOPE", mixed, &mut used),
            structure_const: fresh_ident("__WORTH_MANAGED_INDEX_STRUCTURE", mixed, &mut used),
            membership_const: fresh_ident("__WORTH_MANAGED_INDEX_MEMBERSHIP", mixed, &mut used),
            maintenance_const: fresh_ident("__WORTH_MANAGED_INDEX_MAINTENANCE", mixed, &mut used),
            refinement_const: fresh_ident("__WORTH_MANAGED_INDEX_REFINEMENT", mixed, &mut used),
        }
    }
}

fn collect_used_idents(input: &DeriveInput, spec: &ManagedIndexDeclaration) -> BTreeSet<String> {
    let mut used = BTreeSet::new();
    collect_stream_idents(input.ident.to_token_stream(), &mut used);
    collect_stream_idents(spec.schema.to_token_stream(), &mut used);
    collect_stream_idents(spec.entity.to_token_stream(), &mut used);
    collect_stream_idents(spec.source.to_token_stream(), &mut used);
    match &spec.probe {
        ProbeSpec::ValueType(ty) | ProbeSpec::BindingType(ty) => {
            collect_stream_idents(ty.to_token_stream(), &mut used);
        }
    }
    for expr in [
        spec.source_scope.as_ref(),
        spec.structure.as_ref(),
        spec.membership.as_ref(),
        spec.maintenance.as_ref(),
        spec.refinement.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        collect_stream_idents(expr.to_token_stream(), &mut used);
    }
    used
}

fn collect_stream_idents(stream: TokenStream, out: &mut BTreeSet<String>) {
    for tree in stream {
        match tree {
            TokenTree::Ident(ident) => {
                let text = ident.to_string();
                let normalized = text.strip_prefix("r#").unwrap_or(&text);
                out.insert(normalized.to_owned());
            }
            TokenTree::Group(group) => collect_stream_idents(group.stream(), out),
            TokenTree::Punct(_) | TokenTree::Literal(_) => {}
        }
    }
}

fn fresh_ident(base: &str, span: Span, used: &mut BTreeSet<String>) -> Ident {
    if used.insert(base.to_owned()) {
        return Ident::new(base, span);
    }
    let mut suffix = 1usize;
    loop {
        let candidate = format!("{base}_{suffix}");
        if used.insert(candidate.clone()) {
            return Ident::new(&candidate, span);
        }
        suffix += 1;
    }
}
