//! Canonical local type identity for associated-projection matching.

use std::collections::BTreeSet;

use quote::ToTokens;
use syn::{GenericArgument, Item, PathArguments, Type};

use super::authority_value_gate_projection_matching::{substitute_type, TypeBindings};
use super::authority_value_identity::local_type_key;
use super::crate_modules::ModuleGraph;

pub(super) fn canonical_type(
    graph: &ModuleGraph,
    module_path: &[String],
    ty: &Type,
    bindings: &TypeBindings,
) -> Type {
    canonical_type_inner(graph, module_path, ty, bindings, &mut BTreeSet::new())
}

fn canonical_type_inner(
    graph: &ModuleGraph,
    module_path: &[String],
    ty: &Type,
    bindings: &TypeBindings,
    alias_stack: &mut BTreeSet<String>,
) -> Type {
    let substituted = substitute_type(ty, bindings);
    let Type::Path(type_path) = &substituted else {
        return canonical_children(graph, module_path, substituted, bindings, alias_stack);
    };
    if type_path.qself.is_some() {
        return canonical_children(graph, module_path, substituted, bindings, alias_stack);
    }
    let Some(key) = local_type_key(graph, module_path, &substituted) else {
        return canonical_children(graph, module_path, substituted, bindings, alias_stack);
    };
    let Some(node) = graph.modules.get(&key.0) else {
        return substituted;
    };
    let Some(item) = node.items.iter().find(|item| match item {
        Item::Type(alias) => alias.ident == key.1,
        Item::Struct(item) => item.ident == key.1,
        Item::Enum(item) => item.ident == key.1,
        Item::Union(item) => item.ident == key.1,
        _ => false,
    }) else {
        return substituted;
    };
    if let Item::Type(alias) = item {
        return expand_alias(
            graph,
            &key,
            alias,
            type_path,
            bindings,
            alias_stack,
            substituted.clone(),
        );
    }
    canonical_nominal_type(graph, module_path, &key, type_path, bindings, alias_stack)
}

fn expand_alias(
    graph: &ModuleGraph,
    key: &super::authority_value_gate_defs::DefinitionKey,
    alias: &syn::ItemType,
    type_path: &syn::TypePath,
    bindings: &TypeBindings,
    alias_stack: &mut BTreeSet<String>,
    substituted: Type,
) -> Type {
    let arguments = type_path
        .path
        .segments
        .last()
        .map(|segment| &segment.arguments)
        .unwrap_or(&PathArguments::None);
    let mut alias_bindings = bindings.clone();
    bind_alias_arguments(alias, arguments, &mut alias_bindings);
    let alias_identity = format!(
        "{}::{}<{}>",
        key.0.join("::"),
        key.1,
        arguments.to_token_stream()
    );
    if !alias_stack.insert(alias_identity.clone()) {
        return substituted;
    }
    let canonical = canonical_type_inner(graph, &key.0, &alias.ty, &alias_bindings, alias_stack);
    alias_stack.remove(&alias_identity);
    canonical
}

fn canonical_nominal_type(
    graph: &ModuleGraph,
    module_path: &[String],
    key: &super::authority_value_gate_defs::DefinitionKey,
    type_path: &syn::TypePath,
    bindings: &TypeBindings,
    alias_stack: &mut BTreeSet<String>,
) -> Type {
    let canonical_path = if key.0.is_empty() {
        format!("crate::{}", key.1)
    } else {
        format!("crate::{}::{}", key.0.join("::"), key.1)
    };
    let mut canonical: syn::TypePath = syn::parse_str(&canonical_path).unwrap_or(type_path.clone());
    if let (Some(source), Some(target)) = (
        type_path.path.segments.last(),
        canonical.path.segments.last_mut(),
    ) {
        target.arguments = canonical_arguments(
            graph,
            module_path,
            source.arguments.clone(),
            bindings,
            alias_stack,
        );
    }
    Type::Path(canonical)
}

fn bind_alias_arguments(
    alias: &syn::ItemType,
    arguments: &PathArguments,
    bindings: &mut TypeBindings,
) {
    let PathArguments::AngleBracketed(arguments) = arguments else {
        return;
    };
    let supplied = arguments.args.iter().filter_map(|argument| match argument {
        GenericArgument::Type(ty) => Some(ty.clone()),
        _ => None,
    });
    for (parameter, supplied) in alias.generics.type_params().zip(supplied) {
        bindings.insert(parameter.ident.to_string(), supplied);
    }
}

fn canonical_children(
    graph: &ModuleGraph,
    module_path: &[String],
    mut ty: Type,
    bindings: &TypeBindings,
    alias_stack: &mut BTreeSet<String>,
) -> Type {
    match &mut ty {
        Type::Array(array) => {
            canonicalize_boxed_type(graph, module_path, &mut array.elem, bindings, alias_stack)
        }
        Type::Group(group) => {
            canonicalize_boxed_type(graph, module_path, &mut group.elem, bindings, alias_stack)
        }
        Type::Paren(paren) => {
            canonicalize_boxed_type(graph, module_path, &mut paren.elem, bindings, alias_stack)
        }
        Type::Ptr(pointer) => {
            canonicalize_boxed_type(graph, module_path, &mut pointer.elem, bindings, alias_stack)
        }
        Type::Reference(reference) => canonicalize_boxed_type(
            graph,
            module_path,
            &mut reference.elem,
            bindings,
            alias_stack,
        ),
        Type::Slice(slice) => {
            canonicalize_boxed_type(graph, module_path, &mut slice.elem, bindings, alias_stack)
        }
        Type::Tuple(tuple) => {
            for element in &mut tuple.elems {
                *element = canonical_type_inner(graph, module_path, element, bindings, alias_stack);
            }
        }
        Type::Path(path) => {
            for segment in &mut path.path.segments {
                segment.arguments = canonical_arguments(
                    graph,
                    module_path,
                    segment.arguments.clone(),
                    bindings,
                    alias_stack,
                );
            }
        }
        _ => {}
    }
    ty
}

fn canonicalize_boxed_type(
    graph: &ModuleGraph,
    module_path: &[String],
    ty: &mut Box<Type>,
    bindings: &TypeBindings,
    alias_stack: &mut BTreeSet<String>,
) {
    **ty = canonical_type_inner(graph, module_path, ty, bindings, alias_stack);
}

fn canonical_arguments(
    graph: &ModuleGraph,
    module_path: &[String],
    mut arguments: PathArguments,
    bindings: &TypeBindings,
    alias_stack: &mut BTreeSet<String>,
) -> PathArguments {
    if let PathArguments::AngleBracketed(values) = &mut arguments {
        for argument in &mut values.args {
            match argument {
                GenericArgument::Type(ty) => {
                    *ty = canonical_type_inner(graph, module_path, ty, bindings, alias_stack);
                }
                GenericArgument::AssocType(binding) => {
                    binding.ty = canonical_type_inner(
                        graph,
                        module_path,
                        &binding.ty,
                        bindings,
                        alias_stack,
                    );
                }
                _ => {}
            }
        }
    }
    arguments
}
