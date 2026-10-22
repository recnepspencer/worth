use proc_macro2::TokenStream;
use quote::quote;

use crate::grammar::{Value, ValueKind, VariantPayload};

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
        ValueKind::TaggedEnum { tag, variants } => {
            let arms = variants.iter().map(|variant| {
                let member = &variant.ident;
                let name = &variant.wire_name;
                let length = variant.fields.len() + 1;
                let fields = variant.fields.iter().map(|field| &field.ident).collect::<Vec<_>>();
                let names = variant.fields.iter().map(|field| &field.wire_name);
                let pattern = if variant.unit { quote!() } else { quote!({ #(#fields),* }) };
                quote!(Self::#member #pattern => {
                    let mut state = ::serde::Serializer::serialize_struct(serializer, #wire_name, #length)?;
                    ::serde::ser::SerializeStruct::serialize_field(&mut state, #tag, #name)?;
                    #(::serde::ser::SerializeStruct::serialize_field(&mut state, #names, #fields)?;)*
                    ::serde::ser::SerializeStruct::end(state)
                })
            });
            quote!(match self { #(#arms),* })
        }
        ValueKind::Enum(variants) => {
            let arms = variants.iter().enumerate().map(|(index, variant)| {
                let variant_ident = &variant.ident;
                let variant_name = &variant.wire_name;
                let index = index as u32;
                match &variant.payload {
                    VariantPayload::Newtype(_) => quote! {
                        Self::#variant_ident(payload) => ::serde::Serializer::serialize_newtype_variant(
                            serializer, #wire_name, #index, #variant_name, payload,
                        )
                    },
                    VariantPayload::Unit => quote! {
                        Self::#variant_ident => ::serde::Serializer::serialize_unit_variant(
                            serializer, #wire_name, #index, #variant_name,
                        )
                    },
                    VariantPayload::Struct(fields) => {
                        let members = fields.iter().map(|field| &field.ident).collect::<Vec<_>>();
                        let names = fields.iter().map(|field| &field.wire_name);
                        let length = fields.len();
                        quote!(Self::#variant_ident { #(#members),* } => {
                            let mut state = ::serde::Serializer::serialize_struct_variant(serializer, #wire_name, #index, #variant_name, #length)?;
                            #(::serde::ser::SerializeStructVariant::serialize_field(&mut state, #names, #members)?;)*
                            ::serde::ser::SerializeStructVariant::end(state)
                        })
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
