//! Resolves parsed predicate expressions against declared query parameters and
//! optional aspect-marker shorthand.
use super::{
    bare_expr_ident, PredicateRhs, QueryPredicate, QueryRelatedPredicate, RawPredicate,
    RawRelatedPredicate,
};
use crate::grammar::member_name;
use crate::query::grammar::QueryParam;
use syn::ext::IdentExt;
use syn::{Expr, Path};

pub(in crate::query) fn resolve_predicates(
    raw: Vec<RawPredicate>,
    fields_aspect: Option<&Path>,
    params: &[QueryParam],
) -> syn::Result<Vec<QueryPredicate>> {
    raw.into_iter()
        .map(|pred| resolve_predicate(pred, fields_aspect, params))
        .collect()
}

pub(in crate::query) fn resolve_related_predicates(
    raw: Vec<RawRelatedPredicate>,
    default_fields: Option<&Path>,
    params: &[QueryParam],
) -> syn::Result<Vec<QueryRelatedPredicate>> {
    raw.into_iter()
        .map(|rel| {
            let aspect = rel.target_fields.as_ref().or(default_fields);
            let target_predicates = resolve_predicates(rel.target_predicates, aspect, params)?;
            Ok(QueryRelatedPredicate {
                exists: rel.exists,
                relation: rel.relation,
                direction: rel.direction,
                target_predicates,
                span: rel.span,
            })
        })
        .collect()
}

fn resolve_predicate(
    raw: RawPredicate,
    fields_aspect: Option<&Path>,
    params: &[QueryParam],
) -> syn::Result<QueryPredicate> {
    let field = expand_field_expr(raw.field, fields_aspect);
    let matched_param = bare_expr_ident(&raw.rhs).and_then(|ident| {
        let name = ident.unraw().to_string();
        params
            .iter()
            .find(|p| member_name(&p.member) == name || member_name(&p.selector) == name)
    });
    let rhs = match (matched_param, raw.optional_param) {
        (Some(param), true) => {
            if param.optional_inner.is_none() {
                return Err(syn::Error::new_spanned(
                    &raw.rhs,
                    format!(
                        "`where_*_optional` requires an `Option<T>` parameter field, but `{}` is required",
                        member_name(&param.member)
                    ),
                ));
            }
            PredicateRhs::Parameter {
                selector: param.selector.clone(),
                optional: true,
            }
        }
        (Some(param), false) => {
            if param.optional_inner.is_some() {
                return Err(syn::Error::new_spanned(
                    &raw.rhs,
                    format!(
                        "optional parameter `{}` requires a `where_*_optional` predicate",
                        member_name(&param.member)
                    ),
                ));
            }
            PredicateRhs::Parameter {
                selector: param.selector.clone(),
                optional: false,
            }
        }
        (None, true) => {
            return Err(syn::Error::new_spanned(
                &raw.rhs,
                "`where_*_optional` requires the name of an `Option<T>` parameter field on this struct",
            ));
        }
        (None, false) => PredicateRhs::Literal(raw.rhs),
    };
    Ok(QueryPredicate {
        op: raw.op,
        field,
        rhs,
        span: raw.span,
    })
}

fn expand_field_expr(field: Expr, aspect: Option<&Path>) -> Expr {
    if let (Some(aspect), Some(ident)) = (aspect, bare_expr_ident(&field)) {
        syn::parse_quote!(#aspect::#ident())
    } else {
        field
    }
}
