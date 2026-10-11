use std::collections::BTreeSet;
use syn::visit::{self, Visit};
struct Address<'a> {
    found: bool,
    helpers: &'a BTreeSet<String>,
    locals: &'a BTreeSet<String>,
}
impl<'ast> Visit<'ast> for Address<'_> {
    fn visit_expr_method_call(&mut self, e: &'ast syn::ExprMethodCall) {
        let name = e.method.to_string();
        self.found |= self.helpers.contains(&name)
            || matches!(
                name.as_str(),
                "as_ptr" | "as_mut_ptr" | "addr" | "expose_provenance"
            );
        visit::visit_expr_method_call(self, e);
    }
    fn visit_expr_call(&mut self, e: &'ast syn::ExprCall) {
        if let syn::Expr::Path(p) = e.func.as_ref() {
            let names: Vec<_> = p
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect();
            self.found |= names.last().is_some_and(|n| {
                self.helpers.contains(n)
                    || matches!(
                        n.as_str(),
                        "as_ptr"
                            | "as_mut_ptr"
                            | "addr"
                            | "from_ref"
                            | "into_raw"
                            | "expose_provenance"
                    )
            }) || (names.last().is_some_and(|n| n == "from")
                && names.iter().any(|n| n == "NonNull"));
        }
        visit::visit_expr_call(self, e);
    }
    fn visit_expr_path(&mut self, e: &'ast syn::ExprPath) {
        self.found |= e
            .path
            .get_ident()
            .is_some_and(|i| self.locals.contains(&i.to_string()));
        visit::visit_expr_path(self, e);
    }
    fn visit_expr_cast(&mut self, e: &'ast syn::ExprCast) {
        self.found |= pointer_type(&e.ty);
        visit::visit_expr_cast(self, e);
    }
    fn visit_expr_raw_addr(&mut self, _: &'ast syn::ExprRawAddr) {
        self.found = true;
    }
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        self.found |= m
            .path
            .segments
            .last()
            .is_some_and(|s| matches!(s.ident.to_string().as_str(), "addr_of" | "addr_of_mut"));
    }
}
pub(super) fn expression(
    e: &syn::Expr,
    helpers: &BTreeSet<String>,
    locals: &BTreeSet<String>,
) -> bool {
    let mut visitor = Address {
        found: false,
        helpers,
        locals,
    };
    visitor.visit_expr(e);
    visitor.found
}
pub(super) fn block(b: &syn::Block, helpers: &BTreeSet<String>) -> bool {
    let locals = BTreeSet::new();
    let mut visitor = Address {
        found: false,
        helpers,
        locals: &locals,
    };
    visitor.visit_block(b);
    visitor.found
}
pub(super) fn pointer_type(ty: &syn::Type) -> bool {
    struct Pointer(bool);
    impl<'ast> Visit<'ast> for Pointer {
        fn visit_type_ptr(&mut self, _: &'ast syn::TypePtr) {
            self.0 = true;
        }
    }
    let mut p = Pointer(false);
    p.visit_type(ty);
    p.0
}
pub(super) fn ordering_derive(a: &syn::Attribute) -> bool {
    a.path().is_ident("derive")
        && a.parse_args_with(
            syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
        )
        .is_ok_and(|paths| {
            paths.iter().any(|p| {
                p.segments
                    .last()
                    .is_some_and(|s| matches!(s.ident.to_string().as_str(), "Ord" | "PartialOrd"))
            })
        })
}
pub(super) fn ordered_map_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(p) if p.path.segments.last().is_some_and(|s| matches!(s.ident.to_string().as_str(), "BTreeMap" | "BTreeSet")))
}
