//! The `WorthQueryNameRole` impl and the canonical order of the role enum.
use super::grammar::{Role, RoleDeclaration};
use crate::facade_paths::naming;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub(super) fn roles(input: &DeriveInput, declaration: &RoleDeclaration) -> TokenStream {
    let (naming, role) = (naming(), &input.ident);
    let RoleDeclaration {
        roles,
        canonical_order,
    } = declaration;
    let ordered_names = canonical_order
        .iter()
        .map(|position| &roles[*position].name);
    let dependencies = canonical_order.iter().flat_map(|position| {
        let name = &roles[*position].name;
        roles[*position].depends_on.iter().map(move |dependency| {
            let depends_on = &roles[dependency.role].name;
            quote!((#name, #depends_on))
        })
    });
    let names = roles.iter().map(|declared| {
        let (pattern, name) = (pattern(declared, false), &declared.name);
        quote!(Self::#pattern => #name)
    });
    let indexes = roles.iter().map(|declared| {
        let pattern = pattern(declared, true);
        let index = match declared.indexed {
            true => quote!(::core::option::Option::Some(*index)),
            false => quote!(::core::option::Option::None),
        };
        quote!(Self::#pattern => #index)
    });
    let ranks = roles.iter().enumerate().map(|(position, declared)| {
        let rank = canonical_order
            .iter()
            .position(|ordered| *ordered == position)
            .expect("every role has its place in the canonical order");
        let (pattern, rank) = (pattern(declared, true), rank as u32);
        let index = match declared.indexed {
            true => quote!(*index),
            false => quote!(0u32),
        };
        quote!(Self::#pattern => (#rank, #index))
    });
    quote! {
        impl #naming::WorthQueryNameRole for #role {
            const ROLE_ORDER: &'static [&'static str] = &[#(#ordered_names),*];
            const ROLE_DEPENDENCIES: &'static [(&'static str, &'static str)] =
                &[#(#dependencies),*];
            fn role_name(&self) -> &'static str {
                match self { #(#names,)* }
            }
            fn role_index(&self) -> ::core::option::Option<u32> {
                match self { #(#indexes,)* }
            }
        }
        impl ::core::cmp::Ord for #role {
            fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
                // Canonical role order first, then the index as a number.
                let canonical = |role: &Self| -> (u32, u32) {
                    match role { #(#ranks,)* }
                };
                ::core::cmp::Ord::cmp(&canonical(self), &canonical(other))
            }
        }
        impl ::core::cmp::PartialOrd for #role {
            fn partial_cmp(&self, other: &Self) -> ::core::option::Option<::core::cmp::Ordering> {
                ::core::option::Option::Some(::core::cmp::Ord::cmp(self, other))
            }
        }
    }
}

/// `Variant` or, for an indexed role, `Variant(index)` (`Variant(_)` when the
/// index is not read).
fn pattern(role: &Role, bind_index: bool) -> TokenStream {
    let variant = &role.variant;
    match (role.indexed, bind_index) {
        (false, _) => quote!(#variant),
        (true, true) => quote!(#variant(index)),
        (true, false) => quote!(#variant(_)),
    }
}
