//! Alias-aware generic matching for associated projection impls.

use std::collections::{BTreeMap, BTreeSet};

use quote::ToTokens;
use syn::{GenericArgument, PathArguments, Type};

use super::crate_modules::ModuleGraph;

pub(super) type TypeBindings = BTreeMap<String, Type>;

pub(super) fn unify_impl_projection(
    graph: &ModuleGraph,
    impl_module: &[String],
    item_impl: &syn::ItemImpl,
    projected_module: &[String],
    projected_self: &Type,
    projection_trait_arguments: &PathArguments,
) -> Option<TypeBindings> {
    let generic_names = item_impl
        .generics
        .type_params()
        .map(|parameter| parameter.ident.to_string())
        .collect::<BTreeSet<_>>();
    let pattern = super::authority_value_gate_projection_canonical::canonical_type(
        graph,
        impl_module,
        &item_impl.self_ty,
        &TypeBindings::new(),
    );
    let actual = super::authority_value_gate_projection_canonical::canonical_type(
        graph,
        projected_module,
        projected_self,
        &TypeBindings::new(),
    );
    let mut bindings = TypeBindings::new();
    if !unify_types(&pattern, &actual, &generic_names, &mut bindings) {
        return None;
    }
    let (_, trait_path, _) = item_impl.trait_.as_ref()?;
    let impl_arguments = &trait_path.segments.last()?.arguments;
    if !unify_arguments(
        impl_arguments,
        projection_trait_arguments,
        &generic_names,
        &mut bindings,
    ) || !super::authority_value_gate_projection_bounds::local_type_parameter_bounds_hold(
        graph,
        impl_module,
        item_impl,
        &bindings,
    ) {
        return None;
    }
    Some(bindings)
}

fn unify_types(
    pattern: &Type,
    actual: &Type,
    generic_names: &BTreeSet<String>,
    bindings: &mut TypeBindings,
) -> bool {
    if let Type::Path(path) = pattern {
        if path.qself.is_none() && path.path.segments.len() == 1 {
            let name = path.path.segments[0].ident.to_string();
            if generic_names.contains(&name) {
                return match bindings.get(&name) {
                    Some(bound) => same_type(bound, actual),
                    None => {
                        bindings.insert(name, actual.clone());
                        true
                    }
                };
            }
        }
    }
    match (pattern, actual) {
        (Type::Array(left), Type::Array(right)) => {
            left.len.to_token_stream().to_string() == right.len.to_token_stream().to_string()
                && unify_types(&left.elem, &right.elem, generic_names, bindings)
        }
        (Type::Group(left), Type::Group(right)) => {
            unify_types(&left.elem, &right.elem, generic_names, bindings)
        }
        (Type::Paren(left), Type::Paren(right)) => {
            unify_types(&left.elem, &right.elem, generic_names, bindings)
        }
        (Type::Ptr(left), Type::Ptr(right)) => {
            left.mutability.is_some() == right.mutability.is_some()
                && unify_types(&left.elem, &right.elem, generic_names, bindings)
        }
        (Type::Reference(left), Type::Reference(right)) => {
            left.mutability.is_some() == right.mutability.is_some()
                && unify_types(&left.elem, &right.elem, generic_names, bindings)
        }
        (Type::Slice(left), Type::Slice(right)) => {
            unify_types(&left.elem, &right.elem, generic_names, bindings)
        }
        (Type::Tuple(left), Type::Tuple(right)) => {
            left.elems.len() == right.elems.len()
                && left
                    .elems
                    .iter()
                    .zip(&right.elems)
                    .all(|(left, right)| unify_types(left, right, generic_names, bindings))
        }
        (Type::Path(left), Type::Path(right)) => {
            left.path.segments.len() == right.path.segments.len()
                && left
                    .path
                    .segments
                    .iter()
                    .zip(&right.path.segments)
                    .all(|(left, right)| {
                        left.ident == right.ident
                            && unify_arguments(
                                &left.arguments,
                                &right.arguments,
                                generic_names,
                                bindings,
                            )
                    })
        }
        _ => same_type(pattern, actual),
    }
}

fn unify_arguments(
    pattern: &PathArguments,
    actual: &PathArguments,
    generic_names: &BTreeSet<String>,
    bindings: &mut TypeBindings,
) -> bool {
    match (pattern, actual) {
        (PathArguments::None, PathArguments::None) => true,
        (PathArguments::AngleBracketed(left), PathArguments::AngleBracketed(right)) => {
            left.args.len() == right.args.len()
                && left
                    .args
                    .iter()
                    .zip(&right.args)
                    .all(|(left, right)| match (left, right) {
                        (GenericArgument::Type(left), GenericArgument::Type(right)) => {
                            unify_types(left, right, generic_names, bindings)
                        }
                        _ => {
                            left.to_token_stream().to_string()
                                == right.to_token_stream().to_string()
                        }
                    })
        }
        (PathArguments::Parenthesized(left), PathArguments::Parenthesized(right)) => {
            left.to_token_stream().to_string() == right.to_token_stream().to_string()
        }
        _ => false,
    }
}

pub(super) fn substitute_type(ty: &Type, bindings: &TypeBindings) -> Type {
    if let Type::Path(path) = ty {
        if path.qself.is_none() && path.path.segments.len() == 1 {
            if let Some(bound) = bindings.get(&path.path.segments[0].ident.to_string()) {
                return bound.clone();
            }
        }
    }
    let mut substituted = ty.clone();
    match &mut substituted {
        Type::Array(array) => *array.elem = substitute_type(&array.elem, bindings),
        Type::Group(group) => *group.elem = substitute_type(&group.elem, bindings),
        Type::Paren(paren) => *paren.elem = substitute_type(&paren.elem, bindings),
        Type::Ptr(pointer) => *pointer.elem = substitute_type(&pointer.elem, bindings),
        Type::Reference(reference) => {
            *reference.elem = substitute_type(&reference.elem, bindings);
        }
        Type::Slice(slice) => *slice.elem = substitute_type(&slice.elem, bindings),
        Type::Tuple(tuple) => {
            for element in &mut tuple.elems {
                *element = substitute_type(element, bindings);
            }
        }
        Type::Path(path) => {
            for segment in &mut path.path.segments {
                if let PathArguments::AngleBracketed(arguments) = &mut segment.arguments {
                    for argument in &mut arguments.args {
                        match argument {
                            GenericArgument::Type(ty) => *ty = substitute_type(ty, bindings),
                            GenericArgument::AssocType(binding) => {
                                binding.ty = substitute_type(&binding.ty, bindings);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        _ => {}
    }
    substituted
}

fn same_type(left: &Type, right: &Type) -> bool {
    left.to_token_stream().to_string() == right.to_token_stream().to_string()
}
