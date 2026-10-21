use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{spanned::Spanned, Data, Fields, Member};

use super::grammar::ResidentValue;

pub(crate) fn emit(value: &ResidentValue) -> TokenStream {
    let ident = &value.ident;
    let contract = &value.contract;
    let body = match &value.data {
        Data::Struct(data) => {
            let charges = data.fields.iter().enumerate().map(|(index, field)| {
                let member = field.ident.clone().map_or_else(
                    || Member::Unnamed(syn::Index::from(index)), Member::Named,
                );
                quote_spanned! {field.ty.span()=>
                    .saturating_add(#contract::ChargedBytes::additional_charged_bytes(&self.#member))
                }
            });
            quote! { 0_u64 #(#charges)* }
        }
        Data::Enum(data) => {
            let arms = data.variants.iter().map(|variant| {
                let variant_ident = &variant.ident;
                let bindings: Vec<_> = (0..variant.fields.len())
                    .map(|index| format_ident!("__resident_field_{index}"))
                    .collect();
                let pattern = match &variant.fields {
                    Fields::Named(fields) => {
                        let names = fields.named.iter().map(|field| &field.ident);
                        quote! { { #(#names: #bindings),* } }
                    }
                    Fields::Unnamed(_) => quote! { (#(#bindings),*) },
                    Fields::Unit => quote! {},
                };
                let charges = variant.fields.iter().zip(&bindings).map(|(field, binding)| {
                    quote_spanned! {field.ty.span()=>
                        .saturating_add(#contract::ChargedBytes::additional_charged_bytes(#binding))
                    }
                });
                quote! { Self::#variant_ident #pattern => 0_u64 #(#charges)* }
            });
            if data.variants.is_empty() {
                quote! { match *self {} }
            } else {
                quote! { match self { #(#arms),* } }
            }
        }
        Data::Union(_) => unreachable!("resident grammar rejects unions"),
    };
    quote! {
        impl #contract::ChargedBytes for #ident {
            fn additional_charged_bytes(&self) -> u64 { #body }
        }
    }
}
