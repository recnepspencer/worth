//! Discover literal declarations and reject ambiguous declaration arguments.
use proc_macro2::{TokenStream, TokenTree};
use std::collections::BTreeSet;

// A convention name passed to an item macro may declare a new door or merely
// reference another owner's door. Without a literal exported declaration, the
// syntax cannot establish ownership: reject it rather than grant authority.
pub(super) fn ambiguous_argument(tokens: TokenStream) -> Option<String> {
    let mut candidates = BTreeSet::new();
    argument_names(tokens.clone(), &mut candidates);
    let mut declared = BTreeSet::new();
    names(tokens, &mut declared);
    candidates.difference(&declared).next().cloned()
}

fn argument_names(tokens: TokenStream, names: &mut BTreeSet<String>) {
    for token in tokens {
        match token {
            TokenTree::Ident(name) => {
                let name = name.to_string();
                let name = name.strip_prefix("r#").unwrap_or(&name);
                if super::public_items::test_name(name) {
                    names.insert(name.into());
                }
            }
            TokenTree::Group(group) => argument_names(group.stream(), names),
            _ => {}
        }
    }
}

pub(super) fn names(tokens: TokenStream, names: &mut BTreeSet<String>) {
    let tokens: Vec<_> = tokens.into_iter().collect();
    for (index, token) in tokens.iter().enumerate() {
        if let TokenTree::Group(group) = token {
            self::names(group.stream(), names);
        }
        if !matches!(token, TokenTree::Ident(name) if name == "pub") {
            continue;
        }
        let mut cursor = index + 1;
        if let Some(TokenTree::Group(group)) = tokens.get(cursor) {
            if group.stream().to_string() != "crate" {
                continue;
            }
            cursor += 1;
        }
        while matches!(tokens.get(cursor), Some(TokenTree::Ident(name)) if matches!(name.to_string().as_str(), "async"|"unsafe") || (name == "const" && matches!(tokens.get(cursor + 1), Some(TokenTree::Ident(next)) if next == "fn")))
        {
            cursor += 1;
        }
        if !matches!(tokens.get(cursor), Some(TokenTree::Ident(name)) if matches!(name.to_string().as_str(), "fn"|"struct"|"enum"|"trait"|"type"|"mod"|"static"|"const"))
        {
            continue;
        }
        if let Some(TokenTree::Ident(name)) = tokens.get(cursor + 1) {
            let name = name.to_string();
            let name = name.strip_prefix("r#").unwrap_or(&name);
            if super::public_items::test_name(name) {
                names.insert(name.into());
            }
        }
    }
}
