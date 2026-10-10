//! Host evaluation vocabulary and Signal receivers cannot cross into carried lanes.
use super::super::production_scope::{
    visit_impl_item_in_scope, visit_item_in_scope, ScopedVisitor,
};
use std::collections::BTreeSet;
use syn::visit::{self, Visit};
use syn::{Expr, ImplItem, Item, Pat, Type, UseTree};

pub(super) fn unchecked_references(items: &[Item]) -> BTreeSet<(String, usize)> {
    let mut names = SignalNames::default();
    // Imports precede bindings semantically even when written later. Resolve
    // aliases and typed receivers before following local expression bindings.
    loop {
        let before = names.types.len() + names.receivers.len() + names.producers.len();
        for item in items {
            names.visit_item(item);
        }
        if before == names.types.len() + names.receivers.len() + names.producers.len() {
            break;
        }
    }
    let mut doors = HostDoors {
        names,
        found: BTreeSet::new(),
        scope: Vec::new(),
    };
    for item in items {
        doors.visit_item(item);
    }
    doors.found
}

fn specific_door(name: &str) -> bool {
    let name = name.trim_start_matches("r#");
    matches!(
        name,
        "build_evaluation_plan"
            | "build_evaluation_plan_with_policy_resolver"
            | "execute_prepared_plan"
            | "execute_prepared_plan_with_precompute"
            | "evaluate_dirty"
            | "evaluate_with_plan"
    )
}
fn receiver_door(name: &str) -> bool {
    let name = name.trim_start_matches("r#");
    specific_door(name) || matches!(name, "run" | "read" | "read_many" | "get")
}

struct SignalNames {
    types: BTreeSet<String>,
    receivers: BTreeSet<String>,
    producers: BTreeSet<String>,
    scope: Vec<String>,
}
impl Default for SignalNames {
    fn default() -> Self {
        Self {
            types: [
                "SignalGraph",
                "SignalRuntime",
                "SignalTransaction",
                "RuntimeExecutionRequest",
                "TransactionExecutionRequest",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            receivers: BTreeSet::new(),
            producers: BTreeSet::new(),
            scope: Vec::new(),
        }
    }
}
impl ScopedVisitor for SignalNames {
    fn scope(&mut self) -> &mut Vec<String> {
        &mut self.scope
    }
}
impl SignalNames {
    fn signal_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Reference(ty) => self.signal_type(&ty.elem),
            Type::Path(ty) => ty.path.segments.iter().any(|s| {
                self.types.contains(&s.ident.to_string())
                    || matches!(&s.arguments,
                    syn::PathArguments::AngleBracketed(args) if args.args.iter().any(|arg|
                        matches!(arg, syn::GenericArgument::Type(ty) if self.signal_type(ty))))
            }),
            Type::Paren(ty) => self.signal_type(&ty.elem),
            _ => false,
        }
    }
    fn signal_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Path(path) => path.path.segments.iter().any(|s| {
                self.receivers.contains(&s.ident.to_string())
                    || self.types.contains(&s.ident.to_string())
            }),
            Expr::Field(field) => {
                matches!(&field.member, syn::Member::Named(name) if self.receivers.contains(&name.to_string()))
            }
            Expr::Reference(expr) => self.signal_expr(&expr.expr),
            Expr::Paren(expr) => self.signal_expr(&expr.expr),
            Expr::Try(expr) => self.signal_expr(&expr.expr),
            Expr::MethodCall(call) => self.signal_expr(&call.receiver),
            Expr::Call(call) => match &*call.func {
                Expr::Path(path) => path.path.segments.iter().any(|s| {
                    self.types.contains(&s.ident.to_string())
                        || self.producers.contains(&s.ident.to_string())
                }),
                _ => false,
            },
            _ => false,
        }
    }
    fn bind(&mut self, pat: &Pat) {
        match pat {
            Pat::Ident(pat) => {
                self.receivers.insert(pat.ident.to_string());
            }
            Pat::Type(pat) => self.bind(&pat.pat),
            _ => {}
        }
    }
}
impl<'ast> Visit<'ast> for SignalNames {
    fn visit_item(&mut self, item: &'ast Item) {
        visit_item_in_scope(self, item);
    }
    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        visit_impl_item_in_scope(self, item);
    }
    fn visit_use_tree(&mut self, tree: &'ast UseTree) {
        if let UseTree::Rename(rename) = tree {
            if self.types.contains(&rename.ident.to_string()) {
                self.types.insert(rename.rename.to_string());
            }
        }
        visit::visit_use_tree(self, tree);
    }
    fn visit_item_type(&mut self, alias: &'ast syn::ItemType) {
        if self.signal_type(&alias.ty) {
            self.types.insert(alias.ident.to_string());
        }
        visit::visit_item_type(self, alias);
    }
    fn visit_pat_type(&mut self, pat: &'ast syn::PatType) {
        if self.signal_type(&pat.ty) {
            self.bind(&pat.pat);
        }
        visit::visit_pat_type(self, pat);
    }
    fn visit_field(&mut self, field: &'ast syn::Field) {
        if self.signal_type(&field.ty) {
            if let Some(name) = &field.ident {
                self.receivers.insert(name.to_string());
            }
        }
        visit::visit_field(self, field);
    }
    fn visit_signature(&mut self, signature: &'ast syn::Signature) {
        if let syn::ReturnType::Type(_, ty) = &signature.output {
            if self.signal_type(ty) {
                self.producers.insert(signature.ident.to_string());
            }
        }
        visit::visit_signature(self, signature);
    }
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if local
            .init
            .as_ref()
            .is_some_and(|init| self.signal_expr(&init.expr))
        {
            self.bind(&local.pat);
        }
        visit::visit_local(self, local);
    }
}

