//! Lifecycle state names its variant at every construction.
//!
//! Requirement 2 makes lifecycle truth a sum type whose phase is chosen where
//! it is built. A `Default` would choose a phase for the caller, so the rule
//! rejects a derived or written `Default` on each declared lifecycle type. A
//! declared type that production no longer defines is reported, so the list
//! follows the code.

use super::super::production_scope::{
    self_type_name, visit_impl_item_in_scope, visit_item_in_scope, ProductionModule, ScopedVisitor,
};
use crate::config::TruthTypeDenialConfig;
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use proc_macro2::{TokenStream, TokenTree};
use std::collections::BTreeSet;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Attribute, ImplItem, Item};

pub(super) fn check(
    modules: &[ProductionModule<'_>],
    rule: &TruthTypeDenialConfig,
) -> Vec<Diagnostic> {
    let states: BTreeSet<&str> = rule.lifecycle_states.iter().map(String::as_str).collect();
    let mut defined = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for module in modules {
        let mut reader = DefaultReader {
            states: &states,
            scope: Vec::new(),
            defined: &mut defined,
            defaults: Vec::new(),
        };
        module.items.iter().for_each(|item| reader.visit_item(item));
        for (line, name, form) in reader.defaults {
            diagnostics.push(Diagnostic::new(
                DiagnosticCode::Bc7006LifecycleStateDefault,
                format!("{}/{}:{line}", rule.crate_root, module.relative_source),
                format!("lifecycle state `{name}` {form}: {}", rule.guidance),
            ));
        }
    }
    for name in states.iter().filter(|name| !defined.contains(**name)) {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::Bc7006LifecycleStateDefault,
            &rule.crate_root,
            format!("declared lifecycle state `{name}` is not defined in production; remove it"),
        ));
    }
    diagnostics
}

struct DefaultReader<'a> {
    states: &'a BTreeSet<&'a str>,
    scope: Vec<String>,
    defined: &'a mut BTreeSet<String>,
    defaults: Vec<(usize, String, &'static str)>,
}

impl ScopedVisitor for DefaultReader<'_> {
    fn scope(&mut self) -> &mut Vec<String> {
        &mut self.scope
    }
}

impl DefaultReader<'_> {
    fn definition(&mut self, name: String, attributes: &[Attribute], line: usize) {
        if !self.states.contains(name.as_str()) {
            return;
        }
        if attributes.iter().any(derives_default) {
            self.defaults.push((line, name.clone(), "derives Default"));
        }
        self.defined.insert(name);
    }
}

impl<'ast> Visit<'ast> for DefaultReader<'_> {
    fn visit_item(&mut self, item: &'ast Item) {
        visit_item_in_scope(self, item);
    }

    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        let line = item.span().start().line;
        self.definition(item.ident.to_string(), &item.attrs, line);
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        let line = item.span().start().line;
        self.definition(item.ident.to_string(), &item.attrs, line);
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let name = self_type_name(&item.self_ty);
        let defaults = item.trait_.as_ref().is_some_and(|(_, path, _)| {
            path.segments
                .last()
                .is_some_and(|segment| segment.ident == "Default")
        });
        if defaults && self.states.contains(name.as_str()) {
            let line = item.span().start().line;
            self.defaults.push((line, name, "implements Default"));
        }
        syn::visit::visit_item_impl(self, item);
    }

    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        visit_impl_item_in_scope(self, item);
    }
}

/// A `derive`, directly or under `cfg_attr`, that names `Default`.
fn derives_default(attribute: &Attribute) -> bool {
    let kind = attribute.path().get_ident().map(ToString::to_string);
    let syn::Meta::List(list) = &attribute.meta else {
        return false;
    };
    match kind.as_deref() {
        Some("derive") => names_default(list.tokens.clone()),
        Some("cfg_attr") => derive_under(list.tokens.clone()),
        _ => false,
    }
}

fn derive_under(tokens: TokenStream) -> bool {
    let tokens: Vec<TokenTree> = tokens.into_iter().collect();
    tokens.windows(2).any(|pair| match pair {
        [TokenTree::Ident(ident), TokenTree::Group(group)] if ident == "derive" => {
            names_default(group.stream())
        }
        _ => false,
    })
}

fn names_default(tokens: TokenStream) -> bool {
    tokens.into_iter().any(|token| match token {
        TokenTree::Ident(ident) => ident == "Default",
        TokenTree::Group(group) => names_default(group.stream()),
        _ => false,
    })
}
