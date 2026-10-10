//! Request backing is constructed only at the crate's declared host entry.
//! Production module graphs keep test-only mints out of this rule without
//! permitting a production module to escape through a test-looking filename.
use super::crate_modules::ModuleGraph;
use super::production_scope::{
    production_graphs, production_modules, visit_field_in_scope, visit_impl_item_in_scope,
    visit_item_in_scope, visit_trait_item_in_scope, visit_variant_in_scope, ScopedVisitor,
};
use crate::config::RequestConstructorDenialConfig;
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::collections::BTreeSet;
use std::path::Path;
use syn::visit::{self, Visit};
use syn::{ImplItem, Item, TraitItem};

pub(crate) fn enforce_request_constructor_denials(
    root: &Path,
    rules: &[RequestConstructorDenialConfig],
) -> Vec<Diagnostic> {
    rules
        .iter()
        .flat_map(|rule| match production_graphs(root, &rule.crate_root) {
            Ok(graphs) => check_graphs(&graphs, rule),
            Err(error) => vec![Diagnostic::new(
                DiagnosticCode::Bc7008RequestConstruction,
                &rule.crate_root,
                format!("request-construction boundary could not be read: {error}"),
            )],
        })
        .collect()
}

fn check_graphs(graphs: &[ModuleGraph], rule: &RequestConstructorDenialConfig) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut host_seen = false;
    for module in production_modules(graphs) {
        let mut visitor = RequestBackingNames::default();
        for item in module.items {
            visitor.visit_item(item);
        }
        if visitor.found.is_empty() {
            continue;
        }
        // An allowance names one file exactly, never a directory or item bag.
        if rule
            .host_entry
            .as_deref()
            .is_some_and(|host| module.sources().all(|source| source == host))
        {
            host_seen = true;
            continue;
        }
        diagnostics.extend(visitor.found.into_iter().map(|(item, name, line)| Diagnostic::new(
            DiagnosticCode::Bc7008RequestConstruction,
            format!("{}/{}:{line}", rule.crate_root, module.relative_source),
            format!("request backing name `{name}` in `{item}` is outside the declared host entry: {}", rule.guidance),
        )));
    }
    if let Some(host) = &rule.host_entry {
        if !host_seen {
            diagnostics.push(Diagnostic::new(
                DiagnosticCode::Bc7008RequestConstruction,
                format!("{}/{host}", rule.crate_root),
                "declared host entry names no production request backing; remove the stale allowance",
            ));
        }
    }
    diagnostics
}

#[derive(Default)]
struct RequestBackingNames {
    scope: Vec<String>,
    found: BTreeSet<(String, String, usize)>,
}
impl ScopedVisitor for RequestBackingNames {
    fn scope(&mut self) -> &mut Vec<String> {
        &mut self.scope
    }
}
impl RequestBackingNames {
    fn tokens(&mut self, tokens: proc_macro2::TokenStream) {
        for token in tokens {
            match token {
                proc_macro2::TokenTree::Ident(identifier) => self.visit_ident(&identifier),
                proc_macro2::TokenTree::Group(group) => self.tokens(group.stream()),
                proc_macro2::TokenTree::Punct(_) | proc_macro2::TokenTree::Literal(_) => {}
            }
        }
    }
}
impl<'ast> Visit<'ast> for RequestBackingNames {
    fn visit_item(&mut self, item: &'ast Item) {
        visit_item_in_scope(self, item);
    }
    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        visit_impl_item_in_scope(self, item);
    }
    fn visit_trait_item(&mut self, item: &'ast TraitItem) {
        visit_trait_item_in_scope(self, item);
    }
    fn visit_field(&mut self, field: &'ast syn::Field) {
        visit_field_in_scope(self, field);
    }
    fn visit_variant(&mut self, variant: &'ast syn::Variant) {
        visit_variant_in_scope(self, variant);
    }
    fn visit_ident(&mut self, identifier: &proc_macro2::Ident) {
        let written = identifier.to_string();
        let name = written.trim_start_matches("r#");
        // Deny the backing vocabulary too: a renamed import, type alias,
        // function-value constructor or macro cannot conceal the mint.
        if matches!(name, "SerialRequest" | "serial_request") {
            self.found.insert((
                self.scope.join("::"),
                name.to_owned(),
                identifier.span().start().line,
            ));
        }
    }
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        visit::visit_macro(self, node);
        self.tokens(node.tokens.clone());
    }
}

#[cfg(test)]
mod tests;