struct HostDoors {
    names: SignalNames,
    found: BTreeSet<(String, usize)>,
    scope: Vec<String>,
}
impl ScopedVisitor for HostDoors {
    fn scope(&mut self) -> &mut Vec<String> {
        &mut self.scope
    }
}
impl HostDoors {
    fn deny(&mut self, name: &proc_macro2::Ident) {
        self.found
            .insert((name.to_string(), name.span().start().line));
    }
    fn macro_tokens(&mut self, tokens: proc_macro2::TokenStream) {
        let mut names = Vec::new();
        Self::macro_names(tokens, &mut names);
        let signal_receiver = names.iter().any(|name| {
            self.names.receivers.contains(&name.to_string())
                || self.names.types.contains(&name.to_string())
        });
        for name in names {
            if specific_door(&name.to_string())
                || (signal_receiver && receiver_door(&name.to_string()))
            {
                self.deny(&name);
            }
        }
    }
    fn macro_names(tokens: proc_macro2::TokenStream, names: &mut Vec<proc_macro2::Ident>) {
        for token in tokens {
            match token {
                proc_macro2::TokenTree::Ident(name) => names.push(name),
                proc_macro2::TokenTree::Group(group) => Self::macro_names(group.stream(), names),
                _ => {}
            }
        }
    }
}
impl<'ast> Visit<'ast> for HostDoors {
    fn visit_item(&mut self, item: &'ast Item) {
        visit_item_in_scope(self, item);
    }
    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        visit_impl_item_in_scope(self, item);
    }
    fn visit_ident(&mut self, name: &'ast proc_macro2::Ident) {
        if specific_door(&name.to_string()) {
            self.deny(name);
        }
    }
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if receiver_door(&call.method.to_string()) && self.names.signal_expr(&call.receiver) {
            self.deny(&call.method);
        }
        visit::visit_expr_method_call(self, call);
    }
    fn visit_expr_path(&mut self, expr: &'ast syn::ExprPath) {
        if expr
            .path
            .segments
            .iter()
            .any(|s| self.names.types.contains(&s.ident.to_string()))
        {
            if let Some(method) = expr.path.segments.last() {
                if receiver_door(&method.ident.to_string()) {
                    self.deny(&method.ident);
                }
            }
        }
        visit::visit_expr_path(self, expr);
    }
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        self.macro_tokens(node.tokens.clone());
        visit::visit_macro(self, node);
    }
}
