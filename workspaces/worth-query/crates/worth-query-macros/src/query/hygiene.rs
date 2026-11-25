//! Fresh def-site (`mixed_site`) identifiers for `#[derive(WorthQuery)]`.
//!
//! Walks every interpolated caller token stream (normalizing `r#` raw
//! identifiers) before picking fresh names for generated locals, witnesses,
//! aliases, and helper functions so outer `const`s, unit structs, or `__worth_*`
//! names can never shadow or collide.
use super::grammar::{QueryDeclaration, QueryScope};
use super::predicates::PredicateRhs;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
use std::collections::BTreeSet;
use syn::{DeriveInput, Ident};

pub(super) struct HygienicNames {
    pub(super) schema_alias: Ident,
    pub(super) result_alias: Ident,
    pub(super) root_alias: Ident,
    pub(super) bindings_alias: Ident,
    pub(super) index_alias: Ident,
    pub(super) params_local: Ident,
    pub(super) input_local: Ident,
    pub(super) value_local: Ident,
    pub(super) witness_local: Ident,
    pub(super) builder_local: Ident,
    pub(super) shape_local: Ident,
    pub(super) clauses_local: Ident,
    pub(super) related_local: Ident,
    pub(super) encode_lit_fn: Ident,
}

impl HygienicNames {
    pub(super) fn new(input: &DeriveInput, spec: &QueryDeclaration) -> Self {
        let mut used = collect_used_idents(input, spec);
        let mixed = Span::mixed_site();
        Self {
            schema_alias: fresh_ident("__WorthQuerySchema", mixed, &mut used),
            result_alias: fresh_ident("__WorthQueryResult", mixed, &mut used),
            root_alias: fresh_ident("__WorthQueryRoot", mixed, &mut used),
            bindings_alias: fresh_ident("__WorthQueryBindings", mixed, &mut used),
            index_alias: fresh_ident("__WorthQueryIndex", mixed, &mut used),
            params_local: fresh_ident("__worth_query_params", mixed, &mut used),
            input_local: fresh_ident("__worth_query_input", mixed, &mut used),
            value_local: fresh_ident("__worth_query_param_val", mixed, &mut used),
            witness_local: fresh_ident("__worth_query_witness", mixed, &mut used),
            builder_local: fresh_ident("__worth_query_builder", mixed, &mut used),
            shape_local: fresh_ident("__worth_query_shape", mixed, &mut used),
            clauses_local: fresh_ident("__worth_query_clauses", mixed, &mut used),
            related_local: fresh_ident("__worth_query_related", mixed, &mut used),
            encode_lit_fn: fresh_ident("__worth_query_encode_lit", mixed, &mut used),
        }
    }
}

fn collect_used_idents(input: &DeriveInput, spec: &QueryDeclaration) -> BTreeSet<String> {
    let mut used = BTreeSet::new();
    collect_stream(input.ident.to_token_stream(), &mut used);
    collect_stream(spec.schema.to_token_stream(), &mut used);
    collect_stream(spec.context.to_token_stream(), &mut used);
    collect_stream(spec.namespace.to_token_stream(), &mut used);
    collect_stream(spec.query.to_token_stream(), &mut used);
    collect_stream(spec.input.to_token_stream(), &mut used);
    collect_stream(spec.binding.to_token_stream(), &mut used);
    collect_stream(spec.result.to_token_stream(), &mut used);
    collect_stream(spec.limits.to_token_stream(), &mut used);
    if let Some(p) = &spec.parameters_binding {
        collect_stream(p.to_token_stream(), &mut used);
    }
    if let Some(p) = &spec.parameters_fn {
        collect_stream(p.to_token_stream(), &mut used);
    }
    if let Some(b) = &spec.bindings {
        collect_stream(b.to_token_stream(), &mut used);
    }
    if let Some(v) = &spec.validate {
        collect_stream(v.to_token_stream(), &mut used);
    }
    match &spec.scope {
        QueryScope::Field {
            member,
            value_ty,
            selection,
            ..
        } => {
            collect_stream(member.to_token_stream(), &mut used);
            collect_stream(value_ty.to_token_stream(), &mut used);
            collect_stream(selection.to_token_stream(), &mut used);
        }
        QueryScope::Principal { selection, .. } => {
            collect_stream(selection.to_token_stream(), &mut used);
        }
        QueryScope::Value {
            selection,
            function,
            ..
        } => {
            collect_stream(selection.to_token_stream(), &mut used);
            collect_stream(function.to_token_stream(), &mut used);
        }
    }
    for param in &spec.params {
        collect_stream(param.member.to_token_stream(), &mut used);
        collect_stream(param.value_ty.to_token_stream(), &mut used);
        collect_stream(param.selector.to_token_stream(), &mut used);
        if let Some(b) = &param.binding {
            collect_stream(b.to_token_stream(), &mut used);
        }
    }
    if let Some(mi) = &spec.managed_index {
        collect_stream(mi.index.to_token_stream(), &mut used);
    }
    if let Some(def) = &spec.definition {
        collect_stream(def.root.to_token_stream(), &mut used);
        for expr in [
            def.cardinality.as_ref(),
            def.disclosure.as_ref(),
            def.basis_support.as_ref(),
            def.lanes.as_ref(),
            def.ability.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            collect_stream(expr.to_token_stream(), &mut used);
        }
        for path in &def.root_paths {
            for hop in &path.hops {
                collect_stream(hop.relation.to_token_stream(), &mut used);
            }
        }
        for pred in &def.predicates {
            collect_stream(pred.field.to_token_stream(), &mut used);
            if let PredicateRhs::Literal(expr) = &pred.rhs {
                collect_stream(expr.to_token_stream(), &mut used);
            }
        }
        for rel in &def.related_predicates {
            collect_stream(rel.relation.to_token_stream(), &mut used);
            for pred in &rel.target_predicates {
                collect_stream(pred.field.to_token_stream(), &mut used);
                if let PredicateRhs::Literal(expr) = &pred.rhs {
                    collect_stream(expr.to_token_stream(), &mut used);
                }
            }
        }
        for ord in &def.order_by {
            collect_stream(ord.slot.to_token_stream(), &mut used);
        }
    }
    used
}

fn collect_stream(stream: TokenStream, out: &mut BTreeSet<String>) {
    for tree in stream {
        match tree {
            TokenTree::Ident(ident) => {
                let text = ident.to_string();
                let normalized = text.strip_prefix("r#").unwrap_or(&text);
                out.insert(normalized.to_owned());
            }
            TokenTree::Group(group) => collect_stream(group.stream(), out),
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
