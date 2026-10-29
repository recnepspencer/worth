//! Resolve constructor expressions and lexical fixture calls in one module.
use std::collections::{BTreeMap, BTreeSet};
use syn::visit::{self, Visit};

pub(super) struct Body<'a> {
    pub(super) imports: &'a BTreeMap<String, String>,
    pub(super) foreign: &'a BTreeSet<String>,
    pub(super) scope: &'a [String],
    pub(super) calls: BTreeSet<String>,
    pub(super) constructors: BTreeSet<String>,
}
impl Body<'_> {
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
        let root = expanded.split("::").next().unwrap();
        let glob = self
            .imports
            .iter()
            .find(|(name, _)| name.starts_with("*foreign:"))
            .map(|(_, root)| root);
        let via_glob = !self.imports.contains_key(first)
            && first.chars().next().is_some_and(char::is_uppercase)
            && ![
                "Self", "Vec", "String", "Option", "Result", "Box", "Arc", "Rc",
            ]
            .contains(&first.as_str())
            && glob.is_some();
        let names_type = expanded
            .split("::")
            .any(|segment| segment.chars().next().is_some_and(char::is_uppercase));
        let external = (self.foreign.contains(root) || root == "unresolved" || via_glob)
            && (struct_literal || names_type);
        let runtime = parts.iter().any(|p| {
            [
                "WorthQuery",
                "BankIdentityRuntime",
                "ApplicationRuntime",
                "Session",
                "RuntimeWorld",
            ]
            .contains(&p.as_str())
        });
        if external || runtime {
            self.constructors.insert(if via_glob {
                format!("{}::<glob>::{expanded}", glob.unwrap())
            } else {
                expanded
            });
        }
    }
}
impl<'ast> Visit<'ast> for Body<'_> {
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
