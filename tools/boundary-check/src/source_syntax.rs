//! The one parser every source analysis reads Rust through.
//!
//! `r#name` names the same item as `name`, so a raw spelling must not slip
//! past a rule written against the bare one. Each raw identifier is rewritten
//! to its bare spelling once, here, including inside macro and attribute
//! tokens, so no analysis has to remember to. A raw keyword (`r#type`) stays
//! raw because its bare spelling is not an identifier; no rule names one.

use proc_macro2::{Group, Ident, TokenStream, TokenTree};
use quote::ToTokens;

pub(crate) fn parse_file(source: &str) -> syn::Result<syn::File> {
    let parsed = syn::parse_file(source)?;
    syn::parse2(bare_identifiers(parsed.into_token_stream()))
}

fn bare_identifiers(tokens: TokenStream) -> TokenStream {
    tokens
        .into_iter()
        .map(|token| match token {
            TokenTree::Ident(identifier) => TokenTree::Ident(bare(identifier)),
            TokenTree::Group(group) => {
                let mut bare_group =
                    Group::new(group.delimiter(), bare_identifiers(group.stream()));
                bare_group.set_span(group.span());
                TokenTree::Group(bare_group)
            }
            other => other,
        })
        .collect()
}

fn bare(identifier: Ident) -> Ident {
    let spelled = identifier.to_string();
    match spelled.strip_prefix("r#") {
        Some(name) if syn::parse_str::<Ident>(name).is_ok() => Ident::new(name, identifier.span()),
        _ => identifier,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_file;
    use quote::ToTokens;

    #[test]
    fn raw_identifiers_read_as_their_bare_spelling_everywhere() {
        let file = parse_file(
            "use r#serde_json::r#Value as r#Alias;\n\
             #[r#cfg(r#test)] fn r#run() { r#println!(\"{}\", r#Value); }",
        )
        .expect("raw identifiers parse");
        let spelled = file.to_token_stream().to_string();
        assert!(!spelled.contains("r#"), "{spelled}");
        assert!(
            spelled.contains("serde_json :: Value as Alias"),
            "{spelled}"
        );
        assert!(spelled.contains("cfg (test)"), "{spelled}");
        assert!(spelled.contains("println ! (\"{}\" , Value)"), "{spelled}");
    }

    #[test]
    fn raw_keywords_stay_raw_so_the_source_still_parses() {
        let file =
            parse_file("struct Shape { r#type: u8 } fn r#match() {}").expect("raw keywords parse");
        let spelled = file.to_token_stream().to_string();
        assert!(spelled.contains("r#type"), "{spelled}");
        assert!(spelled.contains("r#match"), "{spelled}");
    }

    #[test]
    fn spans_keep_their_source_lines() {
        let file = parse_file("\n\nfn r#late() {}").expect("parses");
        let syn::Item::Fn(function) = &file.items[0] else {
            panic!("function");
        };
        assert_eq!(function.sig.ident, "late");
        assert_eq!(function.sig.ident.span().start().line, 3);
    }
}
