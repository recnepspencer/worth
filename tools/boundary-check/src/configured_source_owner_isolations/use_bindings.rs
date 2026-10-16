//! The local names a guarded file's `use` declarations bind. A path or glob
//! written through such a name resolves through it, so `use crate::x as y;`
//! followed by `use y::*;` reaches exactly what `use crate::x::*;` reaches.
//! A name one module binds to two paths, say once in a function body, is
//! ambiguous and resolves nowhere.

use std::collections::BTreeMap;

use syn::visit::Visit;

#[derive(Default)]
pub(super) struct UseBindings {
    /// Each name bound in a module, keyed by that module and the name, and
    /// mapped to the path the declaration wrote, or to nothing when two
    /// declarations bind it differently.
    bindings: BTreeMap<(Vec<String>, String), Vec<String>>,
}

impl UseBindings {
    pub(super) fn collect(file: &syn::File, module: &[String]) -> Self {
        let mut collector = Collector {
            bindings: Self::default(),
            module: module.to_vec(),
        };
        collector.visit_file(file);
        collector.bindings
    }

    /// The path `name` was bound to in `module`, if a `use` there bound it to
    /// anything but itself; an empty path when the binding is ambiguous.
    pub(super) fn target(&self, module: &[String], name: &str) -> Option<&[String]> {
        self.bindings
            .get(&(module.to_vec(), name.to_owned()))
            .map(Vec::as_slice)
            .filter(|target| *target != [name])
    }

    fn bind(&mut self, module: &[String], name: String, target: Vec<String>) {
        if target.is_empty() {
            return;
        }
        self.bindings
            .entry((module.to_vec(), name))
            .and_modify(|bound| {
                if *bound != target {
                    bound.clear();
                }
            })
            .or_insert(target);
    }
}

struct Collector {
    bindings: UseBindings,
    module: Vec<String>,
}

impl Collector {
    fn tree(&mut self, tree: &syn::UseTree, prefix: &mut Vec<String>) {
        match tree {
            syn::UseTree::Path(path) => {
                prefix.push(path.ident.to_string());
                self.tree(&path.tree, prefix);
                prefix.pop();
            }
            syn::UseTree::Name(name) => {
                let name = name.ident.to_string();
                if name == "self" {
                    if let Some(last) = prefix.last() {
                        self.bindings
                            .bind(&self.module, last.clone(), prefix.clone());
                    }
                } else {
                    let mut target = prefix.clone();
                    target.push(name.clone());
                    self.bindings.bind(&self.module, name, target);
                }
            }
            syn::UseTree::Rename(rename) => {
                let mut target = prefix.clone();
                if rename.ident != "self" {
                    target.push(rename.ident.to_string());
                }
                self.bindings
                    .bind(&self.module, rename.rename.to_string(), target);
            }
            syn::UseTree::Glob(_) => {}
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    self.tree(tree, prefix);
                }
            }
        }
    }
}

impl Visit<'_> for Collector {
    fn visit_item_extern_crate(&mut self, item: &syn::ItemExternCrate) {
        let name = item.rename.as_ref().map_or(&item.ident, |(_, alias)| alias);
        let target = if item.ident == "self" {
            "crate".to_owned()
        } else {
            item.ident.to_string()
        };
        self.bindings
            .bind(&self.module, name.to_string(), vec![target]);
    }

    fn visit_item_mod(&mut self, item: &syn::ItemMod) {
        self.bindings.bind(
            &self.module,
            item.ident.to_string(),
            vec!["self".into(), item.ident.to_string()],
        );
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
        self.tree(&item.tree, &mut Vec::new());
    }
}

impl super::IsolationVisitor<'_> {
    pub(super) fn visit_use_tree_segments(
        &mut self,
        tree: &syn::UseTree,
        prefix: &mut Vec<String>,
    ) {
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
