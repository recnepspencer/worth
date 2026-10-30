//! Resolve constructor expressions and lexical fixture calls in one module.
use super::super::runtime_roots::Registry;
use std::collections::{BTreeMap, BTreeSet};
use syn::visit::{self, Visit};

pub(super) struct Body<'a> {
    pub(super) package: &'a str,
    pub(super) variables: BTreeMap<String, String>,
    pub(super) roots: &'a Registry,
    pub(super) imports: &'a BTreeMap<String, String>,
    pub(super) enum_variants: &'a BTreeMap<String, String>,
    pub(super) foreign: &'a BTreeSet<String>,
    pub(super) scope: &'a [String],
    pub(super) calls: BTreeSet<String>,
    pub(super) constructors: BTreeSet<String>,
}
impl Body<'_> {
    fn expanded(&self, path: &syn::Path) -> String {
        let mut parts = path.segments.iter().map(|p| p.ident.to_string());
        let Some(first) = parts.next() else {
            return String::new();
        };
        std::iter::once(self.imports.get(&first).cloned().unwrap_or(first))
            .chain(parts)
            .collect::<Vec<_>>()
            .join("::")
    }
    fn identity(&self, path: &str) -> Option<String> {
        let path = self.enum_variants.get(path).map_or(path, String::as_str);
        let parts: Vec<_> = path.split("::").collect();
        let name = parts
            .iter()
            .rev()
            .find(|p| p.chars().next().is_some_and(char::is_uppercase))?;
        let package = if parts.first() == Some(&"crate") || parts.len() == 1 {
            self.package
        } else {
            parts[0]
        };
        Some(format!("{package}::{name}"))
    }
    fn inferred(&self, expression: &syn::Expr) -> Option<String> {
        match expression {
            syn::Expr::Path(path) => self
                .variables
                .get(&path.path.get_ident()?.to_string())
                .cloned(),
            syn::Expr::Call(call) => {
                let syn::Expr::Path(path) = &*call.func else {
                    return None;
                };
                let expanded = self.expanded(&path.path);
                let method = path.path.segments.last()?.ident.to_string();
                if let Some(identity) = self.identity(&expanded) {
                    self.roots
                        .factories
                        .returns(&identity, &method)
                        .or_else(|| {
                            Some(if method == "builder" {
                                format!("{identity}Builder")
                            } else {
                                identity
                            })
                        })
                } else {
                    let package = expanded.split("::").next()?;
                    self.roots.factories.free_returns(package, &method)
                }
            }
            syn::Expr::MethodCall(call) => {
                let receiver = self.inferred(&call.receiver)?;
                let method = call.method.to_string();
                if ["unwrap", "expect"].contains(&method.as_str()) {
                    Some(receiver)
                } else {
                    self.roots.factories.returns(&receiver, &method)
                }
            }
            syn::Expr::Await(expr) => self.inferred(&expr.base),
            syn::Expr::Paren(expr) => self.inferred(&expr.expr),
            syn::Expr::Reference(expr) => self.inferred(&expr.expr),
            _ => None,
        }
    }
    fn returned_root(&mut self, expression: &syn::Expr) {
        if let syn::Expr::Call(call) = expression {
            if let syn::Expr::Path(path) = &*call.func {
                if self.enum_variants.contains_key(&self.expanded(&path.path)) {
                    // Explicit enum construction is classified by its owner below.
                    return;
                }
            }
        }
        if let Some(identity) = self.inferred(expression) {
            if self.roots.matching(&identity).is_some() {
                self.constructors.insert(format!("{identity}::<factory>"));
            }
        }
    }
    fn local_call(&mut self, path: &syn::Path) {
        let raw: Vec<_> = path.segments.iter().map(|p| p.ident.to_string()).collect();
        let Some(first) = raw.first() else {
            return;
        };
        let expanded = self.imports.get(first).map_or_else(
            || raw.join("::"),
            |alias| {
                std::iter::once(alias.as_str())
                    .chain(raw.iter().skip(1).map(String::as_str))
                    .collect::<Vec<_>>()
                    .join("::")
            },
        );
        let mut parts: Vec<String> = expanded.split("::").map(str::to_owned).collect();
        let Some(first) = parts.first() else {
            return;
        };
        if self.foreign.contains(first) || ["std", "core", "alloc"].contains(&first.as_str()) {
            return;
        }
        let lexical = parts.len() == 1;
        let mut scope = self.scope.to_vec();
        if first == "Self" {
            parts.remove(0);
        } else if first == "crate" {
            scope.clear();
            parts.remove(0);
        } else {
            while parts.first().is_some_and(|p| p == "self" || p == "super") {
                if parts.remove(0) == "super" {
                    scope.pop();
                }
            }
        }
        scope.extend(parts);
        self.calls.insert(format!(
            "{}{path}",
            if lexical { "?" } else { "" },
            path = scope.join("::")
        ));
    }
    fn construction(&mut self, path: &syn::Path, struct_literal: bool) {
        let parts: Vec<_> = path.segments.iter().map(|p| p.ident.to_string()).collect();
        let Some(first) = parts.first() else {
            return;
        };
        let spelling = parts.join("::");
        let expanded = self.imports.get(first).map_or_else(
            || spelling.clone(),
            |prefix| {
                std::iter::once(prefix.as_str())
                    .chain(parts.iter().skip(1).map(String::as_str))
                    .collect::<Vec<_>>()
                    .join("::")
            },
        );
        // The registry names work hosts, including roots owned by this crate.
        // Imported keys, enums, budgets and other vocabulary never qualify.
        let owner = self.enum_variants.get(&expanded).unwrap_or(&expanded);
        let returns_configuration = !self.enum_variants.contains_key(&expanded)
            && !struct_literal
            && path.segments.last().is_some_and(|last| {
                let method = last.ident.to_string();
                method == "builder"
                    || self
                        .identity(&expanded)
                        .and_then(|ty| self.roots.factories.returns(&ty, &method))
                        .is_some_and(|ty| self.roots.matching(&ty).is_none())
            });
        if !returns_configuration {
            if self.roots.matching(owner).is_some() {
                self.constructors.insert(expanded);
            } else if !self.imports.contains_key(first) {
                for (name, namespace) in self.imports {
                    if name.starts_with("*foreign:") {
                        let candidate = format!("{namespace}::<glob>::{owner}");
                        if self.roots.matching(&candidate).is_some() {
                            self.constructors.insert(candidate);
                        }
                    }
                }
            }
        }
    }
}
impl<'ast> Visit<'ast> for Body<'_> {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let (syn::Pat::Ident(name), Some(init)) = (&local.pat, &local.init) {
            if let Some(identity) = self.inferred(&init.expr) {
                self.variables.insert(name.ident.to_string(), identity);
            }
        }
        visit::visit_local(self, local);
    }
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        self.returned_root(&syn::Expr::MethodCall(call.clone()));
        visit::visit_expr_method_call(self, call);
    }
    fn visit_expr_path(&mut self, expression: &'ast syn::ExprPath) {
        if let Some(last) = expression.path.segments.last() {
            let name = last.ident.to_string();
            if name.chars().next().is_some_and(char::is_uppercase)
                && !["Self", "None", "Some", "Ok", "Err"].contains(&name.as_str())
                && (name.len() == 1 || name.chars().any(char::is_lowercase))
            {
                self.construction(&expression.path, false);
            }
        }
        visit::visit_expr_path(self, expression);
    }
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        self.returned_root(&syn::Expr::Call(call.clone()));
        if let syn::Expr::Path(path) = &*call.func {
            self.construction(&path.path, false);
            self.local_call(&path.path);
        }
        visit::visit_expr_call(self, call);
    }
    fn visit_expr_struct(&mut self, value: &'ast syn::ExprStruct) {
        self.construction(&value.path, true);
        visit::visit_expr_struct(self, value);
    }
    fn visit_macro(&mut self, invocation: &'ast syn::Macro) {
        // Constructors inside assertions and setup macros still execute.
        if let Ok(expression) = syn::parse2::<syn::Expr>(invocation.tokens.clone()) {
            self.visit_expr(&expression);
        } else if let Ok(arguments) = invocation.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in arguments {
                self.visit_expr(&argument);
            }
        }
    }
}
