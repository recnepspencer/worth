//! Identity aliases reduce to a parameter; containers retain their data meaning.
use super::super::crate_modules::ModuleGraph;
use super::alias_declarations::{Declarations, Definition, Name};
use std::collections::{BTreeMap, BTreeSet};
use syn::visit::{self, Visit};
use syn::{GenericArgument, Path, PathArguments, Type};

pub(super) const RAW: &str = "ExecutionAllocationDenialKind";

pub(super) struct Aliases {
    declarations: Declarations,
    transparent: BTreeMap<Name, BTreeSet<usize>>,
    raw: BTreeSet<Name>,
}

impl Aliases {
    pub fn from_graphs(graphs: &[ModuleGraph]) -> Self {
        let mut aliases = Self {
            declarations: Declarations::from_graphs(graphs),
            transparent: BTreeMap::new(),
            raw: BTreeSet::new(),
        };
        loop {
            let mut transparent = aliases.transparent.clone();
            let mut raw = aliases.raw.clone();
            for (name, definitions) in &aliases.declarations.definitions {
                for definition in definitions {
                    let parameters = aliases.parameters(&definition.ty, definition);
                    if !parameters.is_empty() {
                        transparent
                            .entry(name.clone())
                            .or_default()
                            .extend(parameters);
                    }
                    if aliases.raw_type(&definition.ty, &definition.namespace) {
                        raw.insert(name.clone());
                    }
                }
            }
            if transparent == aliases.transparent && raw == aliases.raw {
                break;
            }
            aliases.transparent = transparent;
            aliases.raw = raw;
        }
        aliases
    }

    pub fn namespace(&self, source: &str) -> Name {
        self.declarations.namespace(source)
    }

    fn parameters(&self, ty: &Type, definition: &Definition) -> BTreeSet<usize> {
        match ty {
            Type::Paren(paren) => self.parameters(&paren.elem, definition),
            Type::Group(group) => self.parameters(&group.elem, definition),
            Type::Path(path) if path.qself.is_none() => {
                let Some(last) = path.path.segments.last() else {
                    return BTreeSet::new();
                };
                if path.path.segments.len() == 1 {
                    if let Some(index) = definition.parameters.get(&last.ident.to_string()) {
                        return BTreeSet::from([*index]);
                    }
                }
                self.declarations
                    .lookup(&path.path, &definition.namespace)
                    .and_then(|key| self.transparent.get(&key))
                    .into_iter()
                    .flatten()
                    .filter_map(|index| argument(&last.arguments, *index))
                    .flat_map(|ty| self.parameters(ty, definition))
                    .collect()
            }
            _ => BTreeSet::new(),
        }
    }

    pub fn raw_type(&self, ty: &Type, scope: &[String]) -> bool {
        match ty {
            Type::Path(path) if path.qself.is_none() => self.raw_path(&path.path, scope),
            Type::Reference(reference) => self.raw_type(&reference.elem, scope),
            Type::Paren(paren) => self.raw_type(&paren.elem, scope),
            Type::Group(group) => self.raw_type(&group.elem, scope),
            _ => false,
        }
    }

    pub fn raw_path(&self, path: &Path, scope: &[String]) -> bool {
        let Some(last) = path.segments.last() else {
            return false;
        };
        if last.ident == RAW {
            return true;
        }
        if self.raw.is_empty() && self.transparent.is_empty() {
            return false;
        }
        let Some(key) = self.declarations.lookup(path, scope) else {
            return false;
        };
        self.raw.contains(&key)
            || self.transparent.get(&key).is_some_and(|indices| {
                indices
                    .iter()
                    .filter_map(|index| argument(&last.arguments, *index))
                    .any(|ty| self.raw_type(ty, scope))
            })
    }

    pub fn mentions_raw(&self, ty: &Type, scope: &[String]) -> bool {
        struct Mentions<'a> {
            aliases: &'a Aliases,
            scope: Name,
            seen: BTreeSet<Name>,
            found: bool,
        }
        impl<'ast> Visit<'ast> for Mentions<'_> {
            fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
                self.found |= self.aliases.raw_path(&path.path, &self.scope);
                if self.found {
                    return;
                }
                if let Some(key) = self.aliases.declarations.lookup(&path.path, &self.scope) {
                    if self.seen.insert(key.clone()) {
                        let aliases = self.aliases;
                        if let Some(definitions) = aliases.declarations.definitions.get(&key) {
                            let saved = self.scope.clone();
                            for definition in definitions {
                                self.scope = definition.namespace.clone();
                                self.visit_type(&definition.ty);
                            }
                            self.scope = saved;
                        }
                    }
                }
                visit::visit_type_path(self, path);
            }
        }
        let mut visitor = Mentions {
            aliases: self,
            scope: scope.to_vec(),
            seen: BTreeSet::new(),
            found: false,
        };
        visitor.visit_type(ty);
        visitor.found
    }
}

fn argument(arguments: &PathArguments, index: usize) -> Option<&Type> {
    let PathArguments::AngleBracketed(arguments) = arguments else {
        return None;
    };
    match arguments.args.iter().nth(index)? {
        GenericArgument::Type(ty) => Some(ty),
        _ => None,
    }
}
