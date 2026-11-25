//! Predicate, root-path, ordering, and live-mode parsing for `#[derive(WorthQuery)]`.
use proc_macro2::{Span, TokenStream};
use syn::ext::IdentExt;
use syn::meta::ParseNestedMeta;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Expr, Ident, Path, Token, Type};

mod plan_clauses;
mod resolve;
pub(super) use plan_clauses::{
    parse_live_mode, parse_order_by, parse_root_path, LiveMode, OrderBySpec, OrderDirection,
    RootPathSpec,
};
pub(super) use resolve::{resolve_predicates, resolve_related_predicates};

/// Comparison operator for struct-level and related-target query predicates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ComparisonOp {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

impl ComparisonOp {
    pub(super) fn variant_ident(self) -> &'static str {
        match self {
            Self::Equal => "Equal",
            Self::NotEqual => "NotEqual",
            Self::LessThan => "LessThan",
            Self::LessThanOrEqual => "LessThanOrEqual",
            Self::GreaterThan => "GreaterThan",
            Self::GreaterThanOrEqual => "GreaterThanOrEqual",
        }
    }
}

/// Direction of a relation traversal (`outgoing`/`forward` vs `incoming`/`reverse`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TraversalDirection {
    Forward,
    Reverse,
}

/// Raw parsed `field = rhs` comparison before binding parameter names.
pub(super) struct RawPredicate {
    pub(super) op: ComparisonOp,
    pub(super) optional_param: bool,
    pub(super) field: Expr,
    pub(super) rhs: Expr,
    pub(super) span: Span,
}

/// Resolved comparison predicate referencing either a query parameter or a literal.
pub(super) struct QueryPredicate {
    pub(super) op: ComparisonOp,
    pub(super) field: Expr,
    pub(super) rhs: PredicateRhs,
    pub(super) span: Span,
}

pub(super) enum PredicateRhs {
    Parameter { selector: Ident, optional: bool },
    Literal(Expr),
}

/// Parsed `where_no_related(...)` or `where_related_exists(...)` clause.
pub(super) struct RawRelatedPredicate {
    pub(super) exists: bool,
    pub(super) relation: Type,
    pub(super) direction: TraversalDirection,
    pub(super) target_fields: Option<Path>,
    pub(super) target_predicates: Vec<RawPredicate>,
    pub(super) span: Span,
}

pub(super) struct QueryRelatedPredicate {
    pub(super) exists: bool,
    pub(super) relation: Type,
    pub(super) direction: TraversalDirection,
    pub(super) target_predicates: Vec<QueryPredicate>,
    pub(super) span: Span,
}

pub(super) fn predicate_key(key: &str) -> Option<(ComparisonOp, bool)> {
    Some(match key {
        "where_equal" => (ComparisonOp::Equal, false),
        "where_not_equal" => (ComparisonOp::NotEqual, false),
        "where_lt" => (ComparisonOp::LessThan, false),
        "where_lte" => (ComparisonOp::LessThanOrEqual, false),
        "where_gt" => (ComparisonOp::GreaterThan, false),
        "where_gte" => (ComparisonOp::GreaterThanOrEqual, false),
        "where_optional" | "where_equal_optional" => (ComparisonOp::Equal, true),
        "where_not_equal_optional" => (ComparisonOp::NotEqual, true),
        "where_lt_optional" => (ComparisonOp::LessThan, true),
        "where_lte_optional" => (ComparisonOp::LessThanOrEqual, true),
        "where_gt_optional" => (ComparisonOp::GreaterThan, true),
        "where_gte_optional" => (ComparisonOp::GreaterThanOrEqual, true),
        _ => return None,
    })
}

pub(super) fn parse_predicate_list(
    meta: &ParseNestedMeta<'_>,
    op: ComparisonOp,
    optional_param: bool,
) -> syn::Result<Vec<RawPredicate>> {
    let content;
    syn::parenthesized!(content in meta.input);
    parse_assignments(&content, op, optional_param, meta.path.span())
}

struct Assignment {
    field: Expr,
    rhs: Expr,
    span: Span,
}

impl Parse for Assignment {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut lhs_tokens = TokenStream::new();
        while !input.is_empty() && !input.peek(Token![=]) && !input.peek(Token![,]) {
            let tt: proc_macro2::TokenTree = input.parse()?;
            lhs_tokens.extend(std::iter::once(tt));
        }
        if !input.peek(Token![=]) {
            return Err(input.error("expected `field = value` predicate"));
        }
        let eq: Token![=] = input.parse()?;
        let field: Expr = syn::parse2(lhs_tokens)?;
        let rhs: Expr = input.parse()?;
        Ok(Self {
            field,
            rhs,
            span: eq.span,
        })
    }
}

