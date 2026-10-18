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
mod path_resolution;
mod test_sources;
mod use_bindings;

use owned_items::{OwnedItemCollector, OwnedItems};
use test_sources::TestSources;
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
            match crate::source_syntax::parse_named(&workspace.join(&path), &source) {
                Ok(file) => parsed.push((path, file)),
                Err(_) => diagnostics.push(unparsable(&path)),
            }
        }
        let tests = TestSources::new(&parsed);
        for (path, file) in &parsed {
            if !tests.contains(path) {
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
            diagnostics.extend(diagnostics_for_syntax(
                &path,
                crate::source_syntax::parse_named(&workspace.join(&path), &source),
                &owned,
                rule,
                path_resolution::external_crates(workspace, &path),
            ));
        }
    }
    diagnostics
}

#[cfg(test)]
fn diagnostics_for_source(
    path: &str,
    source: &str,
    owned: &OwnedItems,
    rule: &SourceOwnerIsolationConfig,
) -> Vec<Diagnostic> {
    diagnostics_for_syntax(
        path,
        crate::source_syntax::parse_file(source).map(std::rc::Rc::new),
        owned,
        rule,
        path_resolution::standard_crates(),
    )
}

fn diagnostics_for_syntax(
    path: &str,
    syntax: syn::Result<std::rc::Rc<syn::File>>,
    owned: &OwnedItems,
    rule: &SourceOwnerIsolationConfig,
    external_crates: BTreeSet<String>,
) -> Vec<Diagnostic> {
    let Ok(file) = syntax else {
        return vec![unparsable(path)];
    };
    let module = module_path(path);
    let mut visitor = IsolationVisitor {
        owned,
        external_crates,
        generic_types: BTreeSet::new(),
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
            match crate::source_documents::read_to_string(&path) {
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
    external_crates: BTreeSet<String>,
    generic_types: BTreeSet<String>,
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
        if let (true, Some(last)) = (segments.len() > 1, segments.last())
            && self.owned.reaches_method(last, &self.module)
        {
            self.found.insert(last.clone());
        }
        if self.independent_root(segments) {
            return;
        }
        for segment in segments {
            if self.owned.types.contains(segment) {
                self.found.insert(segment.clone());
            }
        }
        if let (true, Some(last)) = (segments.len() > 1, segments.last()) {
            if self.owned.reaches_value(last, &self.module) {
                self.found.insert(last.clone());
            }
        }
    }

    fn visit_tokens(&mut self, tokens: proc_macro2::TokenStream) {
        let mut segments = Vec::new();
        let mut qualified = false;
        let mut tokens = tokens.into_iter().peekable();
        while let Some(token) = tokens.next() {
            match token {
                proc_macro2::TokenTree::Ident(identifier) => {
                    if !qualified {
                        self.check_segments(&std::mem::take(&mut segments));
                    }
                    segments.push(identifier.to_string());
                    qualified = false;
                }
                proc_macro2::TokenTree::Punct(punct)
                    if !qualified
                        && punct.as_char() == ':'
                        && punct.spacing() == proc_macro2::Spacing::Joint
                        && matches!(tokens.peek(), Some(proc_macro2::TokenTree::Punct(next)) if next.as_char() == ':') =>
                {
                    tokens.next();
                    qualified = !segments.is_empty();
                }
                other => {
                    self.check_segments(&std::mem::take(&mut segments));
                    qualified = false;
                    if let proc_macro2::TokenTree::Group(group) = other {
                        self.visit_tokens(group.stream());
                    }
                }
            }
        }
        self.check_segments(&segments);
    }
}

impl Visit<'_> for IsolationVisitor<'_> {
    fn visit_item_impl(&mut self, item: &syn::ItemImpl) {
        let previous = self.generic_types.clone();
        self.generic_types.extend(
            item.generics
                .type_params()
                .map(|parameter| parameter.ident.to_string()),
        );
        syn::visit::visit_item_impl(self, item);
        self.generic_types = previous;
    }

    fn visit_item_fn(&mut self, item: &syn::ItemFn) {
        let previous = self.generic_types.clone();
        self.generic_types.extend(
            item.sig
                .generics
                .type_params()
                .map(|parameter| parameter.ident.to_string()),
        );
        syn::visit::visit_item_fn(self, item);
        self.generic_types = previous;
    }

    fn visit_impl_item_fn(&mut self, item: &syn::ImplItemFn) {
        let previous = self.generic_types.clone();
        self.generic_types.extend(
            item.sig
                .generics
                .type_params()
                .map(|parameter| parameter.ident.to_string()),
        );
        syn::visit::visit_impl_item_fn(self, item);
        self.generic_types = previous;
    }

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
        if self.independent_root(&segments) {
            for segment in &path.segments {
                self.visit_path_arguments(&segment.arguments);
            }
        } else {
            syn::visit::visit_path(self, path);
        }
    }

    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        let method = call.method.to_string();
        if self.owned.reaches_method(&method, &self.module) {
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
    }

    fn visit_macro(&mut self, node: &syn::Macro) {
        syn::visit::visit_macro(self, node);
        self.visit_tokens(node.tokens.clone());
    }
}

#[cfg(test)]
#[path = "configured_source_owner_isolations/tests.rs"]
mod tests;
