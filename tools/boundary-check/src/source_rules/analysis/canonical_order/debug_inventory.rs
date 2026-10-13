//! Debug may describe a TypeId-keyed inventory's size, but never walk its keys.
use std::collections::{BTreeMap, BTreeSet};
use syn::visit::{self, Visit};
use syn::{GenericArgument, Item, PathArguments, Type};

struct Alias {
    parameters: Vec<(usize, String)>,
    ty: Type,
}
type AliasKey = (Vec<String>, String);

#[derive(Default)]
pub(super) struct Inventories {
    collections: BTreeSet<String>,
    renames: Vec<(String, String)>,
    aliases: BTreeMap<AliasKey, Alias>,
    namespace: Vec<String>,
}

impl Inventories {
    pub fn from_files(files: &[(&[String], &str, &[Item])]) -> Self {
        let mut inventory = Self::default();
        inventory
            .collections
            .extend(["HashSet".into(), "HashMap".into()]);
        for (namespace, _, items) in files {
            inventory.namespace = namespace.to_vec();
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

    fn aliases(&self, path: &syn::Path, namespace: &[String]) -> Vec<(&AliasKey, &Alias)> {
        let mut names: Vec<_> = path.segments.iter().map(|s| s.ident.to_string()).collect();
        let Some(name) = names.pop() else {
            return Vec::new();
        };
        let unqualified = names.is_empty();
        let mut base = namespace.to_vec();
        let anchored = names
            .first()
            .is_some_and(|n| matches!(n.as_str(), "crate" | "self" | "super"));
        if names.first().is_some_and(|n| n == "crate") {
            base.clear();
            names.remove(0);
        } else if names.first().is_some_and(|n| n == "self") {
            names.remove(0);
        }
        while names.first().is_some_and(|n| n == "super") {
            base.pop();
            names.remove(0);
        }
        loop {
            let mut candidate = base.clone();
            candidate.extend(names.iter().cloned());
            if let Some(alias) = self.aliases.get_key_value(&(candidate, name.clone())) {
                return vec![alias];
            }
            if anchored || base.is_empty() {
                break;
            }
            base.pop();
        }
        // A `use` can bring a sibling's alias here. Preserve scoped shadowing,
        // then conservatively examine every declaration with this bare name.
        if unqualified {
            self.aliases
                .iter()
                .filter(|(key, _)| key.1 == name)
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn derives_members<'a>(
        &self,
        attributes: &[syn::Attribute],
        namespace: &[String],
        generics: &syn::Generics,
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
            parameters: generics
                .type_params()
                .map(|p| p.ident.to_string())
                .collect(),
            namespace: namespace.to_vec(),
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
    fn visit_item_mod(&mut self, _: &'ast syn::ItemMod) {}
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
            (self.namespace.clone(), item.ident.to_string()),
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
    expanding: BTreeSet<AliasKey>,
    namespace: Vec<String>,
    found: bool,
}

impl<'ast> Visit<'ast> for Members<'_> {
    fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
        if self.found {
            return;
        }
        let mut prefix = syn::Path {
            leading_colon: path.path.leading_colon,
            segments: Default::default(),
        };
        for segment in &path.path.segments {
            prefix.segments.push(segment.clone());
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
            for (key, alias) in self.inventory.aliases(&prefix, &self.namespace) {
                if self.expanding.insert(key.clone()) {
                    let saved_namespace = std::mem::replace(&mut self.namespace, key.0.clone());
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
                    self.namespace = saved_namespace;
                    self.expanding.remove(key);
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
