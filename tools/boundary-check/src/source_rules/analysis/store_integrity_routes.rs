//! C.9 raw-codec fence for ordinary Store and recovery production sources.
//!
//! This is a structural rule, not a Rust test fingerprint of current consumers.
//! Every source under each owner is checked, including new or renamed modules.

mod policy;
#[cfg(test)]
mod tests;

use super::crate_modules::{GovernedCrate, ModuleGraph};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use syn::visit::{self, Visit};

use crate::diagnostics::{Diagnostic, DiagnosticCode};

pub(super) fn enforce(governed: &GovernedCrate, graph: &ModuleGraph) -> Vec<Diagnostic> {
    if ![
        "worth-store",
        "worth-store-recovery-runtime",
        "worth-store-recovery-physics",
    ]
    .contains(&governed.package.as_str())
    {
        return Vec::new();
    }
    let nodes: Vec<_> = graph
        .modules
        .iter()
        .filter(|(path, _)| !super::authority_import_explicitness::module_is_test_only(graph, path))
        .map(|(_, node)| node)
        .collect();
    let mut aliases = Aliases::default();
    for node in &nodes {
        for item in &node.items {
            aliases.visit_item(item);
        }
    }
    let mut violations = Vec::new();
    for node in nodes {
        let path = format!("{}/{}", governed.relative_crate_root, node.relative_source);
        let mut visitor = Routes {
            path: &path,
            aliases: &aliases.0,
            found: BTreeSet::new(),
            projection_owner: None,
            projection_method: None,
        };
        for item in &node.items {
            visitor.visit_item(item);
        }
        violations.extend(visitor.found.into_iter().map(|route| {
            denial(
                Path::new(&path),
                format!("raw route `{route}` requires the family admitted view"),
            )
        }));
    }
    violations.sort_by(Diagnostic::compare_subject_message);
    violations.dedup_by(|left, right| left.has_same_subject_message(right));
    violations
}

fn denial(path: &Path, message: String) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::Bc2001BandDependencyViolation,
        path.display().to_string(),
        format!("C.9 integrity admission boundary: {message}"),
    )
}

fn test_only(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("cfg")
            && attribute
                .parse_args::<syn::Path>()
                .is_ok_and(|path| path.is_ident("test"))
    })
}

#[cfg(test)]
fn check_source(path: &str, source: &str) -> Vec<String> {
    let file = match syn::parse_file(source) {
        Ok(file) => file,
        Err(error) => return vec![format!("cannot parse governed source: {error}")],
    };
    let mut aliases = Aliases::default();
    aliases.visit_file(&file);
    let mut visitor = Routes {
        path,
        aliases: &aliases.0,
        found: BTreeSet::new(),
        projection_owner: None,
        projection_method: None,
    };
    visitor.visit_file(&file);
    visitor.found.into_iter().map(|route| format!("raw route `{route}` requires the family admitted view; only named canonical writer/dirty-frame mechanisms are exempt")).collect()
}

#[derive(Default)]
struct Aliases(BTreeMap<String, String>);

impl Visit<'_> for Aliases {
    fn visit_use_rename(&mut self, rename: &syn::UseRename) {
        self.0
            .insert(rename.rename.to_string(), rename.ident.to_string());
    }

    fn visit_item_type(&mut self, alias: &syn::ItemType) {
        if let syn::Type::Path(target) = alias.ty.as_ref() {
            if let Some(segment) = target.path.segments.last() {
                self.0
                    .insert(alias.ident.to_string(), segment.ident.to_string());
            }
        }
    }
}

struct Routes<'a> {
    path: &'a str,
    aliases: &'a BTreeMap<String, String>,
    found: BTreeSet<String>,
    projection_owner: Option<String>,
    projection_method: Option<String>,
}

impl Routes<'_> {
    fn resolve(&self, name: &str) -> String {
        let mut name = name.to_owned();
        for _ in 0..self.aliases.len() {
            let Some(target) = self.aliases.get(&name) else {
                break;
            };
            name = target.clone();
        }
        name
    }

    fn identifier(&mut self, name: &str) {
        let name = self.resolve(name);
        if policy::raw_function(&name) && !policy::allows(self.path, &name) {
            self.found.insert(name);
        }
    }

    fn pair(&mut self, owner: &str, method: &str) {
        let owner = self.resolve(owner);
        let route = format!("{owner}::{method}");
        if policy::raw_method(&owner, method)
            && !policy::allows(self.path, &route)
            && !policy::admitted_projection(
                self.path,
                self.projection_owner.as_deref(),
                self.projection_method.as_deref(),
                &route,
            )
        {
            self.found.insert(route);
        }
    }

    fn tokens(&mut self, tokens: proc_macro2::TokenStream) {
        let mut owner: Option<String> = None;
        let mut colons = 0;
        for token in tokens {
            match token {
                proc_macro2::TokenTree::Ident(ident) => {
                    let name = ident.to_string();
                    self.identifier(&name);
                    if colons == 2 {
                        if let Some(previous) = owner.as_deref() {
                            self.pair(previous, &name);
                        }
                    }
                    owner = Some(name);
                    colons = 0;
                }
                proc_macro2::TokenTree::Punct(punct) if punct.as_char() == ':' => colons += 1,
                proc_macro2::TokenTree::Group(group) => {
                    self.tokens(group.stream());
                    owner = None;
                    colons = 0;
                }
                _ => {
                    owner = None;
                    colons = 0;
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for Routes<'_> {
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let prior = self.projection_owner.take();
        if let syn::Type::Path(ty) = item.self_ty.as_ref() {
            self.projection_owner = ty
                .path
                .segments
                .last()
                .map(|segment| segment.ident.to_string());
        }
        visit::visit_item_impl(self, item);
        self.projection_owner = prior;
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let prior = self.projection_method.replace(item.sig.ident.to_string());
        visit::visit_impl_item_fn(self, item);
        self.projection_method = prior;
    }

    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if !test_only(&item.attrs) {
            visit::visit_item_mod(self, item);
        }
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        if !test_only(&item.attrs) {
            visit::visit_item_fn(self, item);
        }
    }

    fn visit_ident(&mut self, identifier: &'ast proc_macro2::Ident) {
        self.identifier(&identifier.to_string());
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        let segments: Vec<_> = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        for pair in segments.windows(2) {
            self.pair(&pair[0], &pair[1]);
        }
        visit::visit_path(self, path);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        visit::visit_macro(self, node);
        self.tokens(node.tokens.clone());
    }
}
