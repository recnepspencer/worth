//! Owners whose source must never reach into another owner's module.
//!
//! A guarded root may neither name a path through the isolated owner nor name
//! any item the owner declares, so a re-export through a parent facade is no
//! way around the law. Relative paths resolve against the guarded file's own
//! module, so `super::workflow` from a sibling is the same reach as
//! `crate::primary_graph::workflow`. Methods the owner adds to types it does
//! not own count as its items too, whether called or named by path. A glob
//! import may not reach outside the guarded roots, since a bare name it brings
//! in is not traced back to the owner. A name a `use` binds resolves through
//! its binding, so an alias reaches only what its target reaches.

use std::{collections::BTreeSet, fs, path::Path};

use syn::visit::Visit;

use crate::config::SourceOwnerIsolationConfig;
use crate::diagnostics::{Diagnostic, DiagnosticCode};

mod owned_items;
mod use_bindings;

use owned_items::{is_test_source, test_module_roots, OwnedItemCollector, OwnedItems};
use use_bindings::UseBindings;

/// How many aliases one path may pass through before it stops resolving.
const MAXIMUM_ALIAS_DEPTH: usize = 16;

pub(crate) fn validate_source_owner_isolations(
    workspace: &Path,
    rules: &[SourceOwnerIsolationConfig],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for rule in rules {
        let mut owned = OwnedItems::default();
        let mut declared = BTreeSet::new();
        let mut methods = Vec::new();
        let mut parsed = Vec::new();
        for (path, source) in rust_sources(workspace, &rule.owner_roots, &mut diagnostics) {
            match syn::parse_file(&source) {
                Ok(file) => parsed.push((path, file)),
                Err(_) => diagnostics.push(unparsable(&path)),
            }
        }
        let tests = test_module_roots(&parsed);
        for (path, file) in &parsed {
            if !is_test_source(path, &tests) {
                OwnedItemCollector {
                    owned: &mut owned,
                    declared: &mut declared,
                    methods: &mut methods,
                    module: module_path(path),
                }
                .visit_file(file);
            }
        }
        owned.add_foreign_type_methods(methods, &declared);
        for (path, source) in rust_sources(workspace, &rule.guarded_roots, &mut diagnostics) {
            diagnostics.extend(diagnostics_for_source(&path, &source, &owned, rule));
        }
    }
    diagnostics
}

fn diagnostics_for_source(
    path: &str,
    source: &str,
    owned: &OwnedItems,
    rule: &SourceOwnerIsolationConfig,
) -> Vec<Diagnostic> {
    let Ok(file) = syn::parse_file(source) else {
        return vec![unparsable(path)];
    };
    let module = module_path(path);
    let mut visitor = IsolationVisitor {
        owned,
        forbidden_paths: &rule.forbidden_paths,
        bindings: UseBindings::collect(&file, &module),
        module,
        guarded: rule
            .guarded_roots
            .iter()
            .map(|root| module_path(root))
            .collect(),
        found: BTreeSet::new(),
    };
    visitor.visit_file(&file);
    visitor
        .found
        .into_iter()
        .map(|reference| {
            Diagnostic::new(
                DiagnosticCode::Bc2001BandDependencyViolation,
                path,
                format!(
                    "source reaches isolated owner through `{reference}`: {}",
                    rule.guidance
                ),
            )
        })
        .collect()
}

/// The module a source file declares, read from its path below `src/`.
fn module_path(path: &str) -> Vec<String> {
    let components = path.split(['/', '\\']).collect::<Vec<_>>();
    let start = components
        .iter()
        .rposition(|component| *component == "src")
        .map_or(0, |source| source + 1);
    let mut module = components[start..]
        .iter()
        .map(|component| component.trim_end_matches(".rs").to_owned())
        .collect::<Vec<_>>();
    if module
        .last()
        .is_some_and(|last| matches!(last.as_str(), "mod" | "lib" | "main"))
    {
        module.pop();
    }
    module
}

fn unparsable(path: &str) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::Bc2001BandDependencyViolation,
        path,
        "source could not be parsed for owner isolation",
    )
}

/// Every Rust source under `roots`, each a file or a directory, keyed by its
/// workspace-relative path.
fn rust_sources(
    workspace: &Path,
    roots: &[String],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<(String, String)> {
    let mut sources = Vec::new();
    let mut pending = roots
        .iter()
        .map(|root| workspace.join(root))
        .collect::<Vec<_>>();
    while let Some(path) = pending.pop() {
        let relative = path
            .strip_prefix(workspace)
            .unwrap_or(&path)
            .display()
            .to_string();
        if path.is_dir() {
            match fs::read_dir(&path) {
                Ok(entries) => pending.extend(entries.flatten().map(|entry| entry.path())),
                Err(error) => diagnostics.push(unreadable(&relative, &error)),
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            match fs::read_to_string(&path) {
                Ok(source) => sources.push((relative, source)),
                Err(error) => diagnostics.push(unreadable(&relative, &error)),
            }
        } else if !path.exists() {
            diagnostics.push(Diagnostic::new(
                DiagnosticCode::Bc2001BandDependencyViolation,
                relative,
                "configured owner isolation root does not exist",
            ));
        }
    }
    sources
}

fn unreadable(path: &str, error: &std::io::Error) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::Bc2001BandDependencyViolation,
        path,
        format!("configured owner isolation source could not be read: {error}"),
    )
}

struct IsolationVisitor<'a> {
    owned: &'a OwnedItems,
    forbidden_paths: &'a [Vec<String>],
    bindings: UseBindings,
    module: Vec<String>,
    /// The modules the guarded roots declare.
    guarded: Vec<Vec<String>>,
    found: BTreeSet<String>,
}

