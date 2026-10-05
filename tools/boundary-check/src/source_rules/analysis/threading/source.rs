//! Classify production syntax that opens a worker or a parallel-computation lane.

use super::super::production_scope::{
    visit_impl_item_in_scope, visit_item_in_scope, visit_trait_item_in_scope, ScopedVisitor,
};
use std::collections::BTreeSet;
use syn::visit::{self, Visit};
use syn::{ExprMethodCall, ImplItem, Item, ItemUse, TraitItem, UseTree};

#[derive(Clone, Default, Eq, PartialEq)]
struct Imports {
    std_modules: BTreeSet<String>,
    thread_modules: BTreeSet<String>,
    thread_builders: BTreeSet<String>,
    thread_spawns: BTreeSet<String>,
    thread_scopes: BTreeSet<String>,
    rayon_modules: BTreeSet<String>,
    rayon_builders: BTreeSet<String>,
}

pub(super) struct ThreadingCalls {
    scope: Vec<String>,
    imports: Imports,
    sites: Vec<(String, String)>,
}

impl ThreadingCalls {
    pub(super) fn for_items(items: &[Item]) -> Self {
        let imports = Imports::from_items(items);
        let mut calls = Self {
            scope: Vec::new(),
            imports,
            sites: Vec::new(),
        };
        for item in items {
            calls.visit_item(item);
        }
        calls
    }

    pub(super) fn take_sites(&mut self) -> Vec<(String, String)> {
        std::mem::take(&mut self.sites)
    }

    fn record(&mut self, kind: &str) {
        self.sites.push((self.scope.join("::"), kind.into()));
    }

    fn classify_call(&self, segments: &[String]) -> Option<&'static str> {
        let names: Vec<_> = segments.iter().map(String::as_str).collect();
        if names == ["tokio", "task", "spawn_blocking"] {
            return Some("async-blocking");
        }
        if names.starts_with(&["tokio", "runtime", "Builder"])
            && matches!(
                names.last(),
                Some(&"new_current_thread" | &"new_multi_thread")
            )
        {
            return Some("async-runtime");
        }
        if names.len() >= 2 && self.imports.std_modules.contains(names[0]) && names[1] == "thread" {
            return classify_thread_tail(&names[2..]);
        }
        if let Some(first) = names.first() {
            if self.imports.thread_modules.contains(*first) {
                return classify_thread_tail(&names[1..]);
            }
            if self.imports.thread_builders.contains(*first)
                && matches!(names.get(1), Some(&"new" | &"default"))
            {
                return Some("thread-builder");
            }
            if names.len() == 1 && self.imports.thread_spawns.contains(*first) {
                return Some("thread-spawn");
            }
            if names.len() == 1 && self.imports.thread_scopes.contains(*first) {
                return Some("thread-scope");
            }
            if (*first == "ThreadPoolBuilder" || self.imports.rayon_builders.contains(*first))
                && matches!(names.get(1), Some(&"new"))
            {
                return Some("thread-pool");
            }
            if (*first == "ThreadPool" && matches!(names.get(1), Some(&"new")))
                || (self.imports.rayon_modules.contains(*first)
                    && names.iter().any(|name| *name == "ThreadPoolBuilder"))
            {
                return Some("thread-pool");
            }
        }
        None
    }
}

fn classify_thread_tail(tail: &[&str]) -> Option<&'static str> {
    match tail {
        ["spawn"] => Some("thread-spawn"),
        ["scope"] => Some("thread-scope"),
        ["Builder", "new" | "default"] => Some("thread-builder"),
        _ => None,
    }
}

impl ScopedVisitor for ThreadingCalls {
    fn scope(&mut self) -> &mut Vec<String> {
        &mut self.scope
    }
}

impl<'ast> Visit<'ast> for ThreadingCalls {
    fn visit_item(&mut self, item: &'ast Item) {
        visit_item_in_scope(self, item)
    }
    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        visit_impl_item_in_scope(self, item)
    }
    fn visit_trait_item(&mut self, item: &'ast TraitItem) {
        visit_trait_item_in_scope(self, item)
    }

    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        let mut paths = Vec::new();
        flatten_use(&item.tree, Vec::new(), &mut paths);
        if paths
            .iter()
            .any(|(path, _)| path.first().is_some_and(|root| root == "rayon"))
        {
            self.record("rayon-import");
        }
        visit::visit_item_use(self, item);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let method = call.method.to_string();
        if method == "spawn_scoped" {
            self.record("thread-spawn");
        } else if method == "into_par_iter" || method == "par_bridge" || method.starts_with("par_")
        {
            self.record("rayon-call");
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        let segments = path
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        if let Some(kind) = self.classify_call(&segments) {
            self.record(kind);
        }
        let first = segments.first();
        if first.is_some_and(|first| self.imports.rayon_modules.contains(first)) {
            self.record("rayon-path");
        }
        visit::visit_expr_path(self, path);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        let identifiers = macro_identifiers(node.tokens.clone());
        for window in identifiers.windows(2) {
            if self.imports.thread_modules.contains(&window[0]) {
                match window[1].as_str() {
                    "spawn" => self.record("thread-spawn"),
                    "scope" => self.record("thread-scope"),
                    "Builder" => self.record("thread-builder"),
                    _ => {}
                }
            }
        }
        if identifiers
            .iter()
            .any(|ident| self.imports.rayon_modules.contains(ident))
        {
            self.record("rayon-path");
        }
        visit::visit_macro(self, node);
    }
}

