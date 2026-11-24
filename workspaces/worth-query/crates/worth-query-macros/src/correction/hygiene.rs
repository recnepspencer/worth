//! Fresh def-site (`mixed_site`) identifiers for `#[derive(WorthQueryCorrection)]`.
//!
//! Ensures generated type aliases, outer callback helper names, and the
//! `visit` generic parameter never collide with any caller identifier, while
//! parameter/local names are isolated inside an inner `const _: ()` block.
use super::grammar::CorrectionDeclaration;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
use std::collections::BTreeSet;
use syn::{DeriveInput, Ident};

pub(super) struct HygienicNames {
    pub(super) schema_alias: Ident,
    pub(super) source_alias: Ident,
    pub(super) inverse_alias: Ident,
    pub(super) disclosure_alias: Ident,
    pub(super) cap_alias: Ident,
    pub(super) call_disclosure: Ident,
    pub(super) call_source_def: Ident,
    pub(super) call_source_body: Ident,
    pub(super) call_demand: Ident,
    pub(super) call_derive: Ident,
    pub(super) call_inverse_key: Ident,
    pub(super) bytes: Ident,
    pub(super) expected: Ident,
    pub(super) bounds: Ident,
    pub(super) original: Ident,
    pub(super) material: Ident,
    pub(super) preimage: Ident,
    pub(super) key: Ident,
    pub(super) dispatch: Ident,
    pub(super) visitor: Ident,
}

impl HygienicNames {
    pub(super) fn new(input: &DeriveInput, spec: &CorrectionDeclaration) -> Self {
        let mut used = collect_used_idents(input, spec);
        let mixed = Span::mixed_site();
        Self {
            schema_alias: fresh_ident("__WorthCorrectionSchema", mixed, &mut used),
            source_alias: fresh_ident("__WorthCorrectionSource", mixed, &mut used),
            inverse_alias: fresh_ident("__WorthCorrectionInverse", mixed, &mut used),
            disclosure_alias: fresh_ident("__WorthCorrectionDisclosure", mixed, &mut used),
            cap_alias: fresh_ident("__WorthCorrectionCapability", mixed, &mut used),
            call_disclosure: fresh_ident("__worth_correction_call_disclosure", mixed, &mut used),
            call_source_def: fresh_ident("__worth_correction_call_source_def", mixed, &mut used),
            call_source_body: fresh_ident("__worth_correction_call_source_body", mixed, &mut used),
            call_demand: fresh_ident("__worth_correction_call_demand", mixed, &mut used),
            call_derive: fresh_ident("__worth_correction_call_derive", mixed, &mut used),
            call_inverse_key: fresh_ident("__worth_correction_call_inverse_key", mixed, &mut used),
            bytes: Ident::new("__worth_correction_bytes", mixed),
            expected: Ident::new("__worth_correction_expected", mixed),
            bounds: Ident::new("__worth_correction_bounds", mixed),
            original: Ident::new("__worth_correction_original", mixed),
            material: Ident::new("__worth_correction_material", mixed),
            preimage: Ident::new("__worth_correction_preimage", mixed),
            key: Ident::new("__worth_correction_key", mixed),
            dispatch: Ident::new("__worth_correction_dispatch", mixed),
            visitor: fresh_ident("__WorthCorrectionVisitor", mixed, &mut used),
        }
    }
}

fn collect_used_idents(input: &DeriveInput, spec: &CorrectionDeclaration) -> BTreeSet<String> {
    let mut used = BTreeSet::new();
    collect_stream_idents(input.ident.to_token_stream(), &mut used);
    collect_stream_idents(spec.schema.to_token_stream(), &mut used);
    collect_stream_idents(spec.source.to_token_stream(), &mut used);
    collect_stream_idents(spec.inverse.to_token_stream(), &mut used);
    collect_stream_idents(spec.disclosure.to_token_stream(), &mut used);
    collect_stream_idents(spec.derive_fn.to_token_stream(), &mut used);
    if let Some(cap) = &spec.capability {
        collect_stream_idents(cap.to_token_stream(), &mut used);
    }
    if let Some(p) = &spec.disclosure_parameters {
        collect_stream_idents(p.to_token_stream(), &mut used);
    }
    if let Some(p) = &spec.source_definition {
        collect_stream_idents(p.to_token_stream(), &mut used);
    }
    if let Some(p) = &spec.source_body {
        collect_stream_idents(p.to_token_stream(), &mut used);
    }
    if let Some(p) = &spec.inverse_key {
        collect_stream_idents(p.to_token_stream(), &mut used);
    }
    used
}

fn collect_stream_idents(stream: TokenStream, out: &mut BTreeSet<String>) {
    for tree in stream {
        match tree {
            TokenTree::Ident(ident) => {
                out.insert(ident.to_string());
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
