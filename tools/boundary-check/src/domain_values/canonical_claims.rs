//! Canonical constructors claim the binding's actual Value, never the codec type.
use syn::{visit::Visit, Expr, ItemImpl};

pub(super) fn canonical_binding(item: &ItemImpl) -> bool {
    let mut calls = Constructors(false);
    calls.visit_item_impl(item);
    calls.0 || item.items.iter().any(|member| {
        let syn::ImplItem::Fn(function) = member else { return false; };
        if function.sig.ident != "canonical_contract"
            || !item.items.iter().any(|member| matches!(member, syn::ImplItem::Type(value) if value.ident == "Value")) {
            return false;
        }
        // A renamed constructor or an indirect factory must not hide a binding.
        // The sole non-claim is an explicit absent contract.
        !matches!(function.block.stmts.as_slice(), [syn::Stmt::Expr(Expr::Path(path), None)] if path.path.is_ident("None"))
    })
}

struct Constructors(bool);
impl<'ast> Visit<'ast> for Constructors {
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let Expr::Path(path) = call.func.as_ref() {
            let segments: Vec<_> = path.path.segments.iter().collect();
            if segments.windows(2).any(|pair| {
                pair[0].ident == "ApplicationCanonicalValueContract"
                    && (pair[1].ident == "of" || pair[1].ident == "delegated")
            }) {
                self.0 = true;
            }
        }
        syn::visit::visit_expr_call(self, call);
    }
}
