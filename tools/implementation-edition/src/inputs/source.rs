use crate::fingerprint;
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::ToTokens;
use serde_json::{json, Value};
use std::path::Path;
use syn::visit_mut::VisitMut;

pub(crate) fn content_digest(path: &Path, content: &[u8], test: bool) -> Result<String, String> {
    if path.extension().is_none_or(|extension| extension != "rs") {
        return Ok(fingerprint::bytes(content));
    }
    let source = std::str::from_utf8(content).map_err(|error| error.to_string())?;
    let mut parsed =
        syn::parse_file(source).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut excluded = ExcludeTestItems {
        test,
        spans: Vec::new(),
    };
    excluded.visit_file_mut(&mut parsed);
    super::native_source::validate(&parsed)?;
    let tokens = parsed.into_token_stream();
    let semantic = if source_sensitive(tokens.clone()) {
        // Excluded bytes are not compiled, but their line/column effects can
        // reach retained tokens through line!/column! or macro invocation spans.
        // Keep those production positions separately from excluded content.
        let positions = source_positions(tokens);
        let mut preserved = source.to_owned();
        excluded
            .spans
            .sort_by_key(|span| std::cmp::Reverse(span.start));
        for span in excluded.spans {
            preserved.replace_range(span, "");
        }
        json!({"span-sensitive": preserved, "production-positions": positions})
    } else {
        token_value(tokens)
    };
    Ok(fingerprint::value(
        "worth.compiler.semantic-input.v1",
        &semantic,
    ))
}

fn source_positions(tokens: TokenStream) -> Value {
    Value::Array(
        tokens
            .into_iter()
            .map(|token| {
                let start = token.span().start();
                let end = token.span().end();
                let children = match &token {
                    TokenTree::Group(group) => source_positions(group.stream()),
                    _ => Value::Null,
                };
                json!([start.line, start.column, end.line, end.column, children])
            })
            .collect(),
    )
}

#[cfg(test)]
#[path = "source_tests.rs"]
mod tests;

fn source_sensitive(tokens: TokenStream) -> bool {
    let tokens: Vec<_> = tokens.into_iter().collect();
    for (index, token) in tokens.iter().enumerate() {
        if let TokenTree::Ident(name) = token
            && matches!(tokens.get(index + 1), Some(TokenTree::Punct(mark)) if mark.as_char() == '!')
            && !matches!(
                name.to_string().as_str(),
                "include" | "include_str" | "include_bytes" | "env" | "option_env"
            )
        {
            return true;
        }
        if let TokenTree::Group(group) = token
            && source_sensitive(group.stream())
        {
            return true;
        }
    }
    false
}

fn token_value(tokens: TokenStream) -> Value {
    Value::Array(
        tokens
            .into_iter()
            .map(|token| match token {
                TokenTree::Group(group) => json!([
                    match group.delimiter() {
                        Delimiter::Parenthesis => "()",
                        Delimiter::Brace => "{}",
                        Delimiter::Bracket => "[]",
                        Delimiter::None => "none",
                    },
                    token_value(group.stream())
                ]),
                TokenTree::Ident(name) => json!(["identifier", name.to_string()]),
                TokenTree::Literal(literal) => json!(["literal", literal.to_string()]),
                TokenTree::Punct(punct) => json!([
                    "punctuation",
                    punct.as_char().to_string(),
                    format!("{:?}", punct.spacing())
                ]),
            })
            .collect(),
    )
}

struct ExcludeTestItems {
    test: bool,
    spans: Vec<std::ops::Range<usize>>,
}

impl VisitMut for ExcludeTestItems {
    fn visit_file_mut(&mut self, file: &mut syn::File) {
        self.items(&mut file.items);
    }

    fn visit_item_mod_mut(&mut self, module: &mut syn::ItemMod) {
        if let Some((_, items)) = &mut module.content {
            self.items(items);
        }
    }
}

impl ExcludeTestItems {
    fn items(&mut self, items: &mut Vec<syn::Item>) {
        items.retain(|item| {
            // Parsing an item into an attribute-bearing form keeps unsupported
            // grammar conservative rather than guessing an inactive compound cfg.
            let tokens = item.to_token_stream();
            let attributes = match item {
                syn::Item::Fn(item) => &item.attrs,
                syn::Item::Mod(item) => &item.attrs,
                syn::Item::Const(item) => &item.attrs,
                syn::Item::Static(item) => &item.attrs,
                syn::Item::Struct(item) => &item.attrs,
                syn::Item::Enum(item) => &item.attrs,
                syn::Item::Impl(item) => &item.attrs,
                syn::Item::Trait(item) => &item.attrs,
                syn::Item::Use(item) => &item.attrs,
                syn::Item::Type(item) => &item.attrs,
                syn::Item::Macro(item) => &item.attrs,
                syn::Item::ExternCrate(item) => &item.attrs,
                syn::Item::Union(item) => &item.attrs,
                syn::Item::ForeignMod(item) => &item.attrs,
                _ => return true,
            };
            let excluded = !self.test
                && attributes.iter().any(|attribute| {
                    attribute.path().is_ident("cfg")
                        && attribute
                            .parse_args::<syn::Path>()
                            .is_ok_and(|path| path.is_ident("test"))
                });
            if excluded {
                let mut iter = tokens.into_iter();
                if let Some(first) = iter.next() {
                    let start = first.span().byte_range().start;
                    let end = iter.last().map_or_else(
                        || first.span().byte_range().end,
                        |token| token.span().byte_range().end,
                    );
                    self.spans.push(start..end);
                }
            }
            !excluded
        });
        for item in items {
            self.visit_item_mut(item);
        }
    }
}
