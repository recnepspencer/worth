//! Recognize typed allocation-policy safe points, which are not request consults.
use super::super::production_cfg::compiled_out_of_production;
use std::collections::{BTreeMap, BTreeSet};
use syn::{Expr, FnArg, Item, Member, Pat, Type, UseTree};

#[derive(Default)]
pub(super) struct AllocationReceivers {
    types: BTreeSet<String>,
    fields: BTreeMap<String, BTreeSet<String>>,
    bindings: BTreeSet<String>,
}
impl AllocationReceivers {
    pub(super) fn from_items(items: &[Item]) -> Self {
        let mut names = Self::default();
        names.types.insert("ExecutionAllocationPolicy".into());
        for item in items {
            if let Item::Use(item) = item {
                if !compiled_out_of_production(&item.attrs) {
                    names.import(&item.tree);
                }
            }
        }
        for item in items {
            if let Item::Struct(item) = item {
                if compiled_out_of_production(&item.attrs) {
                    continue;
                }
                let fields = item
                    .fields
                    .iter()
                    .filter(|field| {
                        !compiled_out_of_production(&field.attrs) && names.policy_type(&field.ty)
                    })
                    .filter_map(|field| field.ident.as_ref().map(ToString::to_string))
                    .collect();
                names.fields.insert(item.ident.to_string(), fields);
            }
        }
        names
    }
    fn import(&mut self, tree: &UseTree) {
        match tree {
            UseTree::Path(path) => self.import(&path.tree),
            UseTree::Group(group) => {
                for tree in &group.items {
                    self.import(tree);
                }
            }
            UseTree::Rename(rename) if rename.ident == "ExecutionAllocationPolicy" => {
                self.types.insert(rename.rename.to_string());
            }
            UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => {}
        }
    }
    fn policy_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Reference(ty) => self.policy_type(&ty.elem),
            Type::Paren(ty) => self.policy_type(&ty.elem),
            Type::Group(ty) => self.policy_type(&ty.elem),
            Type::Path(ty) => ty
                .path
                .segments
                .last()
                .is_some_and(|segment| self.types.contains(&segment.ident.to_string())),
            _ => false,
        }
    }
    pub(super) fn take_bindings(&mut self) -> BTreeSet<String> {
        std::mem::take(&mut self.bindings)
    }
    pub(super) fn closure(&mut self, closure: &syn::ExprClosure) -> BTreeSet<String> {
        let saved = self.bindings.clone();
        for pat in &closure.inputs {
            let (pat, policy) = match pat {
                Pat::Type(pat) => (pat.pat.as_ref(), self.policy_type(&pat.ty)),
                pat => (pat, false),
            };
            if let Pat::Ident(name) = pat {
                if policy {
                    self.bindings.insert(name.ident.to_string());
                } else {
                    self.bindings.remove(&name.ident.to_string());
                }
            }
        }
        saved
    }
    pub(super) fn restore_bindings(&mut self, saved: BTreeSet<String>) {
        self.bindings = saved;
    }
    pub(super) fn signature(&mut self, signature: &syn::Signature) {
        for argument in &signature.inputs {
            if let FnArg::Typed(argument) = argument {
                if self.policy_type(&argument.ty) {
                    if let Pat::Ident(name) = argument.pat.as_ref() {
                        self.bindings.insert(name.ident.to_string());
                    }
                }
            }
        }
    }
    pub(super) fn local(&mut self, local: &syn::Local, scope: &[String]) {
        let (pat, explicit) = match &local.pat {
            Pat::Type(pat) => (pat.pat.as_ref(), self.policy_type(&pat.ty)),
            pat => (pat, false),
        };
        if let Pat::Ident(name) = pat {
            let policy = explicit
                || local
                    .init
                    .as_ref()
                    .is_some_and(|init| self.receiver(&init.expr, scope));
            if policy {
                self.bindings.insert(name.ident.to_string());
            } else {
                self.bindings.remove(&name.ident.to_string());
            }
        }
    }
    pub(super) fn receiver(&self, expr: &Expr, scope: &[String]) -> bool {
        match expr {
            Expr::Reference(expr) => self.receiver(&expr.expr, scope),
            Expr::Paren(expr) => self.receiver(&expr.expr, scope),
            Expr::Group(expr) => self.receiver(&expr.expr, scope),
            Expr::Path(expr) => expr
                .path
                .get_ident()
                .is_some_and(|name| self.bindings.contains(&name.to_string())),
            Expr::Field(expr) => {
                matches!(expr.base.as_ref(), Expr::Path(base) if base.path.is_ident("self"))
                    && matches!(&expr.member, Member::Named(name) if scope.iter().any(|owner| self.fields.get(owner).is_some_and(|fields| fields.contains(&name.to_string()))))
            }
            _ => false,
        }
    }
    pub(super) fn qualified(&self, path: &syn::Path) -> bool {
        path.segments
            .iter()
            .rev()
            .nth(1)
            .is_some_and(|segment| self.types.contains(&segment.ident.to_string()))
    }
}
