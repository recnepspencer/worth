//! Debug may describe a TypeId-keyed inventory's size, but never walk its keys.
use std::collections::{BTreeMap, BTreeSet};
use syn::visit::{self, Visit};
use syn::{GenericArgument, Item, PathArguments, Type};

struct Alias {
    parameters: Vec<(usize, String)>,
    ty: Type,
}

#[derive(Default)]
pub(super) struct Inventories {
    collections: BTreeSet<String>,
    renames: Vec<(String, String)>,
    aliases: BTreeMap<String, Alias>,
}

impl Inventories {
    pub fn from_files(files: &[(&str, &[Item])]) -> Self {
        let mut inventory = Self::default();
        inventory
            .collections
            .extend(["HashSet".into(), "HashMap".into()]);
        for (_, items) in files {
            for item in *items {
                inventory.visit_item(item);
            }
        }
        loop {
            let before = inventory.collections.len();
            for (from, to) in &inventory.renames {
                if inventory.collections.contains(from) {
                    inventory.collections.insert(to.clone());
                }
            }
            if before == inventory.collections.len() {
                break;
            }
        }
        inventory
    }

    pub fn derives_members<'a>(
        &self,
        attributes: &[syn::Attribute],
        fields: impl IntoIterator<Item = &'a Type>,
        type_ids: &BTreeSet<String>,
    ) -> bool {
        if !attributes
            .iter()
            .filter(|a| a.path().is_ident("derive"))
            .any(|a| {
                a.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                )
                .is_ok_and(|paths| {
                    paths
                        .iter()
                        .any(|p| p.segments.last().is_some_and(|s| s.ident == "Debug"))
                })
            })
        {
            return false;
        }
        let mut visitor = Members {
            inventory: self,
            type_ids,
            parameters: BTreeSet::new(),
            expanding: BTreeSet::new(),
            found: false,
        };
        for field in fields {
            visitor.visit_type(field);
        }
        visitor.found
    }
}

impl<'ast> Visit<'ast> for Inventories {
    fn visit_use_tree(&mut self, tree: &'ast syn::UseTree) {
        if let syn::UseTree::Rename(rename) = tree {
            self.renames
                .push((rename.ident.to_string(), rename.rename.to_string()));
        }
        visit::visit_use_tree(self, tree);
    }
    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        let parameters = item
            .generics
            .params
            .iter()
            .enumerate()
            .filter_map(|(index, param)| {
                if let syn::GenericParam::Type(param) = param {
                    Some((index, param.ident.to_string()))
                } else {
                    None
                }
            })
            .collect();
        self.aliases.insert(
            item.ident.to_string(),
            Alias {
                parameters,
                ty: (*item.ty).clone(),
            },
        );
    }
}

struct Members<'a> {
    inventory: &'a Inventories,
    type_ids: &'a BTreeSet<String>,
    parameters: BTreeSet<String>,
    expanding: BTreeSet<String>,
    found: bool,
}

impl<'ast> Visit<'ast> for Members<'_> {
    fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
        if self.found {
            return;
        }
        for segment in &path.path.segments {
            let name = segment.ident.to_string();
            let arguments = match &segment.arguments {
                PathArguments::AngleBracketed(args) => Some(&args.args),
                _ => None,
            };
            if self.inventory.collections.contains(&name) {
                if let Some(GenericArgument::Type(key)) = arguments.and_then(|args| args.first()) {
                    self.found |= contains_identity(key, self.type_ids, &self.parameters);
                }
            }
            if let Some(alias) = self.inventory.aliases.get(&name) {
                if self.expanding.insert(name.clone()) {
                    let saved = self.parameters.clone();
                    for (index, name) in &alias.parameters {
                        let identity = arguments
                            .and_then(|args| args.iter().nth(*index))
                            .is_some_and(|arg| {
                                matches!(arg, GenericArgument::Type(ty)
                                if contains_identity(ty, self.type_ids, &saved))
                            });
                        self.parameters.remove(name);
                        if identity {
                            self.parameters.insert(name.clone());
                        }
                    }
                    self.visit_type(&alias.ty);
                    self.parameters = saved;
                    self.expanding.remove(&name);
                }
            }
        }
        visit::visit_type_path(self, path);
    }
}

fn contains_identity(ty: &Type, names: &BTreeSet<String>, parameters: &BTreeSet<String>) -> bool {
    struct Identity<'a> {
        names: &'a BTreeSet<String>,
        parameters: &'a BTreeSet<String>,
        found: bool,
    }
    impl<'ast> Visit<'ast> for Identity<'_> {
        fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
            self.found |= path.path.segments.last().is_some_and(|segment| {
                let name = segment.ident.to_string();
                self.names.contains(&name) || self.parameters.contains(&name)
            });
            visit::visit_type_path(self, path);
        }
    }
    let mut identity = Identity {
        names,
        parameters,
        found: false,
    };
    identity.visit_type(ty);
    identity.found
}
