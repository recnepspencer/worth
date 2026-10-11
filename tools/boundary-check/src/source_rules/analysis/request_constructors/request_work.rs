//! The carried request must be used, and work amounts must be finite.
use super::super::production_scope::{
    visit_impl_item_in_scope, visit_item_in_scope, visit_trait_item_in_scope, ScopedVisitor,
};
use std::collections::BTreeSet;
use syn::visit::{self, Visit};
use syn::{Expr, FnArg, ImplItem, Item, Pat, TraitItem, Type};

pub(super) fn violations(items: &[Item], owners: &BTreeSet<String>) -> Vec<(String, usize)> {
    let mut visitor = RequestWork {
        owners,
        scope: Vec::new(),
        work_scope: false,
        found: BTreeSet::new(),
    };
    for item in items {
        visitor.visit_item(item);
    }
    visitor.found.into_iter().collect()
}

struct RequestWork<'a> {
    owners: &'a BTreeSet<String>,
    scope: Vec<String>,
    work_scope: bool,
    found: BTreeSet<(String, usize)>,
}

impl ScopedVisitor for RequestWork<'_> {
    fn scope(&mut self) -> &mut Vec<String> {
        &mut self.scope
    }
}

fn request(ty: &Type, owners: &BTreeSet<String>) -> bool {
    match ty {
        Type::Reference(reference) => request(&reference.elem, owners),
        _ => super::request_aliases::is_owner(ty, owners),
    }
}

fn unbounded(expr: &Expr) -> Option<usize> {
    struct Maximum(Option<usize>);
    impl<'ast> Visit<'ast> for Maximum {
        fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
            let names: Vec<_> = path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect();
            if names.len() >= 2
                && names[names.len() - 1] == "MAX"
                && matches!(names[names.len() - 2].as_str(), "usize" | "u64")
            {
                self.0 = Some(path.path.segments.last().unwrap().ident.span().start().line);
            }
            visit::visit_expr_path(self, path);
        }
    }
    let mut maximum = Maximum(None);
    maximum.visit_expr(expr);
    maximum.0
}

impl RequestWork<'_> {
    fn amount(&mut self, expression: &Expr) {
        if let Some(line) = unbounded(expression) {
            self.found
                .insert(("unbounded production work amount".into(), line));
        }
    }
    fn signature(&mut self, signature: &syn::Signature) {
        for argument in &signature.inputs {
            let FnArg::Typed(argument) = argument else {
                continue;
            };
            if !request(&argument.ty, self.owners) {
                continue;
            }
            let ignored = match argument.pat.as_ref() {
                Pat::Ident(binding) => binding.ident.to_string().starts_with('_'),
                Pat::Wild(_) => true,
                _ => false,
            };
            if ignored {
                self.found.insert((
                    "ignored production request parameter".into(),
                    signature.ident.span().start().line,
                ));
            }
        }
    }
}

impl<'ast> Visit<'ast> for RequestWork<'_> {
    fn visit_item(&mut self, item: &'ast Item) {
        visit_item_in_scope(self, item);
    }
    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        visit_impl_item_in_scope(self, item);
    }
    fn visit_trait_item(&mut self, item: &'ast TraitItem) {
        visit_trait_item_in_scope(self, item);
    }
    fn visit_signature(&mut self, signature: &'ast syn::Signature) {
        self.signature(signature);
        visit::visit_signature(self, signature);
    }
    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        for input in &closure.inputs {
            let Pat::Type(input) = input else { continue };
            if !request(&input.ty, self.owners) {
                continue;
            }
            let ignored = match input.pat.as_ref() {
                Pat::Ident(binding) if binding.ident.to_string().starts_with('_') => {
                    Some(binding.ident.span().start().line)
                }
                Pat::Wild(binding) => Some(binding.underscore_token.span.start().line),
                _ => None,
            };
            if let Some(line) = ignored {
                self.found
                    .insert(("ignored production request parameter".into(), line));
            }
        }
        visit::visit_expr_closure(self, closure);
    }
    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        let prior = self.work_scope;
        self.work_scope = function.sig.ident == "remaining_work";
        visit::visit_item_fn(self, function);
        self.work_scope = prior;
    }
    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        let prior = self.work_scope;
        self.work_scope = function.sig.ident == "remaining_work";
        visit::visit_impl_item_fn(self, function);
        self.work_scope = prior;
    }
    fn visit_expr(&mut self, expression: &'ast Expr) {
        if self.work_scope {
            self.amount(expression);
        }
        visit::visit_expr(self, expression);
    }
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Pat::Ident(binding) = &local.pat {
            if binding.ident.to_string().contains("work") {
                if let Some(initializer) = &local.init {
                    self.amount(&initializer.expr);
                }
            }
        }
        visit::visit_local(self, local);
    }
    fn visit_item_const(&mut self, item: &'ast syn::ItemConst) {
        if item.ident.to_string().to_ascii_lowercase().contains("work") {
            self.amount(&item.expr);
        }
        visit::visit_item_const(self, item);
    }
    fn visit_expr_struct(&mut self, expression: &'ast syn::ExprStruct) {
        for field in &expression.fields {
            if let syn::Member::Named(name) = &field.member {
                if name.to_string().contains("work") {
                    self.amount(&field.expr);
                }
            }
        }
        visit::visit_expr_struct(self, expression);
    }
    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        if let Expr::Path(path) = expression.func.as_ref() {
            let owners = path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>();
            if owners.iter().any(|name| name == "ExecutionBudget") {
                if let Some(work) = expression.args.iter().nth(2) {
                    self.amount(work);
                }
            } else if owners.iter().any(|name| {
                name.contains("Work")
                    || matches!(name.as_str(), "EntityKindScan" | "RelationKindScan")
            }) {
                for amount in &expression.args {
                    self.amount(amount);
                }
            }
        }
        visit::visit_expr_call(self, expression);
    }
    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        let method = expression.method.to_string();
        if matches!(
            method.as_str(),
            "checkpoint" | "charge" | "try_charge" | "admit" | "admit_work" | "prepare"
        ) {
            if let Some(amount) = expression.args.first() {
                self.amount(amount);
            }
        }
        if method.starts_with("bounded_")
            && (method.contains("kind") || method.contains("frontier"))
        {
            if let Some(amount) = expression.args.last() {
                self.amount(amount);
            }
        }
        visit::visit_expr_method_call(self, expression);
    }
}
