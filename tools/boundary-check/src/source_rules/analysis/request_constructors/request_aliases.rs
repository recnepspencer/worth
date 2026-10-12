//! Resolve the request type vocabulary before checking its associated constructors.
use super::{production_modules, ModuleGraph};
use std::collections::BTreeSet;
use syn::visit::{self, Visit};

fn name(ident: &proc_macro2::Ident) -> String {
    ident.to_string().trim_start_matches("r#").to_owned()
}

pub(super) fn is_owner(ty: &syn::Type, names: &BTreeSet<String>) -> bool {
    match ty {
        syn::Type::Path(path) => path
            .path
            .segments
            .iter()
            .any(|s| names.contains(&name(&s.ident))),
        syn::Type::Paren(ty) => is_owner(&ty.elem, names),
        syn::Type::Group(ty) => is_owner(&ty.elem, names),
        _ => false,
    }
}

pub(super) fn owners(graphs: &[ModuleGraph]) -> BTreeSet<String> {
    let mut names = BTreeSet::from(["ExecutionRequest".to_owned()]);
    loop {
        let mut aliases = Aliases {
            names: &names,
            found: BTreeSet::new(),
        };
        for module in production_modules(graphs) {
            for item in module.items {
                aliases.visit_item(item);
            }
        }
        let found = aliases.found;
        let before = names.len();
        names.extend(found);
        if names.len() == before {
            return names;
        }
    }
}

struct Aliases<'a> {
    names: &'a BTreeSet<String>,
    found: BTreeSet<String>,
}
impl Aliases<'_> {
    fn import(&mut self, tree: &syn::UseTree) {
        match tree {
            syn::UseTree::Path(path) => self.import(&path.tree),
            syn::UseTree::Group(group) => {
                for item in &group.items {
                    self.import(item);
                }
            }
            syn::UseTree::Rename(rename) if self.names.contains(&name(&rename.ident)) => {
                self.found.insert(name(&rename.rename));
            }
            syn::UseTree::Name(_) | syn::UseTree::Rename(_) | syn::UseTree::Glob(_) => {}
        }
    }
}
impl<'ast> Visit<'ast> for Aliases<'_> {
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.import(&item.tree);
    }
    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        if is_owner(&item.ty, self.names) {
            self.found.insert(name(&item.ident));
        }
        visit::visit_item_type(self, item);
    }
}