fn parse_assignments(
    input: ParseStream<'_>,
    op: ComparisonOp,
    optional_param: bool,
    span: Span,
) -> syn::Result<Vec<RawPredicate>> {
    let items = Punctuated::<Assignment, Token![,]>::parse_terminated(input)?;
    if items.is_empty() {
        return Err(syn::Error::new(
            span,
            "predicate clause requires at least one `field = value` entry",
        ));
    }
    Ok(items
        .into_iter()
        .map(|item| RawPredicate {
            op,
            optional_param,
            field: item.field,
            rhs: item.rhs,
            span: item.span,
        })
        .collect())
}

pub(super) fn parse_related_predicate(
    meta: &ParseNestedMeta<'_>,
    exists: bool,
) -> syn::Result<RawRelatedPredicate> {
    let content;
    syn::parenthesized!(content in meta.input);
    let span = meta.path.span();
    if content.is_empty() {
        return Err(syn::Error::new(
            span,
            "related predicate requires a relation type, such as `where_no_related(BlockedBy)`",
        ));
    }
    let relation: Type = content.parse()?;
    let mut direction: Option<TraversalDirection> = None;
    let mut target_fields: Option<Path> = None;
    let mut target_predicates = Vec::new();
    while !content.is_empty() {
        let _: Token![,] = content.parse()?;
        if content.is_empty() {
            break;
        }
        if content.peek(Ident) && (content.peek2(Token![,]) || content.peek2(syn::parse::End)) {
            let dir_ident: Ident = content.parse()?;
            let parsed = parse_direction_ident(&dir_ident)?;
            if direction.replace(parsed).is_some() {
                return Err(syn::Error::new(
                    dir_ident.span(),
                    "duplicate direction in related predicate",
                ));
            }
            continue;
        }
        if content.peek(Ident) && content.peek2(syn::token::Paren) {
            let key_ident: Ident = content.fork().parse()?;
            if let Some((op, optional_param)) = predicate_key(&key_ident.unraw().to_string()) {
                let _: Ident = content.parse()?;
                let inner;
                syn::parenthesized!(inner in content);
                target_predicates.extend(parse_assignments(
                    &inner,
                    op,
                    optional_param,
                    key_ident.span(),
                )?);
                continue;
            }
        }
        let assign: Assignment = content.parse()?;
        if let Some(ident) = bare_expr_ident(&assign.field) {
            let name = ident.unraw().to_string();
            if name == "target_fields" {
                let Expr::Path(path) = assign.rhs else {
                    return Err(syn::Error::new_spanned(
                        assign.rhs,
                        "`target_fields` expects an aspect marker path",
                    ));
                };
                if target_fields.replace(path.path).is_some() {
                    return Err(syn::Error::new(
                        ident.span(),
                        "duplicate `target_fields` in related predicate",
                    ));
                }
                continue;
            }
            if name == "direction" {
                let Some(dir_ident) = bare_expr_ident(&assign.rhs) else {
                    return Err(syn::Error::new_spanned(
                        assign.rhs,
                        "`direction` expects `outgoing` or `incoming`",
                    ));
                };
                let parsed = parse_direction_ident(dir_ident)?;
                if direction.replace(parsed).is_some() {
                    return Err(syn::Error::new(
                        ident.span(),
                        "duplicate direction in related predicate",
                    ));
                }
                continue;
            }
        }
        target_predicates.push(RawPredicate {
            op: ComparisonOp::Equal,
            optional_param: false,
            field: assign.field,
            rhs: assign.rhs,
            span: assign.span,
        });
    }
    Ok(RawRelatedPredicate {
        exists,
        relation,
        direction: direction.unwrap_or(TraversalDirection::Forward),
        target_fields,
        target_predicates,
        span,
    })
}

pub(super) fn parse_direction_ident(ident: &Ident) -> syn::Result<TraversalDirection> {
    match ident.unraw().to_string().as_str() {
        "outgoing" | "forward" => Ok(TraversalDirection::Forward),
        "incoming" | "reverse" => Ok(TraversalDirection::Reverse),
        other => Err(syn::Error::new(
            ident.span(),
            format!("unknown direction `{other}`; expected `outgoing` or `incoming`"),
        )),
    }
}

pub(super) fn bare_expr_ident(expr: &Expr) -> Option<&Ident> {
    let Expr::Path(path) = expr else {
        return None;
    };
    if path.qself.is_none() && path.path.leading_colon.is_none() && path.path.segments.len() == 1 {
        let seg = &path.path.segments[0];
        if seg.arguments.is_empty() {
            return Some(&seg.ident);
        }
    }
    None
}