impl IsolationVisitor<'_> {
    /// Checks one path, as written and as resolved from this file's module.
    fn check_segments(&mut self, segments: &[String]) {
        let resolved = self.resolve(segments);
        for forbidden in self.forbidden_paths {
            if [segments, resolved.as_slice()].iter().any(|candidate| {
                candidate
                    .windows(forbidden.len())
                    .any(|window| window == forbidden.as_slice())
            }) {
                self.found.insert(forbidden.join("::"));
            }
        }
        if let (true, Some(last)) = (segments.len() > 1, segments.last()) {
            if self.owned.reaches_value(last, &self.module) || self.owned.methods.contains(last) {
                self.found.insert(last.clone());
            }
        }
    }

    /// The crate-absolute form of a path, or the path as written when it
    /// names no module of this crate.
    fn resolve(&self, segments: &[String]) -> Vec<String> {
        self.resolve_local(segments)
            .unwrap_or_else(|| segments.to_vec())
    }

    /// The crate-absolute form of a `crate::`, `self::` or `super::` path, or
    /// of one that starts with a name a `use` in this module binds. An alias
    /// chain too deep to follow resolves nowhere, which a glob refuses.
    fn resolve_local(&self, segments: &[String]) -> Option<Vec<String>> {
        let mut segments = segments.to_vec();
        for _ in 0..MAXIMUM_ALIAS_DEPTH {
            match segments.first().map(String::as_str) {
                Some("crate") => return Some(segments[1..].to_vec()),
                Some("self") => return Some([&self.module[..], &segments[1..]].concat()),
                Some("super") => {
                    let depth = segments.iter().take_while(|s| *s == "super").count();
                    let kept = self.module.len().saturating_sub(depth);
                    return Some([&self.module[..kept], &segments[depth..]].concat());
                }
                Some(name) => {
                    let target = self.bindings.target(&self.module, name)?;
                    segments = [target, &segments[1..]].concat();
                }
                None => return None,
            }
        }
        Some(Vec::new())
    }

    fn visit_tokens(&mut self, tokens: proc_macro2::TokenStream) {
        let mut segments = Vec::new();
        for token in tokens {
            match token {
                proc_macro2::TokenTree::Ident(identifier) => {
                    self.visit_ident(&identifier);
                    segments.push(identifier.to_string());
                }
                proc_macro2::TokenTree::Punct(punct) if punct.as_char() == ':' => {}
                proc_macro2::TokenTree::Group(group) => {
                    self.check_segments(&std::mem::take(&mut segments));
                    self.visit_tokens(group.stream());
                }
                _ => self.check_segments(&std::mem::take(&mut segments)),
            }
        }
        self.check_segments(&segments);
    }

    fn visit_use_tree_segments(&mut self, tree: &syn::UseTree, prefix: &mut Vec<String>) {
        match tree {
            syn::UseTree::Path(path) => {
                prefix.push(path.ident.to_string());
                self.visit_use_tree_segments(&path.tree, prefix);
                prefix.pop();
            }
            syn::UseTree::Name(name) => self.check_leaf(prefix, &name.ident),
            syn::UseTree::Rename(rename) => self.check_leaf(prefix, &rename.ident),
            syn::UseTree::Glob(_) => {
                let reported = self.found.len();
                self.check_segments(prefix);
                if self.found.len() == reported {
                    self.check_glob(prefix);
                }
            }
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    self.visit_use_tree_segments(tree, prefix);
                }
            }
        }
    }

    /// A crate-relative glob import must stay inside a guarded root; a glob
    /// from an enclosing facade could bring owner values in by bare name.
    fn check_glob(&mut self, prefix: &[String]) {
        let Some(resolved) = self.resolve_local(prefix) else {
            return;
        };
        if resolved.is_empty() || !self.guarded.iter().any(|root| resolved.starts_with(root)) {
            self.found.insert(format!("{}::*", prefix.join("::")));
        }
    }

    fn check_leaf(&mut self, prefix: &[String], leaf: &proc_macro2::Ident) {
        let mut segments = prefix.to_vec();
        segments.push(leaf.to_string());
        self.check_segments(&segments);
    }
}

impl Visit<'_> for IsolationVisitor<'_> {
    fn visit_ident(&mut self, identifier: &proc_macro2::Ident) {
        let identifier = identifier.to_string();
        if self.owned.types.contains(&identifier) {
            self.found.insert(identifier);
        }
    }

    fn visit_path(&mut self, path: &syn::Path) {
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        self.check_segments(&segments);
        syn::visit::visit_path(self, path);
    }

    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        let method = call.method.to_string();
        if self.owned.methods.contains(&method) {
            self.found.insert(method);
        }
        syn::visit::visit_expr_method_call(self, call);
    }

    /// Relative paths inside an inline module resolve against that module.
    fn visit_item_mod(&mut self, item: &syn::ItemMod) {
        let inline = item.content.is_some();
        if inline {
            self.module.push(item.ident.to_string());
        }
        syn::visit::visit_item_mod(self, item);
        if inline {
            self.module.pop();
        }
    }

    fn visit_item_use(&mut self, item: &syn::ItemUse) {
        let mut prefix = Vec::new();
        self.visit_use_tree_segments(&item.tree, &mut prefix);
        syn::visit::visit_item_use(self, item);
    }

    fn visit_macro(&mut self, node: &syn::Macro) {
        syn::visit::visit_macro(self, node);
        self.visit_tokens(node.tokens.clone());
    }
}

#[cfg(test)]
#[path = "configured_source_owner_isolations/tests.rs"]
mod tests;
