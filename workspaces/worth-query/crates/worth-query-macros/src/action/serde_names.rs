//! Canonical serialized field names for `#[derive(WorthQueryAction)]` input accessors.
//!
//! Resolves `#[serde(rename = "...")]`, `#[serde(rename(serialize = "..."))]`,
//! `#[serde(rename_all = "...")]`, `#[serde(skip)]`, `#[serde(skip_serializing)]`,
//! and `#[serde(flatten)]` so `<field>_input()` always matches the serialized
//! member name captured by `RETAIN_CANONICAL_INPUT`, and refuses unsupported
//! container layout attributes rather than silently emitting a wrong key.
use crate::case::{pascal_case, screaming_snake_case, snake_case};
use crate::grammar::member_name;
use syn::{Attribute, Field, LitStr};

#[derive(Clone, Copy)]
pub(super) enum RenameRule {
    Lower,
    Upper,
    Pascal,
    Camel,
    Snake,
    ScreamingSnake,
    Kebab,
    ScreamingKebab,
}

impl RenameRule {
    fn parse(lit: &LitStr) -> syn::Result<Self> {
        match lit.value().as_str() {
            "lowercase" => Ok(Self::Lower),
            "UPPERCASE" => Ok(Self::Upper),
            "PascalCase" => Ok(Self::Pascal),
            "camelCase" => Ok(Self::Camel),
            "snake_case" => Ok(Self::Snake),
            "SCREAMING_SNAKE_CASE" => Ok(Self::ScreamingSnake),
            "kebab-case" => Ok(Self::Kebab),
            "SCREAMING-KEBAB-CASE" => Ok(Self::ScreamingKebab),
            other => Err(syn::Error::new(
                lit.span(),
                format!("unsupported `serde(rename_all = \"{other}\")` for `#[derive(WorthQueryAction)]`"),
            )),
        }
    }

    fn apply(self, name: &str) -> String {
        match self {
            Self::Lower => name.to_ascii_lowercase(),
            Self::Upper => name.to_ascii_uppercase(),
            Self::Pascal => pascal_case(name),
            Self::Camel => {
                let pascal = pascal_case(name);
                let mut chars = pascal.chars();
                match chars.next() {
                    Some(first) => first.to_ascii_lowercase().to_string() + chars.as_str(),
                    None => String::new(),
                }
            }
            Self::Snake => snake_case(name),
            Self::ScreamingSnake => screaming_snake_case(name),
            Self::Kebab => snake_case(name).replace('_', "-"),
            Self::ScreamingKebab => screaming_snake_case(name).replace('_', "-"),
        }
    }
}

pub(super) fn container_rename_rule(attrs: &[Attribute]) -> syn::Result<Option<RenameRule>> {
    let mut rule = None;
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename_all") {
                let lit = parse_rename_value(&meta)?;
                if let Some(lit) = lit {
                    rule = Some(RenameRule::parse(&lit)?);
                }
                return Ok(());
            }
            if meta.path.is_ident("transparent")
                || meta.path.is_ident("tag")
                || meta.path.is_ident("content")
                || meta.path.is_ident("untagged")
            {
                return Err(meta.error(
                    "`#[derive(WorthQueryAction)]` typed input accessors do not support \
                     `#[serde(transparent/tag/content/untagged)]` container rewrites",
                ));
            }
            consume_ignored(&meta)
        })?;
    }
    Ok(rule)
}

/// Returns `Ok(None)` when the field is skipped or flattened out of direct
/// serialized struct members.
pub(super) fn field_wire_name(
    field: &Field,
    container_rule: Option<RenameRule>,
) -> syn::Result<Option<LitStr>> {
    let member = field
        .ident
        .as_ref()
        .expect("named fields carry identifiers");
    let mut explicit: Option<LitStr> = None;
    let mut omitted = false;
    for attr in &field.attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip")
                || meta.path.is_ident("skip_serializing")
                || meta.path.is_ident("flatten")
            {
                omitted = true;
                return consume_ignored(&meta);
            }
            if meta.path.is_ident("rename") {
                if let Some(lit) = parse_rename_value(&meta)? {
                    explicit = Some(lit);
                }
                return Ok(());
            }
            consume_ignored(&meta)
        })?;
    }
    if omitted {
        return Ok(None);
    }
    if let Some(lit) = explicit {
        return Ok(Some(lit));
    }
    let raw = member_name(member);
    let wire = container_rule.map_or(raw.clone(), |rule| rule.apply(&raw));
    Ok(Some(LitStr::new(&wire, member.span())))
}

fn parse_rename_value(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<Option<LitStr>> {
    if meta.input.peek(syn::Token![=]) {
        return Ok(Some(meta.value()?.parse::<LitStr>()?));
    }
    let mut serialize = None;
    meta.parse_nested_meta(|nested| {
        if nested.path.is_ident("serialize") {
            serialize = Some(nested.value()?.parse::<LitStr>()?);
            Ok(())
        } else {
            consume_ignored(&nested)
        }
    })?;
    Ok(serialize)
}

fn consume_ignored(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<()> {
    if meta.input.peek(syn::Token![=]) {
        let _ = meta.value()?.parse::<proc_macro2::TokenStream>()?;
    } else if meta.input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in meta.input);
        let _ = content.parse::<proc_macro2::TokenStream>()?;
    }
    Ok(())
}