impl Imports {
    fn from_items(items: &[Item]) -> Self {
        let mut collector = ImportCollector {
            scope: Vec::new(),
            paths: Vec::new(),
        };
        for item in items {
            collector.visit_item(item);
        }
        let mut imports = Self::default();
        imports.std_modules.insert("std".into());
        imports.thread_modules.insert("thread".into());
        for _ in 0..=collector.paths.len() {
            let previous = imports.clone();
            for (path, alias) in &collector.paths {
                imports.admit(path, alias);
            }
            if imports == previous {
                break;
            }
        }
        imports
    }

    fn admit(&mut self, path: &[String], alias: &str) {
        let names: Vec<_> = path.iter().map(String::as_str).collect();
        if names.as_slice() == ["std"] {
            self.std_modules.insert(alias.into());
        }
        if names.len() >= 2 && self.std_modules.contains(names[0]) && names[1] == "thread" {
            match &names[2..] {
                [] | ["self"] => {
                    if alias == "*" {
                        self.thread_spawns.insert("spawn".into());
                        self.thread_scopes.insert("scope".into());
                        self.thread_builders.insert("Builder".into());
                    } else {
                        self.thread_modules.insert(alias.into());
                    }
                }
                ["Builder"] => {
                    self.thread_builders.insert(alias.into());
                }
                ["spawn"] => {
                    self.thread_spawns.insert(alias.into());
                }
                ["scope"] => {
                    self.thread_scopes.insert(alias.into());
                }
                _ => {}
            }
        }
        match names.as_slice() {
            ["rayon"] => {
                self.rayon_modules.insert(alias.into());
            }
            ["rayon", "ThreadPoolBuilder"] => {
                self.rayon_builders.insert(alias.into());
            }
            _ => {}
        }
    }
}

struct ImportCollector {
    scope: Vec<String>,
    paths: Vec<(Vec<String>, String)>,
}

impl ScopedVisitor for ImportCollector {
    fn scope(&mut self) -> &mut Vec<String> {
        &mut self.scope
    }
}

impl<'ast> Visit<'ast> for ImportCollector {
    fn visit_item(&mut self, item: &'ast Item) {
        visit_item_in_scope(self, item)
    }
    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        visit_impl_item_in_scope(self, item)
    }
    fn visit_trait_item(&mut self, item: &'ast TraitItem) {
        visit_trait_item_in_scope(self, item)
    }
    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        let mut paths = Vec::new();
        flatten_use(&item.tree, Vec::new(), &mut paths);
        self.paths.extend(paths);
    }
}

fn flatten_use(tree: &UseTree, mut prefix: Vec<String>, paths: &mut Vec<(Vec<String>, String)>) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            flatten_use(&path.tree, prefix, paths);
        }
        UseTree::Name(name) => {
            prefix.push(name.ident.to_string());
            paths.push((prefix, name.ident.to_string()));
        }
        UseTree::Rename(rename) => {
            prefix.push(rename.ident.to_string());
            paths.push((prefix, rename.rename.to_string()));
        }
        UseTree::Group(group) => {
            for tree in &group.items {
                flatten_use(tree, prefix.clone(), paths);
            }
        }
        UseTree::Glob(_) => paths.push((prefix, "*".into())),
    }
}

fn macro_identifiers(tokens: proc_macro2::TokenStream) -> Vec<String> {
    let mut identifiers = Vec::new();
    for token in tokens {
        match token {
            proc_macro2::TokenTree::Ident(ident) => identifiers.push(ident.to_string()),
            proc_macro2::TokenTree::Group(group) => {
                identifiers.extend(macro_identifiers(group.stream()))
            }
            proc_macro2::TokenTree::Punct(_) | proc_macro2::TokenTree::Literal(_) => {}
        }
    }
    identifiers
}
