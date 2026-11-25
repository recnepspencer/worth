//! Root-path, ordering, and live-mode clause parsing for `#[derive(WorthQuery)]`.
use super::{parse_direction_ident, TraversalDirection};
use proc_macro2::Span;
use syn::ext::IdentExt;
use syn::meta::ParseNestedMeta;
use syn::spanned::Spanned;
use syn::{Expr, Ident, Token, Type};

pub(in crate::query) struct RootPathHop {
    pub(in crate::query) relation: Type,
    pub(in crate::query) direction: TraversalDirection,
}

pub(in crate::query) struct RootPathSpec {
    pub(in crate::query) hops: Vec<RootPathHop>,
    pub(in crate::query) span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::query) enum OrderDirection {
    Ascending,
    Descending,
}

pub(in crate::query) struct OrderBySpec {
    pub(in crate::query) slot: Expr,
    pub(in crate::query) direction: OrderDirection,
    pub(in crate::query) span: Span,
}

pub(in crate::query) enum LiveMode {
    WholeResult,
    Effect(Box<super::super::live_effect::LiveEffect>),
}

pub(in crate::query) fn parse_live_mode(meta: &ParseNestedMeta<'_>) -> syn::Result<LiveMode> {
    let ident: Ident = meta.value()?.parse()?;
    match ident.unraw().to_string().as_str() {
        "whole_result" => Ok(LiveMode::WholeResult),
        "effect" => {
            let content;
            syn::parenthesized!(content in meta.input);
            super::super::live_effect::LiveEffect::parse(&content)
                .map(|live| LiveMode::Effect(Box::new(live)))
        }
        other => Err(syn::Error::new(
            ident.span(),
            format!("unknown `live = {other}` mode; expected `whole_result` or `effect(..)`"),
        )),
    }
}

pub(in crate::query) fn parse_root_path(meta: &ParseNestedMeta<'_>) -> syn::Result<RootPathSpec> {
    let content;
    syn::parenthesized!(content in meta.input);
    let span = meta.path.span();
    let mut hops = Vec::new();
    while !content.is_empty() {
        let (direction, relation) = if content.peek(Ident) && content.peek2(syn::token::Paren) {
            let dir_ident: Ident = content.parse()?;
            let inner;
            syn::parenthesized!(inner in content);
            let rel: Type = inner.parse()?;
            if !inner.is_empty() {
                return Err(inner.error("expected a single relation type inside hop"));
            }
            (parse_direction_ident(&dir_ident)?, rel)
        } else {
            (TraversalDirection::Forward, content.parse()?)
        };
        hops.push(RootPathHop {
            relation,
            direction,
        });
        if content.is_empty() {
            break;
        }
        let _: Token![,] = content.parse()?;
    }
    if hops.is_empty() {
        return Err(syn::Error::new(
            span,
            "`root_path(..)` requires at least one relation hop",
        ));
    }
    Ok(RootPathSpec { hops, span })
}

pub(in crate::query) fn parse_order_by(meta: &ParseNestedMeta<'_>) -> syn::Result<OrderBySpec> {
    let content;
    syn::parenthesized!(content in meta.input);
    let span = meta.path.span();
    if content.is_empty() {
        return Err(syn::Error::new(
            span,
            "`order_by(..)` requires a result field selector, such as `order_by(TaskRow::key(), asc)`",
        ));
    }
    let slot: Expr = content.parse()?;
    let mut direction = OrderDirection::Ascending;
    if content.peek(Token![,]) {
        let _: Token![,] = content.parse()?;
        if !content.is_empty() {
            let dir: Ident = content.parse()?;
            direction = match dir.unraw().to_string().as_str() {
                "asc" | "Ascending" => OrderDirection::Ascending,
                "desc" | "Descending" => OrderDirection::Descending,
                other => {
                    return Err(syn::Error::new(
                        dir.span(),
                        format!("unknown ordering direction `{other}`; expected `asc` or `desc`"),
                    ))
                }
            };
            if content.peek(Token![,]) {
                let _: Token![,] = content.parse()?;
            }
            if !content.is_empty() {
                return Err(content.error("unexpected extra token in `order_by(..)`"));
            }
        }
    }
    Ok(OrderBySpec {
        slot,
        direction,
        span,
    })
}
