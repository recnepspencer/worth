//! Structural proof of the wire identity's write/admission boundary.

use syn::visit::{self, Visit};
use syn::{Block, Expr, Stmt};

fn path(expression: &Expr, name: &str) -> bool {
    matches!(expression, Expr::Path(value) if value.path.is_ident(name))
}

fn expression(statement: &Stmt) -> Option<&Expr> {
    match statement {
        Stmt::Expr(value, _) => Some(value),
        _ => None,
    }
}

pub(super) fn fixed_encoder(block: &Block) -> bool {
    if !initialized_local(block, "target", "Vec::new", None)
        || !block
            .stmts
            .last()
            .and_then(expression)
            .is_some_and(|value| path(value, "target"))
    {
        return false;
    }
    let Some(Expr::Call(call)) = block.stmts.get(1).and_then(expression) else {
        return false;
    };
    path(&call.func, "field")
        && call.args.len() == 2
        && matches!(call.args.first(), Some(Expr::Reference(reference))
            if reference.mutability.is_some() && path(&reference.expr, "target"))
        && call
            .args
            .last()
            .is_some_and(|value| path(value, super::CURRENT_DOMAIN))
}

pub(super) fn early_admission(block: &Block) -> bool {
    if !initialized_local(block, "cursor", "Cursor::new", Some("bytes")) {
        return false;
    }
    let Some(Expr::Try(attempt)) = block.stmts.get(1).and_then(expression) else {
        return false;
    };
    let Expr::Call(call) = attempt.expr.as_ref() else {
        return false;
    };
    if !path(&call.func, "require_current_domain") || call.args.len() != 1 {
        return false;
    }
    let Some(Expr::Try(field)) = call.args.first() else {
        return false;
    };
    matches!(field.expr.as_ref(), Expr::MethodCall(call)
        if path(&call.receiver, "cursor") && call.method == "field" && call.args.is_empty())
}

fn initialized_local(block: &Block, name: &str, constructor: &str, argument: Option<&str>) -> bool {
    let Some(Stmt::Local(local)) = block.stmts.first() else {
        return false;
    };
    if !matches!(&local.pat, syn::Pat::Ident(binding)
        if binding.ident == name && binding.mutability.is_some() && binding.subpat.is_none())
    {
        return false;
    }
    let Some(initializer) = &local.init else {
        return false;
    };
    if initializer.diverge.is_some() {
        return false;
    }
    let Expr::Call(call) = initializer.expr.as_ref() else {
        return false;
    };
    let Expr::Path(function) = call.func.as_ref() else {
        return false;
    };
    let expected = constructor.split("::");
    let constructor_matches = function.path.leading_colon.is_none()
        && function.path.segments.len() == expected.clone().count()
        && function
            .path
            .segments
            .iter()
            .zip(expected)
            .all(|(segment, name)| {
                segment.ident == name && matches!(segment.arguments, syn::PathArguments::None)
            });
    constructor_matches
        && match argument {
            None => call.args.is_empty(),
            Some(name) => {
                call.args.len() == 1 && call.args.first().is_some_and(|value| path(value, name))
            }
        }
}

pub(super) fn sole_current_success(block: &Block) -> bool {
    let Some(Expr::If(guard)) = block.stmts.first().and_then(expression) else {
        return false;
    };
    let Expr::Binary(condition) = guard.cond.as_ref() else {
        return false;
    };
    if !matches!(condition.op, syn::BinOp::Eq(_))
        || !path(&condition.left, "domain")
        || !path(&condition.right, super::CURRENT_DOMAIN)
        || guard.else_branch.is_some()
        || guard.then_branch.stmts.len() != 1
    {
        return false;
    }
    let Some(Expr::Return(success)) = guard.then_branch.stmts.first().and_then(expression) else {
        return false;
    };
    if !success.expr.as_deref().is_some_and(ok_unit) {
        return false;
    }
    let mut returns = DomainReturns::default();
    returns.visit_block(block);
    returns.successes == 1
        && !returns.other_return
        && block.stmts.last().and_then(expression).is_some_and(err)
}

fn ok_unit(expression: &Expr) -> bool {
    matches!(expression, Expr::Call(call) if path(&call.func, "Ok") && call.args.len() == 1
        && matches!(call.args.first(), Some(Expr::Tuple(tuple)) if tuple.elems.is_empty()))
}

fn err(expression: &Expr) -> bool {
    matches!(expression, Expr::Call(call) if path(&call.func, "Err") && call.args.len() == 1)
}

#[derive(Default)]
struct DomainReturns {
    successes: usize,
    other_return: bool,
}

impl<'ast> Visit<'ast> for DomainReturns {
    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        if matches!(expression.func.as_ref(), Expr::Path(path) if path.path.segments.last().is_some_and(|part| part.ident == "Ok"))
        {
            self.successes += 1;
        }
        visit::visit_expr_call(self, expression);
    }
    fn visit_expr_return(&mut self, expression: &'ast syn::ExprReturn) {
        if !expression
            .expr
            .as_deref()
            .is_some_and(|value| ok_unit(value) || err(value))
        {
            self.other_return = true;
        }
        visit::visit_expr_return(self, expression);
    }
}
