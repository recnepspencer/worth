//! Evaluate the finite program into effects without replacing native execution
//! or writing output. The native result must equal this checked description.
use super::{
    effects::Effects,
    values::{Fault, Value},
    Program,
};
use crate::rust_identifier::semantic_name;
use std::{collections::BTreeMap, fs, path::Path};
use syn::{Expr, Stmt};

pub(super) struct Evaluation<'a> {
    pub(super) program: &'a Program,
    pub(super) frames: Vec<Vec<BTreeMap<String, Value>>>,
    pub(super) effects: Effects,
    pub(super) root: &'a Path,
    pub(super) output: &'a Path,
    pub(super) compiler: &'a Path,
    pub(super) environment: &'a BTreeMap<String, std::ffi::OsString>,
    pub(super) steps: usize,
}

impl Evaluation<'_> {
    pub(super) fn lookup(&mut self, name: &str) -> Result<Value, Fault> {
        if name == "None" {
            return Ok(Value::Optional(None));
        }
        if let Some(value) = self
            .frames
            .last()
            .and_then(|scopes| scopes.iter().rev().find_map(|vars| vars.get(name)))
        {
            return Ok(value.clone());
        }
        if let Some(expr) = self.program.constants.get(name).cloned() {
            return self.expr(&expr);
        }
        Err(format!("unqualified generator value: {name}").into())
    }
    pub(super) fn function(&mut self, name: &str) -> Result<Value, Fault> {
        if self.frames.len() >= 32 {
            return Err("generator helper recursion limit".into());
        }
        let function = self
            .program
            .functions
            .get(name)
            .ok_or("generator helper missing")?
            .clone();
        self.frames.push(Vec::new());
        let result = match self.block(&function.block) {
            Err(Fault::Return(value)) => Ok(value),
            other => other,
        };
        self.frames.pop();
        result
    }
    fn block(&mut self, block: &syn::Block) -> Result<Value, Fault> {
        self.frames.last_mut().unwrap().push(BTreeMap::new());
        let result = self.statements(&block.stmts);
        self.frames.last_mut().unwrap().pop();
        result
    }
    fn statements(&mut self, statements: &[Stmt]) -> Result<Value, Fault> {
        let mut value = Value::Unit;
        for statement in statements {
            match statement {
                Stmt::Local(local) => {
                    let pattern = match &local.pat {
                        syn::Pat::Type(pattern) => pattern.pat.as_ref(),
                        pattern => pattern,
                    };
                    let syn::Pat::Ident(name) = pattern else {
                        return Err("generator local pattern requires qualification".into());
                    };
                    if name.by_ref.is_some() || name.subpat.is_some() {
                        return Err("generator binding mode requires qualification".into());
                    }
                    let init = local
                        .init
                        .as_ref()
                        .ok_or("generator local initializer missing")?;
                    if init.diverge.is_some() {
                        return Err("generator let-else requires qualification".into());
                    }
                    let value = self.expr(&init.expr)?;
                    self.frames
                        .last_mut()
                        .unwrap()
                        .last_mut()
                        .unwrap()
                        .insert(semantic_name(&name.ident), value);
                }
                Stmt::Expr(expr, semicolon) => {
                    value = self.expr(expr)?;
                    if semicolon.is_some() {
                        value = Value::Unit;
                    }
                }
                Stmt::Macro(invocation) => {
                    value = self.macro_value(&invocation.mac)?;
                }
                _ => return Err("generator statement requires qualification".into()),
            }
        }
        Ok(value)
    }
    pub(super) fn expr(&mut self, expr: &Expr) -> Result<Value, Fault> {
        self.steps += 1;
        if self.steps > 20_000 {
            return Err("generator evaluation bound exceeded".into());
        }
        match expr {
            Expr::Lit(literal) => Ok(match &literal.lit {
                syn::Lit::Str(s) => Value::Text(s.value()),
                syn::Lit::Char(c) => Value::Text(c.value().to_string()),
                syn::Lit::Int(i) => Value::Int(i.base10_parse().map_err(|e| e.to_string())?),
                syn::Lit::Bool(b) => Value::Bool(b.value),
                _ => return Err("generator literal requires qualification".into()),
            }),
            Expr::Path(path) => self.lookup(&self.program.path(&path.path)),
            Expr::Reference(reference) => self.expr(&reference.expr),
            Expr::Paren(paren) => self.expr(&paren.expr),
            Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Not(_)) => {
                match self.expr(&unary.expr)? {
                    Value::Bool(value) => Ok(Value::Bool(!value)),
                    _ => Err("generator ! requires a boolean".into()),
                }
            }
            Expr::Block(block) => self.block(&block.block),
            Expr::Return(ret) => Err(Fault::Return(if let Some(expr) = &ret.expr {
                self.expr(expr)?
            } else {
                Value::Unit
            })),
            Expr::Try(try_) => match self.expr(&try_.expr)? {
                Value::Optional(Some(value)) => Ok(*value),
                Value::Optional(None) => Err(Fault::Return(Value::Optional(None))),
                _ => Err("generator ? requires an Option".into()),
            },
            Expr::If(if_) => {
                let Value::Bool(condition) = self.expr(&if_.cond)? else {
                    return Err("generator condition is not boolean".into());
                };
                if condition {
                    self.block(&if_.then_branch)
                } else if let Some((_, expr)) = &if_.else_branch {
                    self.expr(expr)
                } else {
                    Ok(Value::Unit)
                }
            }
            Expr::Match(match_) => self.match_value(match_),
            Expr::Binary(binary) => {
                let left = self.expr(&binary.left)?;
                if matches!(binary.op, syn::BinOp::Or(_)) && left == Value::Bool(true) {
                    return Ok(left);
                }
                if matches!(binary.op, syn::BinOp::And(_)) && left == Value::Bool(false) {
                    return Ok(left);
                }
                let right = self.expr(&binary.right)?;
                Ok(Value::Bool(match binary.op {
                    syn::BinOp::Eq(_) => left == right,
                    syn::BinOp::Ne(_) => left != right,
                    syn::BinOp::Lt(_) | syn::BinOp::Ge(_) => {
                        let (Value::Int(a), Value::Int(b)) = (left, right) else {
                            return Err("generator comparison requires integers".into());
                        };
                        if matches!(binary.op, syn::BinOp::Lt(_)) {
                            a < b
                        } else {
                            a >= b
                        }
                    }
                    syn::BinOp::Or(_) | syn::BinOp::And(_) => {
                        let (Value::Bool(a), Value::Bool(b)) = (left, right) else {
                            return Err("generator boolean operator requires booleans".into());
                        };
                        if matches!(binary.op, syn::BinOp::Or(_)) {
                            a || b
                        } else {
                            a && b
                        }
                    }
                    _ => return Err("generator operator requires qualification".into()),
                }))
            }
            Expr::Field(field) => match (self.expr(&field.base)?, &field.member) {
                (Value::Output(bytes), syn::Member::Named(name))
                    if semantic_name(name) == "stdout" =>
                {
                    Ok(Value::Bytes(bytes))
                }
                _ => Err("generator field requires qualification".into()),
            },
            Expr::Call(call) => {
                let Expr::Path(path) = call.func.as_ref() else {
                    return Err("generator indirect call".into());
                };
                let name = self.program.path(&path.path);
                let mut args = Vec::new();
                for argument in &call.args {
                    args.push(self.expr(argument)?);
                }
                self.call(&name, args)
            }
            Expr::MethodCall(call) => self.method(call),
            Expr::Macro(invocation) => self.macro_value(&invocation.mac),
            _ => Err("generator expression requires qualification".into()),
        }
    }
    fn match_value(&mut self, match_: &syn::ExprMatch) -> Result<Value, Fault> {
        let value = self.expr(&match_.expr)?;
        for arm in &match_.arms {
            if arm.guard.is_some() {
                return Err("generator guarded pattern requires qualification".into());
            }
            match (&arm.pat, &value) {
                (syn::Pat::TupleStruct(pattern), Value::Optional(Some(value)))
                    if pattern
                        .path
                        .get_ident()
                        .is_some_and(|name| semantic_name(name) == "Some")
                        && pattern.elems.len() == 1 =>
                {
                    let syn::Pat::Ident(name) = &pattern.elems[0] else {
                        return Err("generator Some pattern requires an identifier".into());
                    };
                    self.frames.last_mut().unwrap().push(BTreeMap::from([(
                        semantic_name(&name.ident),
                        *value.clone(),
                    )]));
                    let result = self.expr(&arm.body);
                    self.frames.last_mut().unwrap().pop();
                    return result;
                }
                (syn::Pat::Path(path), Value::Optional(None))
                    if path
                        .path
                        .get_ident()
                        .is_some_and(|name| semantic_name(name) == "None") =>
                {
                    return self.expr(&arm.body)
                }
                (syn::Pat::Ident(name), Value::Optional(None))
                    if semantic_name(&name.ident) == "None" =>
                {
                    return self.expr(&arm.body)
                }
                _ => {}
            }
        }
        Err("generator match is not exhaustive for its value".into())
    }
    fn call(&mut self, operation: &str, args: Vec<Value>) -> Result<Value, Fault> {
        if self.program.functions.contains_key(operation) {
            if !args.is_empty() {
                return Err("generator helper arguments require qualification".into());
            }
            return self.function(operation);
        }
        match (operation, args.as_slice()) {
            ("Some", [value]) => Ok(Value::Optional(Some(Box::new(value.clone())))),
            ("std::env::var" | "std::env::var_os", [Value::Text(name)]) => {
                let value = self.environment_value(name)?;
                Ok(if operation == "std::env::var_os" {
                    Value::Optional(Some(Box::new(value)))
                } else {
                    Value::Result(Ok(Box::new(value)))
                })
            }
            ("std::path::PathBuf::from", [Value::OutputPath(path)]) => {
                Ok(Value::OutputPath(path.clone()))
            }
            ("std::process::Command::new", [Value::Toolchain(path)]) => {
                Ok(Value::Command(path.clone(), Vec::new()))
            }
            ("std::fs::read" | "std::fs::read_to_string", [Value::Text(path)]) => {
                let path = fs::canonicalize(self.root.join(path)).map_err(|e| e.to_string())?;
                if !path.starts_with(self.root) {
                    return Err("generator read escapes package".into());
                }
                let bytes = fs::read(&path).map_err(|e| e.to_string())?;
                self.effects.reads.insert(path, bytes.clone());
                let value = if operation == "std::fs::read" {
                    Value::Bytes(bytes)
                } else {
                    Value::Text(String::from_utf8(bytes).map_err(|e| e.to_string())?)
                };
                Ok(Value::Result(Ok(Box::new(value))))
            }
            ("std::fs::write", [Value::OutputPath(path), value]) => {
                self.effects.writes.insert(path.clone(), value.bytes()?);
                Ok(Value::Result(Ok(Box::new(Value::Unit))))
            }
            ("std::str::from_utf8", [Value::Bytes(bytes)]) => Ok(Value::Result(
                String::from_utf8(bytes.clone())
                    .map(|s| Box::new(Value::Text(s)))
                    .map_err(|e| e.to_string()),
            )),
            _ => Err(format!("generator call effect requires qualification: {operation}").into()),
        }
    }
}
