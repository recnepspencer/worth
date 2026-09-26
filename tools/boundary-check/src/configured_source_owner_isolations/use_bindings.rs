//! The local names a guarded file's `use` declarations bind. A path or glob
//! written through such a name resolves through it, so `use crate::x as y;`
//! followed by `use y::*;` reaches exactly what `use crate::x::*;` reaches.

use std::collections::BTreeMap;

use syn::visit::Visit;

#[derive(Default)]
pub(super) struct UseBindings {
    /// Each name bound in a module, keyed by that module and the name, and
    /// mapped to the path the declaration wrote.
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
    /// anything but itself.
    pub(super) fn target(&self, module: &[String], name: &str) -> Option<&[String]> {
        self.bindings
            .get(&(module.to_vec(), name.to_owned()))
            .map(Vec::as_slice)
            .filter(|target| *target != [name])
    }

    fn bind(&mut self, module: &[String], name: String, target: Vec<String>) {
        if !target.is_empty() {
            self.bindings.insert((module.to_vec(), name), target);
        }
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
        self.tree(&item.tree, &mut Vec::new());
    }
}
