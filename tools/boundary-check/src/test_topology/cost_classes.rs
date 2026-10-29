//! Resolve real module paths rather than guessing cost classes from filenames.
use super::failure;
use crate::{
    diagnostics::Diagnostic,
    source_rules::{parse_compiled_target_where, GovernedCrate},
};
use std::path::Path;
use syn::parse::Parser;
use syn::visit::{self, Visit};

const CLASSES: &[&str] = &["fast", "process", "exhaustive", "scale", "compile_fail"];

pub(super) fn validate(owner: &GovernedCrate, target: &Path, expected: &str) -> Vec<Diagnostic> {
    let graph = match parse_compiled_target_where(owner, target, &|_| true) {
        Ok(graph) => graph,
        Err(error) => return vec![failure(&owner.relative_crate_root, error)],
    };
    let mut diagnostics = Vec::new();
    for (path, node) in graph.modules {
        let first = path
            .first()
            .map(|first| first.strip_prefix("r#").unwrap_or(first));
        let classed = first == Some(expected);
        if path.len() == 1 && first.is_some_and(|name| CLASSES.contains(&name) && name != expected)
        {
            diagnostics.push(failure(
                &owner.relative_crate_root,
                format!(
                    "court declares a second class module `{}`; expected `{expected}`",
                    first.unwrap()
                ),
            ));
        }
        let mut tests = Tests { names: Vec::new() };
        for item in &node.items {
            if !matches!(item, syn::Item::Mod(_)) {
                tests.visit_item(item);
            }
        }
        if !classed {
            for name in tests.names {
                diagnostics.push(failure(
                    &format!("{}/{}", owner.relative_crate_root, node.relative_source),
                    format!("integration test `{name}` must be under its court class `{expected}`"),
                ));
            }
        }
    }
    diagnostics
}

struct Tests {
    names: Vec<String>,
}
impl<'ast> Visit<'ast> for Tests {
    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        if function.attrs.iter().any(|attr| test_attribute(&attr.meta)) {
            self.names.push(function.sig.ident.to_string());
        }
        visit::visit_item_fn(self, function);
    }
    fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
        if item.mac.path.is_ident("include") || contains_test_attribute(item.mac.tokens.clone()) {
            self.names.push(
                item.ident
                    .as_ref()
                    .map_or_else(|| "<macro>".into(), ToString::to_string),
            );
        }
    }
}

fn test_attribute(meta: &syn::Meta) -> bool {
    if meta
        .path()
        .segments
        .last()
        .is_some_and(|s| s.ident == "test")
    {
        return true;
    }
    let syn::Meta::List(list) = meta else {
        return false;
    };
    if !list.path.is_ident("cfg_attr") {
        return false;
    }
    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
        .parse2(list.tokens.clone())
        .is_ok_and(|metas| metas.iter().skip(1).any(test_attribute))
}

fn contains_test_attribute(tokens: proc_macro2::TokenStream) -> bool {
    use proc_macro2::TokenTree;
    let tokens: Vec<_> = tokens.into_iter().collect();
    for pair in tokens.windows(2) {
        if matches!(&pair[0], TokenTree::Punct(mark) if mark.as_char() == '#') {
            if let TokenTree::Group(group) = &pair[1] {
                if syn::parse2::<syn::Meta>(group.stream()).is_ok_and(|meta| test_attribute(&meta))
                {
                    return true;
                }
            }
        }
    }
    tokens.into_iter().any(|token| match token {
        TokenTree::Group(group) => contains_test_attribute(group.stream()),
        TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => false,
    })
}
