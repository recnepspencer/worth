//! The sealed types an owner defines, and the constructors it gives them.

use super::super::super::production_cfg::compiled_out_of_production;
use super::super::super::production_scope::{
    self_type_name, visit_item_in_scope, ProductionModule, ScopedVisitor,
};
use super::owned_types;
use std::collections::{BTreeMap, BTreeSet};
use syn::visit::Visit;
use syn::{ImplItem, Item, Type};

/// Read every owner module for its sealed types and their constructors.
pub(super) fn declared(
    owners: &BTreeMap<&str, &[String]>,
    modules: &[ProductionModule<'_>],
) -> Declared {
    let mut declared = Declared::default();
    for module in modules {
        let owned = owned_types(owners, module);
        if !owned.is_empty() {
            let mut reader = OwnerReader {
                owned: &owned,
                scope: Vec::new(),
                declared: &mut declared,
            };
            module.items.iter().for_each(|item| reader.visit_item(item));
        }
    }
    declared
}

#[derive(Default)]
pub(super) struct Declared {
    pub(super) defined: BTreeSet<String>,
    pub(super) constructors: BTreeMap<String, BTreeSet<String>>,
}

/// Reads an owner for the sealed types it defines and their constructors.
struct OwnerReader<'a> {
    owned: &'a BTreeSet<&'a str>,
    scope: Vec<String>,
    declared: &'a mut Declared,
}

impl ScopedVisitor for OwnerReader<'_> {
    fn scope(&mut self) -> &mut Vec<String> {
        &mut self.scope
    }
}

impl<'ast> Visit<'ast> for OwnerReader<'_> {
    fn visit_item(&mut self, item: &'ast Item) {
        let owned = |name: &str| self.owned.contains(name);
        match item {
            Item::Struct(item) if owned(&item.ident.to_string()) => {
                self.declared.defined.insert(item.ident.to_string());
            }
            Item::Enum(item) if owned(&item.ident.to_string()) => {
                let name = item.ident.to_string();
                let variants = item
                    .variants
                    .iter()
                    .map(|variant| variant.ident.to_string());
                self.declared
                    .constructors
                    .entry(name.clone())
                    .or_default()
                    .extend(variants);
                self.declared.defined.insert(name);
            }
            Item::Impl(item) if owned(&self_type_name(&item.self_ty)) => {
                let name = self_type_name(&item.self_ty);
                let constructors = item
                    .items
                    .iter()
                    .filter_map(|member| constructor_name(member, &name));
                let constructors: Vec<_> = constructors.collect();
                self.declared
                    .constructors
                    .entry(name)
                    .or_default()
                    .extend(constructors);
            }
            _ => {}
        }
        visit_item_in_scope(self, item);
    }
}

/// An associated function without a receiver, or an associated constant,
/// that yields `owner`.
fn constructor_name(member: &ImplItem, owner: &str) -> Option<String> {
    match member {
        ImplItem::Fn(function)
            if !compiled_out_of_production(&function.attrs)
                && function.sig.receiver().is_none() =>
        {
            let syn::ReturnType::Type(_, output) = &function.sig.output else {
                return None;
            };
            mentions(output, owner).then(|| function.sig.ident.to_string())
        }
        ImplItem::Const(constant)
            if !compiled_out_of_production(&constant.attrs) && mentions(&constant.ty, owner) =>
        {
            Some(constant.ident.to_string())
        }
        _ => None,
    }
}

fn mentions(output: &Type, owner: &str) -> bool {
    struct Mentions<'a>(&'a str, bool);
    impl<'ast> Visit<'ast> for Mentions<'_> {
        fn visit_ident(&mut self, ident: &'ast proc_macro2::Ident) {
            self.1 |= ident == "Self" || ident == self.0;
        }
    }
    let mut mentions = Mentions(owner, false);
    mentions.visit_type(output);
    mentions.1
}
