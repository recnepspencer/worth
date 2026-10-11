//! A commit request has one consult owner; stage observers therefore count every consult.
use super::crate_modules::ModuleGraph;
use super::production_scope::{
    production_graphs, production_modules, visit_field_in_scope, visit_impl_item_in_scope,
    visit_item_in_scope, visit_trait_item_in_scope, visit_variant_in_scope, ScopedVisitor,
};
use crate::diagnostics::{Diagnostic, DiagnosticCode};
use std::path::Path;
use syn::visit::Visit;

const CRATE: &str = "crates/worth-relational";
const OWNER: &str = "src/authority/commit/pipeline/request_checkpoints.rs";
const STAGE: &str = "consult_stage_control";
const ROOTS: [&str; 5] = [
    "src/authority/commit/",
    "src/mvcc/transaction/",
    "src/mvcc/validation/",
    "src/mvcc/publication/",
    "src/commit_strategies/validation.rs",
];

pub(super) fn validate(root: &Path) -> Vec<Diagnostic> {
    // This rule owns Relational's commit paths, not unrelated workspace roots.
    // A present but unreadable crate still reports the parsing failure below.
    if !root.join(CRATE).exists() {
        return Vec::new();
    }
    match production_graphs(root, CRATE) {
        Ok(graphs) => check_graphs(&graphs),
        Err(error) => vec![diagnostic(
            CRATE,
            format!("commit consult boundary could not be read: {error}"),
        )],
    }
}
fn diagnostic(subject: impl Into<String>, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::Bc7011CommitRequestConsult, subject, message)
}
fn covered(source: &str) -> bool {
    ROOTS.iter().any(|root| source.starts_with(root))
}
fn check_graphs(graphs: &[ModuleGraph]) -> Vec<Diagnostic> {
    let mut found = Vec::new();
    let mut owner_consults = 0;
    for module in production_modules(graphs) {
        if !module.sources().any(covered) {
            continue;
        }
        let at_owner = module.sources().all(|source| source == OWNER);
        let mut visitor = ConsultNames {
            allocation: allocation_receivers::AllocationReceivers::from_items(module.items),
            ..ConsultNames::default()
        };
        for item in module.items {
            visitor.visit_item(item);
        }
        for (scope, line, spelling) in visitor.found {
            if at_owner && scope == STAGE && spelling == "consult" {
                owner_consults += 1;
            } else {
                found.push(diagnostic(format!("{CRATE}/{}:{line}", module.relative_source),
                format!("request liveness read `{spelling}` in `{scope}` bypasses the commit stage; borrow the request through consult_stage")));
            }
        }
    }
    if owner_consults != 1 {
        found.push(diagnostic(
            format!("{CRATE}/{OWNER}"),
            format!("{STAGE} must name exactly one request consult, found {owner_consults}"),
        ));
    }
    found
}
#[derive(Default)]
struct ConsultNames {
    scope: Vec<String>,
    found: Vec<(String, usize, String)>,
    allocation: allocation_receivers::AllocationReceivers,
}
impl ScopedVisitor for ConsultNames {
    fn scope(&mut self) -> &mut Vec<String> {
        &mut self.scope
    }
}
impl ConsultNames {
    fn record(&mut self, name: &proc_macro2::Ident) {
        self.found.push((
            self.scope.join("::"),
            name.span().start().line,
            name.to_string().trim_start_matches("r#").to_owned(),
        ));
    }
    fn tokens(&mut self, tokens: proc_macro2::TokenStream) {
        if let Ok(expression) = syn::parse2::<syn::Expr>(tokens.clone()) {
            self.visit_expr(&expression);
            return;
        }
        for token in tokens {
            match token {
                proc_macro2::TokenTree::Ident(name) => {
                    if name.to_string().trim_start_matches("r#") == "check_live" {
                        self.record(&name);
                    } else {
                        self.visit_ident(&name);
                    }
                }
                proc_macro2::TokenTree::Group(group) => self.tokens(group.stream()),
                proc_macro2::TokenTree::Punct(_) | proc_macro2::TokenTree::Literal(_) => {}
            }
        }
    }
}
impl<'ast> Visit<'ast> for ConsultNames {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        let saved = matches!(item, syn::Item::Fn(_)).then(|| self.allocation.take_bindings());
        visit_item_in_scope(self, item);
        if let Some(saved) = saved {
            self.allocation.restore_bindings(saved);
        }
    }
    fn visit_impl_item(&mut self, item: &'ast syn::ImplItem) {
        let saved = matches!(item, syn::ImplItem::Fn(_)).then(|| self.allocation.take_bindings());
        visit_impl_item_in_scope(self, item);
        if let Some(saved) = saved {
            self.allocation.restore_bindings(saved);
        }
    }
    fn visit_trait_item(&mut self, item: &'ast syn::TraitItem) {
        visit_trait_item_in_scope(self, item);
    }
    fn visit_field(&mut self, item: &'ast syn::Field) {
        visit_field_in_scope(self, item);
    }
    fn visit_variant(&mut self, item: &'ast syn::Variant) {
        visit_variant_in_scope(self, item);
    }
    fn visit_ident(&mut self, name: &proc_macro2::Ident) {
        // ExecutionRequest has two liveness reads; Relational adds one helper.
        if matches!(
            name.to_string().trim_start_matches("r#"),
            "consult" | "check_request"
        ) {
            self.record(name);
        }
    }
    fn visit_signature(&mut self, signature: &'ast syn::Signature) {
        self.allocation.signature(signature);
        syn::visit::visit_signature(self, signature);
    }
    fn visit_local(&mut self, local: &'ast syn::Local) {
        self.allocation.local(local, &self.scope);
        syn::visit::visit_local(self, local);
    }
    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        let saved = self.allocation.closure(closure);
        syn::visit::visit_expr_closure(self, closure);
        self.allocation.restore_bindings(saved);
    }
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if call.method.to_string().trim_start_matches("r#") == "check_live"
            && !self.allocation.receiver(&call.receiver, &self.scope)
        {
            self.record(&call.method);
        }
        syn::visit::visit_expr_method_call(self, call);
    }
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if let Some(last) = path.segments.last() {
            if last.ident.to_string().trim_start_matches("r#") == "check_live"
                && !self.allocation.qualified(path)
            {
                self.record(&last.ident);
            }
        }
        syn::visit::visit_path(self, path);
    }
    fn visit_use_name(&mut self, name: &'ast syn::UseName) {
        if name.ident.to_string().trim_start_matches("r#") == "check_live" {
            self.record(&name.ident);
        }
        syn::visit::visit_use_name(self, name);
    }
    fn visit_use_rename(&mut self, name: &'ast syn::UseRename) {
        if name.ident.to_string().trim_start_matches("r#") == "check_live" {
            self.record(&name.ident);
        }
        syn::visit::visit_use_rename(self, name);
    }
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        self.visit_path(&item.path);
        self.tokens(item.tokens.clone());
    }
}
mod allocation_receivers;
#[cfg(test)]
mod tests;
