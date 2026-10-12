//! Lexical declarations and import routes for aliases in a production tree.
use super::super::crate_modules::ModuleGraph;
use super::super::production_cfg::module_compiled_out_of_production;
use super::super::production_scope::{visit_item_in_scope, ScopedVisitor};
use proc_macro2::{TokenStream, TokenTree};
use std::collections::{BTreeMap, BTreeSet};
use syn::visit::{self, Visit};

pub(super) type Name = Vec<String>;

pub(super) struct Definition {
    pub namespace: Name,
    pub parameters: BTreeMap<String, usize>,
    pub ty: syn::Type,
}

#[derive(Default)]
pub(super) struct Declarations {
    pub definitions: BTreeMap<Name, Vec<Definition>>,
    sources: BTreeMap<String, Name>,
    modules: BTreeSet<Name>,
    imports: BTreeMap<Name, (Name, Name)>,
    nominal_types: BTreeSet<Name>,
}

impl Declarations {
    pub fn from_graphs(graphs: &[ModuleGraph]) -> Self {
        let mut declarations = Self::default();
        for graph in graphs {
            for (namespace, node) in &graph.modules {
                if module_compiled_out_of_production(graph, namespace) {
                    continue;
                }
                declarations
                    .sources
                    .entry(node.relative_source.clone())
                    .or_insert_with(|| namespace.clone());
                declarations.modules.insert(namespace.clone());
                let mut collector = Collector {
                    declarations: &mut declarations,
                    scope: namespace.clone(),
                };
                for item in &node.items {
                    collector.visit_item(item);
                }
            }
        }
        declarations
    }

    pub fn namespace(&self, source: &str) -> Name {
        self.sources.get(source).cloned().unwrap_or_default()
    }

    pub fn lookup(&self, path: &syn::Path, scope: &[String]) -> Option<Name> {
        let names: Name = path.segments.iter().map(|s| s.ident.to_string()).collect();
        self.lookup_names(&names, scope, &mut BTreeSet::new())
    }

    fn lookup_names(
        &self,
        names: &[String],
        scope: &[String],
        seen: &mut BTreeSet<Name>,
    ) -> Option<Name> {
        let (prefix, rest, absolute) = match names.first()?.as_str() {
            "crate" => (Vec::new(), &names[1..], true),
            "self" | "super" => {
                let mut prefix = scope.to_vec();
                while !self.modules.contains(&prefix) {
                    prefix.pop()?;
                }
                let mut rest = names;
                if rest[0] == "self" {
                    rest = &rest[1..];
                }
                while rest.first().is_some_and(|s| s == "super") {
                    prefix.pop()?;
                    rest = &rest[1..];
                }
                (prefix, rest, true)
            }
            _ => (scope.to_vec(), names, false),
        };
        let mut prefix = prefix;
        loop {
            let key: Name = prefix.iter().chain(rest).cloned().collect();
            if self.definitions.contains_key(&key) {
                return Some(key);
            }
            for length in 1..=rest.len() {
                let binding: Name = prefix.iter().chain(&rest[..length]).cloned().collect();
                if self.nominal_types.contains(&binding) {
                    return None;
                }
                if let Some((namespace, target)) = self.imports.get(&binding) {
                    if seen.insert(binding) {
                        let target: Name = target.iter().chain(&rest[length..]).cloned().collect();
                        if let Some(key) = self.lookup_names(&target, namespace, seen) {
                            return Some(key);
                        }
                    }
                }
            }
            if absolute || prefix.pop().is_none() {
                return None;
            }
        }
    }
}

struct Collector<'a> {
    declarations: &'a mut Declarations,
    scope: Name,
}

impl ScopedVisitor for Collector<'_> {
    fn scope(&mut self) -> &mut Vec<String> {
        &mut self.scope
    }
}

impl Collector<'_> {
    fn import(&mut self, tree: &syn::UseTree, path: &mut Name) {
        match tree {
            syn::UseTree::Path(item) => {
                path.push(item.ident.to_string());
                self.import(&item.tree, path);
                path.pop();
            }
            syn::UseTree::Group(group) => {
                for item in &group.items {
                    self.import(item, path);
                }
            }
            syn::UseTree::Name(item) => self.bind_import(path, &item.ident, &item.ident),
            syn::UseTree::Rename(item) => self.bind_import(path, &item.ident, &item.rename),
            syn::UseTree::Glob(_) => {}
        }
    }
    fn bind_import(&mut self, path: &[String], name: &syn::Ident, local: &syn::Ident) {
        let mut target = path.to_vec();
        target.push(name.to_string());
        let mut key = self.scope.clone();
        key.push(local.to_string());
        self.declarations
            .imports
            .insert(key, (self.scope.clone(), target));
    }
    fn macro_items(&mut self, tokens: TokenStream) {
        if let Ok(file) = syn::parse2::<syn::File>(tokens.clone()) {
            for item in &file.items {
                self.visit_item(item);
            }
            return;
        }
        for token in tokens {
            if let TokenTree::Group(group) = token {
                self.macro_items(group.stream());
            }
        }
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        let ident = match item {
            syn::Item::Struct(item) => Some(&item.ident),
            syn::Item::Enum(item) => Some(&item.ident),
            syn::Item::Union(item) => Some(&item.ident),
            syn::Item::Trait(item) => Some(&item.ident),
            _ => None,
        };
        if let Some(ident) = ident {
            let mut name = self.scope.clone();
            name.push(ident.to_string());
            self.declarations.nominal_types.insert(name);
        }
        visit_item_in_scope(self, item);
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        self.declarations.modules.insert(self.scope.clone());
        visit::visit_item_mod(self, item);
    }
    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        let parameters = item
            .generics
            .params
            .iter()
            .enumerate()
            .filter_map(|(index, param)| {
                if let syn::GenericParam::Type(param) = param {
                    Some((param.ident.to_string(), index))
                } else {
                    None
                }
            })
            .collect();
        self.declarations
            .definitions
            .entry(self.scope.clone())
            .or_default()
            .push(Definition {
                namespace: self.scope[..self.scope.len() - 1].to_vec(),
                parameters,
                ty: (*item.ty).clone(),
            });
        visit::visit_item_type(self, item);
    }
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.import(&item.tree, &mut Vec::new());
    }
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        self.macro_items(mac.tokens.clone());
    }
}
