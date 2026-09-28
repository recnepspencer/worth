//! Decide whether one resolved definition carries rustdoc, and where a missing
//! `///` comment belongs.
//!
//! Accepted: `///`, `//!`, `#[doc = ...]`, and `cfg_attr(..., doc = ...)`. A
//! module counts its declaration's outer docs and its body's inner docs. A
//! macro-generated item counts docs on or inside its invocation and docs
//! inside its `macro_rules!` template. `#[doc(hidden)]` is not documentation.

use super::resolution::item_shape::{attributes, defining_line, kind_label};
use super::resolution::{Definition, Resolver};
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use syn::{Attribute, Expr, Item, Lit, Meta};

/// Where a definition lacking rustdoc lives, for the contributor to fix.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct MissingDoc {
    pub(crate) kind: &'static str,
    /// Repository-relative `file:line` of the defining keyword.
    pub(crate) site: String,
}

/// `None` when documented or defined outside the workspace.
pub(crate) fn missing_doc(resolver: &Resolver<'_>, definition: &Definition) -> Option<MissingDoc> {
    let crates = resolver.crates();
    match definition {
        Definition::OutsideWorkspace(_) => None,
        Definition::Module { krate, path } => {
            let indexed = crates.crate_at(*krate);
            let node = &indexed.graph.modules[path];
            if carries_doc(&node.attributes) {
                return None;
            }
            let Some((name, parent_path)) = path.split_last() else {
                let site = format!(
                    "{}/{}:1",
                    indexed.repo_relative_root,
                    first(&node.relative_source)
                );
                return Some(MissingDoc {
                    kind: "crate root",
                    site,
                });
            };
            let parent = &indexed.graph.modules[parent_path];
            let declarations = parent
                .items
                .iter()
                .filter(|item| matches!(item, Item::Mod(module) if module.ident == name))
                .collect::<Vec<_>>();
            if declarations
                .iter()
                .any(|item| carries_doc(attributes(item)))
            {
                return None;
            }
            let line = declarations.first().map_or(1, |item| defining_line(item));
            Some(missing(
                "module",
                &indexed.repo_relative_root,
                &parent.relative_source,
                line,
            ))
        }
        Definition::Item {
            krate,
            module,
            index,
        } => {
            let indexed = crates.crate_at(*krate);
            let node = &indexed.graph.modules[module];
            let item = &node.items[*index];
            (!carries_doc(attributes(item))).then(|| {
                missing(
                    kind_label(item),
                    &indexed.repo_relative_root,
                    &node.relative_source,
                    defining_line(item),
                )
            })
        }
        Definition::Variant {
            krate,
            module,
            index,
            variant,
        } => {
            let indexed = crates.crate_at(*krate);
            let node = &indexed.graph.modules[module];
            let Item::Enum(item) = &node.items[*index] else {
                return None;
            };
            let found = item.variants.iter().find(|entry| entry.ident == variant)?;
            (!carries_doc(&found.attrs)).then(|| {
                missing(
                    "enum variant",
                    &indexed.repo_relative_root,
                    &node.relative_source,
                    found.ident.span().start().line,
                )
            })
        }
        Definition::Generated {
            krate,
            module,
            invocation,
            template,
        } => {
            let indexed = crates.crate_at(*krate);
            let node = &indexed.graph.modules[module];
            let Item::Macro(call) = &node.items[*invocation] else {
                return None;
            };
            let documented = carries_doc(&call.attrs)
                || contains_doc_attribute(&call.mac.tokens)
                || contains_doc_attribute(&resolver.template_tokens(template));
            (!documented).then(|| {
                missing(
                    "macro-generated item",
                    &indexed.repo_relative_root,
                    &node.relative_source,
                    defining_line(&node.items[*invocation]),
                )
            })
        }
    }
}

pub(crate) fn carries_doc(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| match &attribute.meta {
        Meta::NameValue(value) if attribute.path().is_ident("doc") => match &value.value {
            Expr::Lit(expr) => match &expr.lit {
                Lit::Str(text) => !text.value().trim().is_empty(),
                _ => false,
            },
            _ => true,
        },
        Meta::List(list) if attribute.path().is_ident("cfg_attr") => {
            let tokens = list.tokens.clone().into_iter().collect::<Vec<_>>();
            tokens.windows(2).any(|pair| {
                matches!(&pair[0], TokenTree::Ident(ident) if ident == "doc")
                    && matches!(&pair[1], TokenTree::Punct(punct) if punct.as_char() == '=')
            })
        }
        _ => false,
    })
}

/// Whether a token stream spells `#[doc ...]` (or `#![doc ...]`) anywhere.
fn contains_doc_attribute(tokens: &TokenStream) -> bool {
    let tokens = tokens.clone().into_iter().collect::<Vec<_>>();
    tokens.iter().enumerate().any(|(index, token)| match token {
        TokenTree::Group(group) => {
            let bracket_after_pound = group.delimiter() == Delimiter::Bracket
                && pound_before(&tokens, index)
                && matches!(
                    group.stream().into_iter().next(),
                    Some(TokenTree::Ident(ident)) if ident == "doc"
                );
            bracket_after_pound || contains_doc_attribute(&group.stream())
        }
        _ => false,
    })
}

fn pound_before(tokens: &[TokenTree], index: usize) -> bool {
    let pound =
        |token: &TokenTree| matches!(token, TokenTree::Punct(punct) if punct.as_char() == '#');
    let bang =
        |token: &TokenTree| matches!(token, TokenTree::Punct(punct) if punct.as_char() == '!');
    match index {
        0 => false,
        1 => pound(&tokens[0]),
        _ => pound(&tokens[index - 1]) || (bang(&tokens[index - 1]) && pound(&tokens[index - 2])),
    }
}

fn missing(kind: &'static str, crate_root: &str, relative_source: &str, line: usize) -> MissingDoc {
    MissingDoc {
        kind,
        site: format!("{crate_root}/{}:{line}", first(relative_source)),
    }
}

/// A cfg-merged module lists every source joined by `;`; the first is canonical.
fn first(relative_source: &str) -> &str {
    relative_source.split(';').next().unwrap_or(relative_source)
}
