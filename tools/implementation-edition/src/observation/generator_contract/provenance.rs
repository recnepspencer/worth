//! Effect arguments have source-checked origins on every control-flow path.
//! Evaluation alone does not grant permission to spawn or write.
use super::Program;
use crate::rust_identifier::semantic_name;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path},
};
use syn::{visit::Visit, Expr};

#[derive(Default)]
pub(super) struct Bindings {
    initializers: BTreeMap<String, Vec<Option<Expr>>>,
    mutated: BTreeSet<String>,
}
impl Bindings {
    pub(super) fn of(block: &syn::Block) -> Self {
        let mut bindings = Self::default();
        bindings.visit_block(block);
        bindings
    }
    fn initializer(&self, expression: &Expr) -> Option<&Expr> {
        let Expr::Path(path) = expression else {
            return None;
        };
        let name = semantic_name(path.path.get_ident()?);
        if self.mutated.contains(&name) {
            return None;
        }
        let values = self.initializers.get(&name)?;
        if values.len() == 1 {
            values[0].as_ref()
        } else {
            None
        }
    }
    pub(super) fn boolean(&self, program: &Program, expression: &Expr, depth: usize) -> bool {
        if depth > 16 {
            return false;
        }
        if let Some(init) = self.initializer(expression) {
            return self.boolean(program, init, depth + 1);
        }
        match expression {
            Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Bool(_),
                ..
            }) => true,
            Expr::Binary(binary) => matches!(
                binary.op,
                syn::BinOp::Eq(_)
                    | syn::BinOp::Ne(_)
                    | syn::BinOp::Lt(_)
                    | syn::BinOp::Ge(_)
                    | syn::BinOp::And(_)
                    | syn::BinOp::Or(_)
            ),
            Expr::Unary(value) if matches!(value.op, syn::UnOp::Not(_)) => {
                self.boolean(program, &value.expr, depth + 1)
            }
            Expr::Paren(value) => self.boolean(program, &value.expr, depth + 1),
            Expr::Reference(value) => self.boolean(program, &value.expr, depth + 1),
            Expr::MethodCall(call) => semantic_name(&call.method) == "starts_with",
            Expr::Path(path) => program
                .constants
                .get(&program.path(&path.path))
                .is_some_and(|value| self.boolean(program, value, depth + 1)),
            Expr::Call(call) => {
                matches!(call.func.as_ref(), Expr::Path(path) if program.functions.get(&program.path(&path.path)).is_some_and(|function| matches!(&function.sig.output, syn::ReturnType::Type(_, ty) if matches!(ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("bool")))))
            }
            _ => false,
        }
    }
    pub(super) fn cargo_value(
        &self,
        program: &Program,
        expression: &Expr,
        name: &str,
        depth: usize,
    ) -> bool {
        if depth > 16 {
            return false;
        }
        if let Some(init) = self.initializer(expression) {
            return self.cargo_value(program, init, name, depth + 1);
        }
        match expression {
            Expr::Try(value) => self.cargo_value(program, &value.expr, name, depth + 1),
            Expr::Paren(value) => self.cargo_value(program, &value.expr, name, depth + 1),
            Expr::Reference(value) => self.cargo_value(program, &value.expr, name, depth + 1),
            Expr::MethodCall(call)
                if call.args.is_empty()
                    && matches!(semantic_name(&call.method).as_str(), "unwrap" | "ok") =>
            {
                self.cargo_value(program, &call.receiver, name, depth + 1)
            }
            Expr::Call(call) if call.args.len() == 1 => {
                matches!(call.func.as_ref(),Expr::Path(path) if matches!(program.path(&path.path).as_str(),"std::env::var"|"std::env::var_os"))
                    && literal(&call.args[0]).as_deref() == Some(name)
            }
            _ => false,
        }
    }
    pub(super) fn output_path(&self, program: &Program, expression: &Expr, depth: usize) -> bool {
        if depth > 16 {
            return false;
        }
        if let Some(init) = self.initializer(expression) {
            return self.output_path(program, init, depth + 1);
        }
        match expression {
            Expr::Reference(value) => self.output_path(program, &value.expr, depth + 1),
            Expr::Paren(value) => self.output_path(program, &value.expr, depth + 1),
            Expr::Call(call) if call.args.len() == 1 => {
                matches!(call.func.as_ref(),Expr::Path(path) if program.path(&path.path)=="std::path::PathBuf::from")
                    && self.cargo_value(program, &call.args[0], "OUT_DIR", depth + 1)
            }
            Expr::MethodCall(call)
                if semantic_name(&call.method) == "join" && call.args.len() == 1 =>
            {
                literal(&call.args[0]).is_some_and(|s| relative_path(&s))
                    && self.output_path(program, &call.receiver, depth + 1)
            }
            _ => false,
        }
    }

    pub(super) fn compiler_query(
        &self,
        program: &Program,
        expression: &Expr,
        depth: usize,
    ) -> bool {
        if depth > 16 {
            return false;
        }
        // A fresh expression avoids mutable Command aliases accumulating
        // additional arguments outside this query's visible chain.
        match expression {
            Expr::Paren(value) => self.compiler_query(program, &value.expr, depth + 1),
            Expr::MethodCall(call)
                if semantic_name(&call.method) == "arg"
                    && call.args.len() == 1
                    && literal(&call.args[0]).as_deref() == Some("--version") =>
            {
                self.compiler_command(program, &call.receiver, depth + 1)
            }
            _ => false,
        }
    }

    fn compiler_command(&self, program: &Program, expression: &Expr, depth: usize) -> bool {
        if depth > 16 {
            return false;
        }
        match expression {
            Expr::Paren(value) => self.compiler_command(program, &value.expr, depth + 1),
            Expr::Call(call) if call.args.len() == 1 => {
                matches!(call.func.as_ref(),Expr::Path(path) if program.path(&path.path)=="std::process::Command::new")
                    && self.cargo_value(program, &call.args[0], "RUSTC", depth + 1)
            }
            _ => false,
        }
    }
}
impl<'ast> Visit<'ast> for Bindings {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        let pattern = match &local.pat {
            syn::Pat::Type(p) => p.pat.as_ref(),
            p => p,
        };
        if let (syn::Pat::Ident(name), Some(init)) = (pattern, &local.init) {
            self.initializers
                .entry(semantic_name(&name.ident))
                .or_default()
                .push(Some((*init.expr).clone()));
        }
        if let Some(init) = &local.init {
            self.visit_expr(&init.expr);
        }
    }
    fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
        self.initializers
            .entry(semantic_name(&pattern.ident))
            .or_default()
            .push(None);
        syn::visit::visit_pat_ident(self, pattern);
    }
    fn visit_expr_assign(&mut self, assignment: &'ast syn::ExprAssign) {
        self.invalidate(&assignment.left);
        syn::visit::visit_expr_assign(self, assignment);
    }
    fn visit_expr_binary(&mut self, binary: &'ast syn::ExprBinary) {
        if matches!(
            binary.op,
            syn::BinOp::AddAssign(_)
                | syn::BinOp::SubAssign(_)
                | syn::BinOp::MulAssign(_)
                | syn::BinOp::DivAssign(_)
                | syn::BinOp::RemAssign(_)
                | syn::BinOp::BitXorAssign(_)
                | syn::BinOp::BitAndAssign(_)
                | syn::BinOp::BitOrAssign(_)
                | syn::BinOp::ShlAssign(_)
                | syn::BinOp::ShrAssign(_)
        ) {
            self.invalidate(&binary.left);
        }
        syn::visit::visit_expr_binary(self, binary);
    }
}
impl Bindings {
    fn invalidate(&mut self, expr: &Expr) {
        match expr {
            Expr::Path(path) => {
                if let Some(name) = path.path.get_ident() {
                    self.mutated.insert(semantic_name(name));
                }
            }
            Expr::Paren(value) => self.invalidate(&value.expr),
            Expr::Field(value) => self.invalidate(&value.base),
            Expr::Index(value) => self.invalidate(&value.expr),
            _ => {}
        }
    }
}
pub(super) fn literal(expression: &Expr) -> Option<String> {
    match expression {
        Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(value),
            ..
        }) => Some(value.value()),
        _ => None,
    }
}
pub(super) fn relative_path(value: &str) -> bool {
    !value.is_empty()
        && Path::new(value)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}
