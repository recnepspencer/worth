use proc_macro2::TokenStream;
use quote::quote;

use crate::grammar::{Value, ValueKind};

pub(crate) fn emit(value: &Value) -> TokenStream {
    let ident = &value.ident;
    let wire_name = &value.wire_name;
    let body = match &value.kind {
        ValueKind::Struct(fields) => {
            let length = fields.len();
            let members = fields.iter().map(|field| &field.ident);
            let names = fields.iter().map(|field| &field.wire_name);
            quote! {
                let mut state = ::serde::Serializer::serialize_struct(serializer, #wire_name, #length)?;
                #(::serde::ser::SerializeStruct::serialize_field(&mut state, #names, &self.#members)?;)*
                ::serde::ser::SerializeStruct::end(state)
            }
        }
        ValueKind::Newtype(_) => quote! {
            ::serde::Serializer::serialize_newtype_struct(serializer, #wire_name, &self.0)
        },
        ValueKind::Unit => quote! {
            ::serde::Serializer::serialize_unit_struct(serializer, #wire_name)
        },
        ValueKind::Enum(variants) => {
            let arms = variants.iter().enumerate().map(|(index, variant)| {
                let variant_ident = &variant.ident;
                let variant_name = &variant.wire_name;
                let index = index as u32;
                match variant.payload {
                    Some(_) => quote! {
                        Self::#variant_ident(payload) => ::serde::Serializer::serialize_newtype_variant(
                            serializer, #wire_name, #index, #variant_name, payload,
                        )
                    },
                    None => quote! {
                        Self::#variant_ident => ::serde::Serializer::serialize_unit_variant(
                            serializer, #wire_name, #index, #variant_name,
                        )
                    },
                }
            });
            quote! { match self { #(#arms),* } }
        }
    };
    quote! {
        impl ::serde::Serialize for #ident {
            fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error> {
                #body
            }
        }
    }
}
