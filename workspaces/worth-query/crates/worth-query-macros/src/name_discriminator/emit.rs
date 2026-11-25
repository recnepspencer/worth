//! The `ApplicationNameDiscriminator` impl, written as the `variant`,
//! `field` and `value` calls a hand-written `encode_canonical` makes.
use super::grammar::{DiscriminatorDeclaration, Members, Shape, VariantDeclaration};
use crate::facade_paths::naming;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{DeriveInput, Ident, Index};

pub(super) fn discriminator(
    input: &DeriveInput,
    declaration: &DiscriminatorDeclaration,
) -> TokenStream {
    let (naming, discriminator, identity) = (naming(), &input.ident, &declaration.identity);
    let referenced_roles = declaration.referenced_roles.as_ref().map(|roles| {
        quote! {
            fn referenced_roles() -> &'static [&'static str] {
                &[#(#roles),*]
            }
        }
    });
    let encode = match &declaration.shape {
        Shape::Struct(members) => struct_members(members),
        Shape::Enum(variants) => {
            let arms = variants.iter().map(variant_arm);
            quote!(match self { #(#arms)* })
        }
    };
    let decode = match &declaration.shape {
        Shape::Struct(members) => {
            let construct = construct(quote!(Self), members);
            quote!(::core::result::Result::Ok(#construct))
        }
        Shape::Enum(variants) => {
            let arms = variants.iter().map(|declaration| {
                let (variant, name) = (&declaration.variant, &declaration.name);
                let construct = construct(quote!(Self::#variant), &declaration.members);
                quote!(#name => ::core::result::Result::Ok(#construct),)
            });
            quote! {
                match decoder.variant()? {
                    #(#arms)*
                    other => ::core::result::Result::Err(decoder.unknown_variant(other)),
                }
            }
        }
    };
    quote! {
        impl #naming::ApplicationNameDiscriminatorDecode for #discriminator {
            // A unit struct reads nothing after its identity.
            #[allow(unused_variables)]
            fn decode_canonical(
                decoder: &mut #naming::ApplicationNameDiscriminatorDecoder<'_>,
            ) -> ::core::result::Result<Self, #naming::ApplicationNameDiscriminatorDecodeDenial> {
                #decode
            }
        }
        impl #naming::ApplicationNameDiscriminator for #discriminator {
            const DISCRIMINATOR_IDENTITY: &'static str = #identity;
            #referenced_roles
            // A unit struct writes nothing after its identity.
            #[allow(unused_variables)]
            fn encode_canonical(
                &self,
                encoder: &mut #naming::ApplicationNameDiscriminatorEncoder,
            ) {
                #encode
            }
        }
    }
}

/// `path { field: read, .. }` or `path(read, ..)`: one read per field, in
/// the order the encoding writes them.
fn construct(path: TokenStream, members: &Members) -> TokenStream {
    match members {
        Members::Unit => path,
        Members::Named(fields) => {
            let reads = fields.iter().map(
                |(member, name, ty)| quote_spanned!(ty.span()=> #member: decoder.field(#name)?),
            );
            quote!(#path { #(#reads,)* })
        }
        Members::Positional(types) => {
            let reads = types
                .iter()
                .map(|ty| quote_spanned!(ty.span()=> decoder.value()?));
            quote!(#path(#(#reads,)*))
        }
    }
}

/// One write per field of a struct, read through `self`.
fn struct_members(members: &Members) -> TokenStream {
    match members {
        Members::Unit => TokenStream::new(),
        Members::Named(fields) => fields
            .iter()
            .map(|(member, name, ty)| {
                // Spanned to the field's type, so a type that is not a
                // discriminator field is refused there.
                quote_spanned!(ty.span()=> encoder.field(#name, &self.#member);)
            })
            .collect(),
        Members::Positional(types) => types
            .iter()
            .enumerate()
            .map(|(position, ty)| {
                let member = Index::from(position);
                quote_spanned!(ty.span()=> encoder.value(&self.#member);)
            })
            .collect(),
    }
}

/// `Self::Variant { .. } => { variant name, then one write per field }`.
fn variant_arm(declaration: &VariantDeclaration) -> TokenStream {
    let VariantDeclaration {
        variant,
        name,
        members,
    } = declaration;
    // Bound under generated names, so no field shadows `encoder`.
    let bound: Vec<Ident> = (0..members.len())
        .map(|position| format_ident!("field_{position}"))
        .collect();
    let (pattern, writes): (TokenStream, TokenStream) = match members {
        Members::Unit => (TokenStream::new(), TokenStream::new()),
        Members::Named(fields) => {
            let members = fields.iter().map(|(member, _, _)| member);
            let writes = fields.iter().zip(&bound).map(
                |((_, name, ty), bound)| quote_spanned!(ty.span()=> encoder.field(#name, #bound);),
            );
            (quote!({ #(#members: #bound,)* }), writes.collect())
        }
        Members::Positional(types) => {
            let writes = types
                .iter()
                .zip(&bound)
                .map(|(ty, bound)| quote_spanned!(ty.span()=> encoder.value(#bound);));
            (quote!((#(#bound,)*)), writes.collect())
        }
    };
    quote! {
        Self::#variant #pattern => {
            encoder.variant(#name);
            #writes
        }
    }
}
