use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::grammar::{Field, Value, ValueKind, Variant};

pub(crate) fn emit(value: &Value) -> TokenStream {
    let ident = &value.ident;
    let wire_name = &value.wire_name;
    let (visit, dispatch) = match &value.kind {
        ValueKind::Struct(fields) => {
            let visit = struct_visitor(ident, fields);
            let names = fields.iter().map(|field| &field.wire_name);
            (
                visit,
                quote! {
                    ::serde::Deserializer::deserialize_struct(
                        deserializer, #wire_name, &[#(#names),*], __WorthValueVisitor { bounds },
                    )
                },
            )
        }
        ValueKind::Newtype(ty) => (
            quote! {
                fn visit_newtype_struct<D: ::serde::Deserializer<'de>>(
                    self, deserializer: D,
                ) -> ::core::result::Result<Self::Value, D::Error> {
                    ::serde::de::DeserializeSeed::deserialize(
                    ::worth_foundational::facade::CanonicalValueSeed::<#ty>::new(self.bounds),
                        deserializer,
                    ).map(#ident)
                }
            },
            quote! {
                ::serde::Deserializer::deserialize_newtype_struct(
                    deserializer, #wire_name, __WorthValueVisitor { bounds },
                )
            },
        ),
        ValueKind::Unit => (
            quote! {
                fn visit_unit<E: ::serde::de::Error>(self) -> ::core::result::Result<Self::Value, E> {
                let _ = self.bounds;
                    Ok(#ident)
                }
            },
            quote! {
                ::serde::Deserializer::deserialize_unit_struct(
                    deserializer, #wire_name, __WorthValueVisitor { bounds },
                )
            },
        ),
        ValueKind::Enum(variants) => {
            let visit = enum_visitor(ident, variants);
            let names = variants.iter().map(|variant| &variant.wire_name);
            (
                visit,
                quote! {
                    ::serde::Deserializer::deserialize_enum(
                        deserializer, #wire_name, &[#(#names),*], __WorthValueVisitor { bounds },
                    )
                },
            )
        }
    };
    quote! {
        struct __WorthValueVisitor {
            bounds: ::worth_foundational::facade::ValueDecodeBounds,
        }
        impl<'de> ::serde::de::Visitor<'de> for __WorthValueVisitor {
            type Value = #ident;

            fn expecting(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                formatter.write_str("a canonical derived value within admitted bounds")
            }

            #visit
        }
        #dispatch
    }
}

fn struct_visitor(ident: &syn::Ident, fields: &[Field]) -> TokenStream {
    let slots: Vec<_> = fields
        .iter()
        .enumerate()
        .map(|(index, _)| format_ident!("__worth_field_{index}"))
        .collect();
    let types: Vec<_> = fields.iter().map(|field| &field.ty).collect();
    let names: Vec<_> = fields.iter().map(|field| &field.wire_name).collect();
    let members = fields.iter().map(|field| &field.ident);
    quote! {
        fn visit_map<M: ::serde::de::MapAccess<'de>>(
            self, mut __worth_map: M,
        ) -> ::core::result::Result<Self::Value, M::Error> {
            let __worth_child_bounds = self.bounds.descend().map_err(::serde::de::Error::custom)?;
            #(let mut #slots: ::core::option::Option<#types> = None;)*
            while let Some(__worth_key) = ::serde::de::MapAccess::next_key_seed(
                &mut __worth_map,
                ::worth_foundational::facade::CanonicalValueSeed::<::std::string::String>::new(__worth_child_bounds),
            )? {
                match __worth_key.as_str() {
                    #(#names => {
                        if #slots.is_some() {
                            return Err(::serde::de::Error::duplicate_field(#names));
                        }
                        #slots = Some(::serde::de::MapAccess::next_value_seed(
                            &mut __worth_map,
                            ::worth_foundational::facade::CanonicalValueSeed::<#types>::new(__worth_child_bounds),
                        )?);
                    },)*
                    _ => return Err(::serde::de::Error::unknown_field(&__worth_key, &[#(#names),*])),
                }
            }
            Ok(#ident {
                #(#members: #slots.ok_or_else(|| ::serde::de::Error::missing_field(#names))?),*
            })
        }
    }
}

fn enum_visitor(ident: &syn::Ident, variants: &[Variant]) -> TokenStream {
    let arms = variants.iter().map(|variant| {
        let member = &variant.ident;
        let name = &variant.wire_name;
        match &variant.payload {
            Some(ty) => quote! {
                #name => ::serde::de::VariantAccess::newtype_variant_seed(
                    __worth_variant,
                    ::worth_foundational::facade::CanonicalValueSeed::<#ty>::new(__worth_child_bounds),
                ).map(#ident::#member),
            },
            None => quote! {
                #name => {
                    ::serde::de::VariantAccess::unit_variant(__worth_variant)?;
                    Ok(#ident::#member)
                },
            },
        }
    });
    let names = variants.iter().map(|variant| &variant.wire_name);
    quote! {
        fn visit_enum<A: ::serde::de::EnumAccess<'de>>(
            self, __worth_access: A,
        ) -> ::core::result::Result<Self::Value, A::Error> {
            let __worth_child_bounds = self.bounds.descend().map_err(::serde::de::Error::custom)?;
            let (__worth_name, __worth_variant) = ::serde::de::EnumAccess::variant_seed(
                __worth_access,
                ::worth_foundational::facade::CanonicalValueSeed::<::std::string::String>::new(__worth_child_bounds),
            )?;
            match __worth_name.as_str() {
                #(#arms)*
                _ => Err(::serde::de::Error::unknown_variant(&__worth_name, &[#(#names),*])),
            }
        }
    }
}
