//! Lexical evidence of build identity in a deciding expression.
use std::collections::BTreeSet;
use syn::visit::{self, Visit};

pub(super) fn decides_order(name: &str) -> bool {
    ["sort", "min", "max", "binary_search"]
        .iter()
        .any(|prefix| name.starts_with(prefix))
        || matches!(name, "cmp" | "partial_cmp")
}

pub(super) fn names_type_identity<'a>(
    expressions: impl IntoIterator<Item = &'a syn::Expr>,
    type_ids: &BTreeSet<String>,
) -> bool {
    struct Names<'a> {
        type_ids: &'a BTreeSet<String>,
        found: bool,
    }
    impl<'ast> Visit<'ast> for Names<'_> {
        fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
            if call.method == "identity"
                && call.args.len() == 1
                && matches!(call.receiver.as_ref(), syn::Expr::Path(_))
            {
                // A named declaration lookup consumes equality identity and
                // supplies the sort key. Its input is not the deciding value.
                self.visit_expr(&call.receiver);
                if let Some(arguments) = &call.turbofish {
                    self.visit_angle_bracketed_generic_arguments(arguments);
                }
                return;
            }
            visit::visit_expr_method_call(self, call);
        }
        fn visit_ident(&mut self, ident: &'ast syn::Ident) {
            let name = ident.to_string();
            self.found |=
                self.type_ids.contains(&name) || name == "type_id" || name.ends_with("_type_id");
            visit::visit_ident(self, ident);
        }
    }
    let mut names = Names {
        type_ids,
        found: false,
    };
    for expression in expressions {
        names.visit_expr(expression);
    }
    names.found
}
