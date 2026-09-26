//! Producer inventory for the explicitly protected C7 phase and context set.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use quote::ToTokens;
use syn::{
    AngleBracketedGenericArguments, Fields, GenericArgument, ImplItem, Item, PathArguments,
    ReturnType, TraitItem, Type, TypeParamBound, Visibility,
};

#[path = "producer_inventory/protected_types.rs"]
mod protected_types;
use protected_types::PROTECTED;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Producer {
    source: String,
    owner: String,
    name: String,
    authority: String,
    shape: &'static str,
}

#[path = "producer_inventory/court.rs"]
mod court;

fn source_roots() -> [(PathBuf, &'static str); 3] {
    let root = super::root();
    [
        (
            root.join("workspaces/worth-query/crates/worth-query-execution/src"),
            "query-execution",
        ),
        (
            root.join("workspaces/worth-query/crates/worth-query/src"),
            "query-api",
        ),
        (
            root.join("workspaces/worth-query-bank-world/crates/bank-server/src"),
            "bank-server",
        ),
    ]
}

fn rust_files(root: &Path) -> Vec<PathBuf> {
    fn visit(directory: &Path, files: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(directory)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
        {
            let path = entry.expect("directory entry").path();
            if path.is_dir() {
                visit(&path, files);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }
    let mut files = Vec::new();
    visit(root, &mut files);
    files.sort();
    files
}

fn broad(visibility: &Visibility) -> bool {
    !matches!(visibility, Visibility::Inherited)
}

fn terminal(ty: &Type) -> Option<String> {
    let Type::Path(path) = ty else { return None };
    path.path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}

fn authority_types(
    ty: &Type,
    owner: Option<&str>,
    aliases: &BTreeMap<String, Type>,
    associated: &BTreeMap<String, Type>,
    output: &mut BTreeSet<String>,
) {
    match ty {
        Type::Array(array) => authority_types(&array.elem, owner, aliases, associated, output),
        Type::BareFn(function) => {
            if let ReturnType::Type(_, returned) = &function.output {
                authority_types(returned, owner, aliases, associated, output);
            }
        }
        Type::Group(group) => authority_types(&group.elem, owner, aliases, associated, output),
        Type::ImplTrait(implementation) => {
            for bound in &implementation.bounds {
                if let TypeParamBound::Trait(bound) = bound {
                    collect_path_arguments(&bound.path, owner, aliases, associated, output);
                }
            }
        }
        Type::TraitObject(object) => {
            for bound in &object.bounds {
                if let TypeParamBound::Trait(bound) = bound {
                    collect_path_arguments(&bound.path, owner, aliases, associated, output);
                }
            }
        }
        Type::Paren(paren) => authority_types(&paren.elem, owner, aliases, associated, output),
        Type::Path(path) => {
            let segments = &path.path.segments;
            let last = segments.last().expect("type path has a segment");
            let key = last.ident.to_string();
            if PROTECTED.contains(&key.as_str()) {
                output.insert(key.clone());
            } else if key == "Self" {
                if let Some(owner) = owner.filter(|owner| PROTECTED.contains(owner)) {
                    output.insert(owner.to_owned());
                }
            } else if segments
                .first()
                .is_some_and(|segment| segment.ident == "Self")
            {
                if let Some(target) = associated.get(&key) {
                    authority_types(target, owner, aliases, associated, output);
                }
            } else if let Some(target) = aliases.get(&key).filter(|_| segments.len() == 1) {
                // Only a bare name refers to this file's own alias; a qualified
                // path such as `retention::View` names another module's item,
                // even when the alias re-exports it under the same name.
                authority_types(target, owner, aliases, associated, output);
            }
            collect_path_arguments(&path.path, owner, aliases, associated, output);
        }
        Type::Ptr(pointer) => authority_types(&pointer.elem, owner, aliases, associated, output),
        // A borrow observes an existing authority; it does not mint an owned
        // phase/context value. Raw-borrow doors are fenced separately by the
        // field/getter topology tests in the parent module.
        Type::Reference(_) => {}
        Type::Slice(slice) => authority_types(&slice.elem, owner, aliases, associated, output),
        Type::Tuple(tuple) => {
            for element in &tuple.elems {
                authority_types(element, owner, aliases, associated, output);
            }
        }
        _ => {}
    }
}

fn collect_path_arguments(
    path: &syn::Path,
    owner: Option<&str>,
    aliases: &BTreeMap<String, Type>,
    associated: &BTreeMap<String, Type>,
    output: &mut BTreeSet<String>,
) {
    for segment in &path.segments {
        let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
            &segment.arguments
        else {
            continue;
        };
        for argument in args {
            match argument {
                GenericArgument::Type(ty) => {
                    authority_types(ty, owner, aliases, associated, output)
                }
                GenericArgument::AssocType(binding) => {
                    authority_types(&binding.ty, owner, aliases, associated, output)
                }
                GenericArgument::Constraint(constraint) => {
                    for bound in &constraint.bounds {
                        if let TypeParamBound::Trait(bound) = bound {
                            collect_path_arguments(&bound.path, owner, aliases, associated, output);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn returned_authorities(
    output: &ReturnType,
    owner: Option<&str>,
    aliases: &BTreeMap<String, Type>,
    associated: &BTreeMap<String, Type>,
) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    if let ReturnType::Type(_, ty) = output {
        authority_types(ty, owner, aliases, associated, &mut found);
    }
    found
}

fn push_returns(
    producers: &mut BTreeSet<Producer>,
    source: &str,
    owner: &str,
    name: &str,
    output: &ReturnType,
    aliases: &BTreeMap<String, Type>,
    associated: &BTreeMap<String, Type>,
    shape: &'static str,
) {
    for authority in returned_authorities(output, Some(owner), aliases, associated) {
        producers.insert(Producer {
            source: source.to_owned(),
            owner: owner.to_owned(),
            name: name.to_owned(),
            authority,
            shape,
        });
    }
}

fn inventory_file(source_name: &str, source: &str) -> BTreeSet<Producer> {
    let syntax = syn::parse_file(source).expect("producer specimen must parse");
    let aliases: BTreeMap<_, _> = syntax
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Type(alias) if broad(&alias.vis) => {
                Some((alias.ident.to_string(), (*alias.ty).clone()))
            }
            _ => None,
        })
        .collect();
    let mut producers = BTreeSet::new();
    for item in &syntax.items {
        match item {
            Item::Const(value) if broad(&value.vis) => push_returns(
                &mut producers,
                source_name,
                "<module>",
                &value.ident.to_string(),
                &ReturnType::Type(Default::default(), value.ty.clone()),
                &aliases,
                &BTreeMap::new(),
                "const",
            ),
            Item::Fn(function) if broad(&function.vis) => push_returns(
                &mut producers,
                source_name,
                "<module>",
                &function.sig.ident.to_string(),
                &function.sig.output,
                &aliases,
                &BTreeMap::new(),
                "free-fn",
            ),
            Item::Impl(item_impl) => {
                let owner = terminal(item_impl.self_ty.as_ref()).unwrap_or_else(|| "<?>".into());
                let associated: BTreeMap<_, _> = item_impl
                    .items
                    .iter()
                    .filter_map(|item| match item {
                        ImplItem::Type(binding) => {
                            Some((binding.ident.to_string(), binding.ty.clone()))
                        }
                        _ => None,
                    })
                    .collect();
                if item_impl.trait_.as_ref().is_some_and(|(_, path, _)| {
                    path.segments
                        .last()
                        .is_some_and(|segment| segment.ident == "Default")
                }) && PROTECTED.contains(&owner.as_str())
                {
                    producers.insert(Producer {
                        source: source_name.to_owned(),
                        owner: owner.clone(),
                        name: "default".to_owned(),
                        authority: owner.clone(),
                        shape: "default",
                    });
                }
                for member in &item_impl.items {
                    if let ImplItem::Fn(function) = member {
                        if broad(&function.vis) || item_impl.trait_.is_some() {
                            push_returns(
                                &mut producers,
                                source_name,
                                &owner,
                                &function.sig.ident.to_string(),
                                &function.sig.output,
                                &aliases,
                                &associated,
                                "method",
                            );
                        }
                    }
                }
            }
            Item::Static(value) if broad(&value.vis) => push_returns(
                &mut producers,
                source_name,
                "<module>",
                &value.ident.to_string(),
                &ReturnType::Type(Default::default(), value.ty.clone()),
                &aliases,
                &BTreeMap::new(),
                "static",
            ),
            Item::Struct(structure)
                if PROTECTED.contains(&structure.ident.to_string().as_str()) =>
            {
                let derives_default = structure.attrs.iter().any(|attribute| {
                    attribute.path().is_ident("derive")
                        && attribute
                            .meta
                            .to_token_stream()
                            .to_string()
                            .contains("Default")
                });
                if derives_default {
                    producers.insert(Producer {
                        source: source_name.to_owned(),
                        owner: structure.ident.to_string(),
                        name: "derive(Default)".to_owned(),
                        authority: structure.ident.to_string(),
                        shape: "default",
                    });
                }
                if broad(&structure.vis) && matches!(&structure.fields, Fields::Unit) {
                    producers.insert(Producer {
                        source: source_name.to_owned(),
                        owner: structure.ident.to_string(),
                        name: "unit-constructor".to_owned(),
                        authority: structure.ident.to_string(),
                        shape: "constructor",
                    });
                }
            }
            Item::Trait(item_trait) if broad(&item_trait.vis) => {
                for member in &item_trait.items {
                    if let TraitItem::Fn(function) = member {
                        push_returns(
                            &mut producers,
                            source_name,
                            &item_trait.ident.to_string(),
                            &function.sig.ident.to_string(),
                            &function.sig.output,
                            &aliases,
                            &BTreeMap::new(),
                            "trait-fn",
                        );
                    }
                }
            }
            Item::Type(alias) if broad(&alias.vis) => {
                let mut found = BTreeSet::new();
                authority_types(&alias.ty, None, &aliases, &BTreeMap::new(), &mut found);
                for authority in found {
                    producers.insert(Producer {
                        source: source_name.to_owned(),
                        owner: "<module>".to_owned(),
                        name: alias.ident.to_string(),
                        authority,
                        shape: "alias",
                    });
                }
            }
            _ => {}
        }
    }
    producers
}

fn protected_declarations(source: &str) -> BTreeSet<String> {
    syn::parse_file(source)
        .expect("producer specimen must parse")
        .items
        .into_iter()
        .filter_map(|item| match item {
            Item::Struct(item) if PROTECTED.contains(&item.ident.to_string().as_str()) => {
                Some(item.ident.to_string())
            }
            Item::Enum(item) if PROTECTED.contains(&item.ident.to_string().as_str()) => {
                Some(item.ident.to_string())
            }
            _ => None,
        })
        .collect()
}
