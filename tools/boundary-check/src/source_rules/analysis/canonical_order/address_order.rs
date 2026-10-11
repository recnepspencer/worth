//! Syntactic address flow may establish equality, never an ordering key.
mod reads;
use super::inventory::Finding;
use std::collections::BTreeSet;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

pub(super) fn is_finding(reason: &str) -> bool {
    reason.ends_with("an address") || reason == "ordering derive holds a raw pointer"
}
pub(super) fn scan(files: &[(&str, &[syn::Item])]) -> Vec<Finding> {
    let mut found = Vec::new();
    for (source, items) in files {
        let mut helpers = reads::imported_functions(items);
        loop {
            let before = helpers.len();
            for item in *items {
                match item {
                    syn::Item::Fn(f) if reads::block(&f.block, &helpers) => {
                        helpers.insert(f.sig.ident.to_string());
                    }
                    syn::Item::Impl(i) => {
                        for member in &i.items {
                            if let syn::ImplItem::Fn(f) = member {
                                if reads::block(&f.block, &helpers) {
                                    helpers.insert(f.sig.ident.to_string());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            if helpers.len() == before {
                break;
            }
        }
        let mut scanner = Sites {
            source,
            item: String::new(),
            helpers: &helpers,
            locals: BTreeSet::new(),
            maps: BTreeSet::new(),
            found: &mut found,
        };
        for item in *items {
            scanner.visit_item(item);
        }
    }
    found
}
struct Sites<'a> {
    source: &'a str,
    item: String,
    helpers: &'a BTreeSet<String>,
    locals: BTreeSet<String>,
    maps: BTreeSet<String>,
    found: &'a mut Vec<Finding>,
}
impl Sites<'_> {
    fn record(&mut self, span: proc_macro2::Span, reason: &'static str) {
        self.found.push(Finding {
            source: self.source.into(),
            item: self.item.clone(),
            line: span.start().line,
            reason,
        });
    }
    fn address(&self, expr: &syn::Expr) -> bool {
        reads::expression(expr, self.helpers, &self.locals)
    }
}
impl<'ast> Visit<'ast> for Sites<'_> {
    fn visit_item_mod(&mut self, _: &'ast syn::ItemMod) {}
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        self.item = item.ident.to_string();
        if item.fields.iter().any(|f| reads::pointer_type(&f.ty))
            && item.attrs.iter().any(reads::ordering_derive)
        {
            self.record(item.span(), "ordering derive holds a raw pointer");
        }
    }
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        self.item = match item.self_ty.as_ref() {
            syn::Type::Path(p) => p.path.segments.last().unwrap().ident.to_string(),
            _ => String::new(),
        };
        let ordered = item.trait_.as_ref().is_some_and(|(_, p, _)| {
            p.segments
                .last()
                .is_some_and(|s| matches!(s.ident.to_string().as_str(), "Ord" | "PartialOrd"))
        });
        if ordered
            && item
                .items
                .iter()
                .any(|i| matches!(i, syn::ImplItem::Fn(f) if reads::block(&f.block, self.helpers)))
        {
            self.record(item.span(), "ordering impl reads an address");
            return;
        }
        visit::visit_item_impl(self, item);
    }
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.item = item.sig.ident.to_string();
        self.locals.clear();
        self.maps.clear();
        visit::visit_item_fn(self, item);
    }
    fn visit_local(&mut self, local: &'ast syn::Local) {
        let (pattern, ty) = match &local.pat {
            syn::Pat::Type(p) => (p.pat.as_ref(), Some(p.ty.as_ref())),
            p => (p, None),
        };
        if let syn::Pat::Ident(p) = pattern {
            if local.init.as_ref().is_some_and(|i| self.address(&i.expr)) {
                self.locals.insert(p.ident.to_string());
            }
            if ty.is_some_and(reads::ordered_map_type) {
                self.maps.insert(p.ident.to_string());
            }
        }
        visit::visit_local(self, local);
    }
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let name = call.method.to_string();
        let ordering = ["sort", "min", "max", "binary_search"]
            .iter()
            .any(|p| name.starts_with(p))
            || matches!(name.as_str(), "cmp" | "partial_cmp");
        let map_insert = name == "insert"
            && matches!(call.receiver.as_ref(), syn::Expr::Path(p) if p.path.get_ident().is_some_and(|i| self.maps.contains(&i.to_string())));
        if (ordering && (self.address(&call.receiver) || call.args.iter().any(|e| self.address(e))))
            || (map_insert && call.args.first().is_some_and(|e| self.address(e)))
        {
            self.record(call.span(), "ordering expression reads an address");
            return;
        }
        visit::visit_expr_method_call(self, call);
    }
    fn visit_expr_binary(&mut self, expr: &'ast syn::ExprBinary) {
        if matches!(
            expr.op,
            syn::BinOp::Lt(_) | syn::BinOp::Le(_) | syn::BinOp::Gt(_) | syn::BinOp::Ge(_)
        ) && (self.address(&expr.left) || self.address(&expr.right))
        {
            self.record(expr.span(), "ordering expression reads an address");
            return;
        }
        visit::visit_expr_binary(self, expr);
    }
}
