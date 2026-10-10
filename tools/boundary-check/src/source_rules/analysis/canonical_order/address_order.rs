//! Allocation addresses establish equality, never a declared ordering.
use syn::visit::{self, Visit};

pub(super) fn reads_address(item: &syn::ItemImpl) -> bool {
    #[derive(Default)]
    struct Address(bool);
    impl<'ast> Visit<'ast> for Address {
        fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
            self.0 |= matches!(call.method.to_string().as_str(), "as_ptr" | "addr");
            visit::visit_expr_method_call(self, call);
        }
        fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
            if let syn::Expr::Path(path) = call.func.as_ref() {
                let mut names = path.path.segments.iter().rev().map(|s| s.ident.to_string());
                self.0 |= names.next().is_some_and(|name| {
                    name == "as_ptr"
                        || name == "addr"
                        || (name == "from_ref"
                            && names.next().is_some_and(|parent| parent == "ptr"))
                });
            }
            visit::visit_expr_call(self, call);
        }
    }
    let mut address = Address::default();
    address.visit_item_impl(item);
    address.0
}
