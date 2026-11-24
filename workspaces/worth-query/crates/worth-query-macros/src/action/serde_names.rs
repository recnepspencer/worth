//! Canonical serialized field names for `#[derive(WorthQueryAction)]` input accessors.
//!
//! Resolves `#[serde(rename = "...")]`, `#[serde(rename(serialize = "..."))]`,
//! `#[serde(rename_all = "...")]`, `#[serde(skip)]`, `#[serde(skip_serializing)]`,
//! and `#[serde(flatten)]` matching Serde's exact `RenameRule::apply_to_field`
//! behavior, consumes ignored sibling attributes without swallowing subsequent
//! items, and refuses container layout rewrites (`transparent`, `tag`, `content`,
//! `untagged`, `into`, `from`, `try_from`).
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
                format!(
                    "unsupported `serde(rename_all = \"{other}\")` for `#[derive(WorthQueryAction)]`"
                ),
            )),
        }
    }

    /// Matches `serde_derive_internals::attr::RenameRule::apply_to_field`.
    fn apply_to_field(self, field: &str) -> String {
        match self {
            Self::Lower | Self::Snake => field.to_owned(),
            Self::Upper => field.to_ascii_uppercase(),
            Self::Pascal => {
                let mut pascal = String::new();
                let mut capitalize = true;
                for ch in field.chars() {
                    if ch == '_' {
                        capitalize = true;
                    } else if capitalize {
                        pascal.push(ch.to_ascii_uppercase());
                        capitalize = false;
                    } else {
                        pascal.push(ch);
                    }
                }
                pascal
            }
            Self::Camel => {
                let pascal = Self::Pascal.apply_to_field(field);
                let mut chars = pascal.chars();
                match chars.next() {
                    Some(first) => first.to_ascii_lowercase().to_string() + chars.as_str(),
                    None => String::new(),
                }
            }
            Self::ScreamingSnake => field.to_ascii_uppercase(),
            Self::Kebab => field.replace('_', "-"),
            Self::ScreamingKebab => field.to_ascii_uppercase().replace('_', "-"),
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
                if let Some(lit) = parse_rename_value(&meta)? {
                    rule = Some(RenameRule::parse(&lit)?);
                }
                return Ok(());
            }
            if meta.path.is_ident("transparent")
                || meta.path.is_ident("tag")
                || meta.path.is_ident("content")
                || meta.path.is_ident("untagged")
                || meta.path.is_ident("into")
                || meta.path.is_ident("from")
                || meta.path.is_ident("try_from")
            {
                return Err(meta.error(
                    "`#[derive(WorthQueryAction)]` typed input accessors do not support \
                     `#[serde(transparent/tag/content/untagged/into/from/try_from)]` container rewrites",
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
    let wire = container_rule.map_or(raw.clone(), |rule| rule.apply_to_field(&raw));
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

/// Consumes only the value belonging to the current `#[serde(...)]` item so
/// sibling attributes after `,` remain intact for `parse_nested_meta`.
fn consume_ignored(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<()> {
    if meta.input.peek(syn::Token![=]) {
        let _ = meta.value()?.parse::<syn::Expr>()?;
    } else if meta.input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in meta.input);
        let _ = content.parse::<proc_macro2::TokenStream>()?;
    }
    Ok(())
}
