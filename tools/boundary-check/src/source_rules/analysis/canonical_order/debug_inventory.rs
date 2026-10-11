//! Debug may describe a TypeId-keyed inventory's size, but never walk its keys.
use std::collections::BTreeSet;
use syn::visit::{self, Visit};

pub(super) fn derives_members(item: &syn::ItemStruct, type_ids: &BTreeSet<String>) -> bool {
    let derives_debug = item
        .attrs
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
        });
    if !derives_debug {
        return false;
    }
    struct Inventory<'a> {
        type_ids: &'a BTreeSet<String>,
        found: bool,
    }
    impl<'ast> Visit<'ast> for Inventory<'_> {
        fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
            for segment in &path.path.segments {
                if segment.ident == "HashSet" || segment.ident == "HashMap" {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(key)) = args.args.first() {
                            self.found |= key_contains_identity(key, self.type_ids);
                        }
                    }
                }
            }
            visit::visit_type_path(self, path);
        }
    }
    let mut inventory = Inventory {
        type_ids,
        found: false,
    };
    for field in &item.fields {
        inventory.visit_type(&field.ty);
    }
    inventory.found
}

fn key_contains_identity(ty: &syn::Type, names: &BTreeSet<String>) -> bool {
    match ty {
        syn::Type::Path(path) => path
            .path
            .segments
            .last()
            .is_some_and(|s| names.contains(&s.ident.to_string())),
        syn::Type::Tuple(tuple) => tuple
            .elems
            .iter()
            .any(|ty| key_contains_identity(ty, names)),
        syn::Type::Paren(paren) => key_contains_identity(&paren.elem, names),
        syn::Type::Group(group) => key_contains_identity(&group.elem, names),
        _ => false,
    }
}
