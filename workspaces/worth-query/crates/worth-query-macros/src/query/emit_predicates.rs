//! Lowering of `#[derive(WorthQuery)]` comparison and related-record predicates.
use super::predicates::{
    ComparisonOp, PredicateRhs, QueryPredicate, QueryRelatedPredicate, TraversalDirection,
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Expr, Ident, Lit};

pub(super) fn partition_predicates<'a>(
    predicates: &'a [QueryPredicate],
    encode_lit_fn: &Ident,
    eq_steps: &mut Vec<TokenStream>,
    filter_preds: &mut Vec<&'a QueryPredicate>,
) {
    for pred in predicates {
        if pred.op == ComparisonOp::Equal {
            match &pred.rhs {
                PredicateRhs::Parameter {
                    selector,
                    optional: false,
                } => {
                    let field = &pred.field;
                    eq_steps
                        .push(quote_spanned!(pred.span=> .where_equal(#field, Self::#selector())));
                }
                PredicateRhs::Literal(expr) => {
                    let field = &pred.field;
                    let lit = literal_value_expr(expr);
                    eq_steps.push(quote_spanned!(pred.span=> .where_equal_value(#field, #encode_lit_fn(&(#field), #lit))));
                }
                PredicateRhs::Parameter { optional: true, .. } => filter_preds.push(pred),
            }
        } else {
            filter_preds.push(pred);
        }
    }
}

pub(super) fn emit_filter_block(
    filter_preds: &[&QueryPredicate],
    target_local: &Ident,
    clauses_local: &Ident,
    encode_lit_fn: &Ident,
    aq: &TokenStream,
) -> TokenStream {
    if filter_preds.is_empty() {
        return TokenStream::new();
    }
    let clause_calls = filter_preds
        .iter()
        .map(|pred| emit_clause_call(pred, clauses_local, encode_lit_fn, aq));
    quote! {
        let mut #clauses_local = #aq::ApplicationQueryWhereFilterClauses::<Self>::new();
        #(#clause_calls)*
        #target_local = #target_local.where_filter(
            #clauses_local
                .finish()
                .expect("derived WorthQuery where_filter clauses admit"),
        );
    }
}

pub(super) fn emit_related_predicate(
    rel: &QueryRelatedPredicate,
    builder_local: &Ident,
    related_local: &Ident,
    clauses_local: &Ident,
    encode_lit_fn: &Ident,
    aq: &TokenStream,
) -> TokenStream {
    let relation = &rel.relation;
    let dir_ctor = match rel.direction {
        TraversalDirection::Forward => quote!(forward),
        TraversalDirection::Reverse => quote!(reverse),
    };
    let method = if rel.exists {
        quote!(where_related_exists)
    } else {
        quote!(where_no_related)
    };
    let (mut eq_steps, mut filter_preds) = (Vec::new(), Vec::new());
    partition_predicates(
        &rel.target_predicates,
        encode_lit_fn,
        &mut eq_steps,
        &mut filter_preds,
    );
    let filter_block = emit_filter_block(
        &filter_preds,
        related_local,
        clauses_local,
        encode_lit_fn,
        aq,
    );
    quote_spanned! {rel.span=> {
        #[allow(unused_mut)]
        let mut #related_local = #aq::ApplicationQueryRelatedPredicate::#dir_ctor(<#relation>::reference()) #(#eq_steps)*;
        #filter_block
        #builder_local = #builder_local.#method(#related_local);
    }}
}

fn emit_clause_call(
    pred: &QueryPredicate,
    clauses_local: &Ident,
    encode_lit_fn: &Ident,
    aq: &TokenStream,
) -> TokenStream {
    let field = &pred.field;
    let op_ident = format_ident!("{}", pred.op.variant_ident(), span = Span::mixed_site());
    match &pred.rhs {
        PredicateRhs::Parameter { selector, .. } => quote_spanned! {pred.span=>
            #clauses_local = #clauses_local
                .compare_parameter(
                    #aq::ApplicationQueryComparisonOp::#op_ident,
                    #field,
                    Self::#selector(),
                )
                .expect("derived WorthQuery parameter clause admits");
        },
        PredicateRhs::Literal(expr) => {
            let lit = literal_value_expr(expr);
            quote_spanned! {pred.span=>
                #clauses_local = #clauses_local
                    .compare_literal(
                        #aq::ApplicationQueryComparisonOp::#op_ident,
                        #field,
                        #encode_lit_fn(&(#field), #lit),
                    )
                    .expect("derived WorthQuery literal clause admits");
            }
        }
    }
}

fn literal_value_expr(expr: &Expr) -> TokenStream {
    if let Expr::Lit(lit) = expr
        && matches!(lit.lit, Lit::Str(_))
    {
        return quote_spanned!(expr.span()=> ::std::string::String::from(#expr));
    }
    quote_spanned!(expr.span()=> #expr)
}
