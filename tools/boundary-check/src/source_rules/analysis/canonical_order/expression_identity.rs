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
