//! Inspect compiler-owned expanded syntax before analysis or code generation.
use rustc_ast::{
    ast,
    visit::{self, Visitor},
};

pub(super) fn check(krate: &ast::Crate) -> Result<(), String> {
    let mut check = Expanded { refusal: None };
    check.visit_crate(krate);
    check.refusal.map_or(Ok(()), Err)
}
struct Expanded {
    refusal: Option<String>,
}
impl<'ast> Visitor<'ast> for Expanded {
    fn visit_expr(&mut self, expression: &'ast ast::Expr) {
        if matches!(expression.kind, ast::ExprKind::InlineAsm(..)) {
            self.refusal = Some("expanded native assembly requires qualification".into());
        }
        visit::walk_expr(self, expression);
    }
    fn visit_item(&mut self, item: &'ast ast::Item) {
        if matches!(
            item.kind,
            ast::ItemKind::ForeignMod(..) | ast::ItemKind::GlobalAsm(..)
        ) {
            self.refusal = Some("expanded foreign/native inputs require qualification".into());
        }
        visit::walk_item(self, item);
    }
}
