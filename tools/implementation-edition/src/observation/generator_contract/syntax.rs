use super::provenance::{literal, relative_path, Bindings};
use super::Program;
use crate::rust_identifier::semantic_name;
use quote::ToTokens;
use std::collections::BTreeSet;
use syn::{visit::Visit, Expr};

pub(super) struct Syntax<'a> {
    pub(super) program: &'a Program,
    pub(super) reads: BTreeSet<String>,
    pub(super) environment: BTreeSet<String>,
    pub(super) bindings: Bindings,
    pub(super) error: Option<String>,
}
impl Syntax<'_> {
    fn refuse(&mut self, detail: impl std::fmt::Display) {
        self.error
            .get_or_insert_with(|| format!("generator effect requires qualification: {detail}"));
    }
}
impl<'ast> Visit<'ast> for Syntax<'_> {
    fn visit_attribute(&mut self, _: &'ast syn::Attribute) {
        self.refuse("attribute");
    }
    fn visit_expr(&mut self, expr: &'ast Expr) {
        if !matches!(
            expr,
            Expr::Lit(_)
                | Expr::Path(_)
                | Expr::Call(_)
                | Expr::MethodCall(_)
                | Expr::Reference(_)
                | Expr::Field(_)
                | Expr::Binary(_)
                | Expr::If(_)
                | Expr::Match(_)
                | Expr::Return(_)
                | Expr::Try(_)
                | Expr::Block(_)
                | Expr::Macro(_)
                | Expr::Paren(_)
                | Expr::Unary(_)
        ) {
            self.refuse(expr.to_token_stream());
        }
        if let Expr::Reference(reference) = expr
            && reference.mutability.is_some()
        {
            self.refuse("mutable reference");
        }
        if let Expr::Unary(unary) = expr
            && (!matches!(unary.op, syn::UnOp::Not(_))
                || !self.bindings.boolean(self.program, &unary.expr, 0))
        {
            self.refuse("unary operator requires a source-proven boolean !");
        }
        if let Expr::Binary(binary) = expr
            && !matches!(
                binary.op,
                syn::BinOp::Eq(_)
                    | syn::BinOp::Ne(_)
                    | syn::BinOp::Lt(_)
                    | syn::BinOp::Ge(_)
                    | syn::BinOp::And(_)
                    | syn::BinOp::Or(_)
            )
        {
            self.refuse("binary operator or mutation");
        }
        syn::visit::visit_expr(self, expr);
    }
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        let Expr::Path(path) = call.func.as_ref() else {
            self.refuse("indirect call");
            return;
        };
        let name = self.program.path(&path.path);
        if name == "std::process::Command::new"
            && !call
                .args
                .first()
                .is_some_and(|e| self.bindings.cargo_value(self.program, e, "RUSTC", 0))
        {
            self.refuse("compiler command requires a source-proven RUSTC origin");
        }
        if name == "std::path::PathBuf::from"
            && !call
                .args
                .first()
                .is_some_and(|e| self.bindings.cargo_value(self.program, e, "OUT_DIR", 0))
        {
            self.refuse("output path requires a source-proven OUT_DIR origin");
        }
        if name == "std::fs::write"
            && !call
                .args
                .first()
                .is_some_and(|e| self.bindings.output_path(self.program, e, 0))
        {
            self.refuse("write requires a source-proven output path");
        }
        if matches!(name.as_str(), "std::env::var" | "std::env::var_os") {
            match call.args.first().and_then(literal) {
                Some(variable)
                    if matches!(
                        variable.as_str(),
                        "OUT_DIR" | "RUSTC" | "TARGET" | "HOST" | "CARGO_PKG_VERSION_PATCH"
                    ) =>
                {
                    self.environment.insert(variable);
                }
                _ => self.refuse("ambient environment"),
            }
        }
        if !self.program.functions.contains_key(&name)
            && !matches!(
                name.as_str(),
                "Some"
                    | "std::env::var"
                    | "std::env::var_os"
                    | "std::path::PathBuf::from"
                    | "std::fs::read"
                    | "std::fs::read_to_string"
                    | "std::fs::write"
                    | "std::process::Command::new"
                    | "std::str::from_utf8"
            )
        {
            self.refuse(&name);
        }
        if matches!(name.as_str(), "std::fs::read" | "std::fs::read_to_string") {
            if let Some(Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(path),
                ..
            })) = call.args.first()
            {
                if relative_path(&path.value()) {
                    self.reads.insert(path.value());
                } else {
                    self.refuse("file read escapes package");
                }
            } else {
                self.refuse("file read requires a literal input");
            }
        }
        syn::visit::visit_expr_call(self, call);
    }
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if semantic_name(&call.method) == "output"
            && !self
                .bindings
                .compiler_query(self.program, &call.receiver, 0)
        {
            self.refuse("process output requires a source-proven selected rustc --version query");
        }
        if semantic_name(&call.method) == "arg"
            && (call.args.len() != 1
                || call.args.first().and_then(literal).as_deref() != Some("--version"))
        {
            self.refuse("compiler query argument requires selected rustc --version");
        }
        if semantic_name(&call.method) == "join"
            && (call.args.len() != 1
                || !call
                    .args
                    .first()
                    .and_then(literal)
                    .is_some_and(|p| relative_path(&p)))
        {
            self.refuse("output path join requires a confined literal");
        }
        if !matches!(
            semantic_name(&call.method).as_str(),
            "unwrap"
                | "ok"
                | "trim"
                | "parse"
                | "join"
                | "replace"
                | "split"
                | "next"
                | "starts_with"
                | "arg"
                | "output"
        ) {
            self.refuse(&call.method);
        }
        syn::visit::visit_expr_method_call(self, call);
    }
    fn visit_macro(&mut self, invocation: &'ast syn::Macro) {
        if !invocation
            .path
            .get_ident()
            .is_some_and(|name| matches!(semantic_name(name).as_str(), "println" | "format"))
            || syn::parse2::<syn::LitStr>(invocation.tokens.clone()).is_err()
        {
            self.refuse("macro requires one literal format string");
        }
    }
    fn visit_item(&mut self, _: &'ast syn::Item) {
        self.refuse("nested item");
    }
    fn visit_pat(&mut self, pattern: &'ast syn::Pat) {
        if !matches!(
            pattern,
            syn::Pat::Ident(_) | syn::Pat::Type(_) | syn::Pat::TupleStruct(_) | syn::Pat::Path(_)
        ) {
            self.refuse("pattern");
        }
        syn::visit::visit_pat(self, pattern);
    }
}
