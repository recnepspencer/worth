//! Owners whose source must never reach into another owner's module.
//!
//! A guarded root may neither name a path through the isolated owner nor name
//! any item the owner declares, so a re-export through a parent facade is no
//! way around the law. Relative paths resolve against the guarded file's own
//! module, so `super::workflow` from a sibling is the same reach as
//! `crate::primary_graph::workflow`.

use std::{collections::BTreeSet, fs, path::Path};

use syn::visit::Visit;

use crate::config::SourceOwnerIsolationConfig;
use crate::diagnostics::{Diagnostic, DiagnosticCode};

pub(crate) fn validate_source_owner_isolations(
    workspace: &Path,
    rules: &[SourceOwnerIsolationConfig],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for rule in rules {
        let mut owned = OwnedItems::default();
        for (path, source) in rust_sources(workspace, &rule.owner_roots, &mut diagnostics) {
            match syn::parse_file(&source) {
                Ok(file) => OwnedItemCollector { owned: &mut owned }.visit_file(&file),
                Err(_) => diagnostics.push(unparsable(&path)),
            }
        }
        for (path, source) in rust_sources(workspace, &rule.guarded_roots, &mut diagnostics) {
            diagnostics.extend(diagnostics_for_source(&path, &source, &owned, rule));
        }
    }
    diagnostics
}

/// What an isolated owner declares.
#[derive(Default)]
struct OwnedItems {
    /// Type names, refused wherever they appear.
    types: BTreeSet<String>,
    /// Visible function, constant, static and exported macro names, refused
    /// when a qualified path or an import reaches them.
    values: BTreeSet<String>,
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
    let mut visitor = IsolationVisitor {
        owned,
        forbidden_paths: &rule.forbidden_paths,
        module: module_path(path),
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

struct OwnedItemCollector<'a> {
    owned: &'a mut OwnedItems,
}

impl OwnedItemCollector<'_> {
    fn value(&mut self, visibility: &syn::Visibility, identifier: &proc_macro2::Ident) {
        if !matches!(visibility, syn::Visibility::Inherited) {
            self.owned.values.insert(identifier.to_string());
        }
    }
}

impl Visit<'_> for OwnedItemCollector<'_> {
    fn visit_item_struct(&mut self, item: &syn::ItemStruct) {
        self.owned.types.insert(item.ident.to_string());
    }

    fn visit_item_enum(&mut self, item: &syn::ItemEnum) {
        self.owned.types.insert(item.ident.to_string());
    }

    fn visit_item_trait(&mut self, item: &syn::ItemTrait) {
        self.owned.types.insert(item.ident.to_string());
    }

    fn visit_item_type(&mut self, item: &syn::ItemType) {
        self.owned.types.insert(item.ident.to_string());
    }

    fn visit_item_union(&mut self, item: &syn::ItemUnion) {
        self.owned.types.insert(item.ident.to_string());
    }

    fn visit_item_fn(&mut self, item: &syn::ItemFn) {
        self.value(&item.vis, &item.sig.ident);
    }

    fn visit_item_const(&mut self, item: &syn::ItemConst) {
        self.value(&item.vis, &item.ident);
    }

    fn visit_item_static(&mut self, item: &syn::ItemStatic) {
        self.value(&item.vis, &item.ident);
    }

    fn visit_item_macro(&mut self, item: &syn::ItemMacro) {
        let exported = item
            .attrs
            .iter()
            .any(|attribute| attribute.path().is_ident("macro_export"));
        if let (true, Some(identifier)) = (exported, &item.ident) {
            self.owned.values.insert(identifier.to_string());
        }
    }
}

struct IsolationVisitor<'a> {
    owned: &'a OwnedItems,
    forbidden_paths: &'a [Vec<String>],
    module: Vec<String>,
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
            if self.owned.values.contains(last) {
                self.found.insert(last.clone());
            }
        }
    }

    /// The crate-absolute form of a `crate::`, `self::` or `super::` path.
    fn resolve(&self, segments: &[String]) -> Vec<String> {
        match segments.first().map(String::as_str) {
            Some("crate") => segments[1..].to_vec(),
            Some("self") => [&self.module[..], &segments[1..]].concat(),
            Some("super") => {
                let depth = segments.iter().take_while(|s| *s == "super").count();
                let kept = self.module.len().saturating_sub(depth);
                [&self.module[..kept], &segments[depth..]].concat()
            }
            _ => segments.to_vec(),
        }
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
            syn::UseTree::Glob(_) => self.check_segments(prefix),
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    self.visit_use_tree_segments(tree, prefix);
                }
            }
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
