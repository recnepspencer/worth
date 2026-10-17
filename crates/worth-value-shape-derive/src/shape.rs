use proc_macro2::TokenStream;
use quote::quote;

use crate::grammar::{Value, ValueKind};

pub(crate) fn emit(value: &Value) -> TokenStream {
    let ident = &value.ident;
    let shape = value_shape(value);
    let depth = shape_depth(value);
    let nullable = wire_nullable(value);
    let validation = validation(value);
    let decode = crate::seed::emit(value);
    quote! {
        impl ::worth_foundational::facade::CanonicalValue for #ident {
            const SHAPE_DEPTH: usize = #depth;
            const WIRE_NULLABLE: bool = #nullable;

            fn value_shape() -> ::worth_foundational::facade::ValueShape {
                assert!(Self::SHAPE_DEPTH <= 64, "WorthValue shape exceeds supported finite nesting");
                #shape
            }

            fn validate_bounds(
                &self,
                bounds: ::worth_foundational::facade::ValueDecodeBounds,
            ) -> ::core::result::Result<(), ::worth_foundational::facade::ValueCodecDenial> {
                #validation
            }

            fn deserialize_bounded<'de, D: ::serde::Deserializer<'de>>(
                deserializer: D,
                bounds: ::worth_foundational::facade::ValueDecodeBounds,
            ) -> ::core::result::Result<Self, D::Error> {
                #decode
            }
        }
    }
}

fn wire_nullable(value: &Value) -> TokenStream {
    match &value.kind {
        ValueKind::Unit => quote! { true },
        ValueKind::Newtype(ty) => quote! {
            <#ty as ::worth_foundational::facade::CanonicalValue>::WIRE_NULLABLE
        },
        ValueKind::Struct(_) | ValueKind::Enum(_) => quote! { false },
    }
}

fn shape_depth(value: &Value) -> TokenStream {
    let children = match &value.kind {
        ValueKind::Struct(fields) => fields.iter().map(|field| &field.ty).collect::<Vec<_>>(),
        ValueKind::Newtype(ty) => {
            return quote! {
                <#ty as ::worth_foundational::facade::CanonicalValue>::SHAPE_DEPTH
            }
        }
        ValueKind::Unit => return quote! { 0 },
        ValueKind::Enum(variants) => variants
            .iter()
            .filter_map(|variant| variant.payload.as_ref())
            .collect(),
    };
    quote! {{
        let maximum = 0usize;
        #(let maximum = {
            let child = <#children as ::worth_foundational::facade::CanonicalValue>::SHAPE_DEPTH;
            if child > maximum { child } else { maximum }
        };)*
        let depth = maximum + 1;
        assert!(depth <= 64, "WorthValue requires a finite wire shape no deeper than 64 levels");
        depth
    }}
}

fn value_shape(value: &Value) -> TokenStream {
    match &value.kind {
        ValueKind::Struct(fields) => {
            let names = fields.iter().map(|field| &field.wire_name);
            let types = fields.iter().map(|field| &field.ty);
            quote! {
                ::worth_foundational::facade::ValueShape::Struct(::std::vec![
                    #(::worth_foundational::facade::ValueFieldShape::new(
                        #names,
                        <#types as ::worth_foundational::facade::CanonicalValue>::value_shape(),
                    )),*
                ])
            }
        }
        ValueKind::Newtype(ty) => quote! {
            <#ty as ::worth_foundational::facade::CanonicalValue>::value_shape()
        },
        ValueKind::Unit => unit_shape(),
        ValueKind::Enum(variants) => {
            let descriptions = variants.iter().map(|variant| {
                let name = &variant.wire_name;
                match &variant.payload {
                    Some(ty) => quote! {
                        ::worth_foundational::facade::ValueVariantShape::newtype(
                            #name,
                            <#ty as ::worth_foundational::facade::CanonicalValue>::value_shape(),
                        )
                    },
                    None => quote! {
                        ::worth_foundational::facade::ValueVariantShape::unit(#name)
                    },
                }
            });
            quote! { ::worth_foundational::facade::ValueShape::Enum(::std::vec![#(#descriptions),*]) }
        }
    }
}

fn unit_shape() -> TokenStream {
    quote! { ::worth_foundational::facade::ValueShape::Primitive(
        ::worth_foundational::facade::ValuePrimitive::Unit,
    ) }
}

fn validation(value: &Value) -> TokenStream {
    let children = match &value.kind {
        ValueKind::Struct(fields) => {
            let statements = fields.iter().map(|field| {
                let member = &field.ident;
                let name = &field.wire_name;
                quote! {
                    child_bounds.check_string_bytes(#name.len())?;
                    ::worth_foundational::facade::CanonicalValue::validate_bounds(&self.#member, child_bounds)?;
                }
            });
            quote! { #(#statements)* }
        }
        ValueKind::Newtype(_) => {
            return quote! {
                ::worth_foundational::facade::CanonicalValue::validate_bounds(&self.0, bounds)
            }
        }
        ValueKind::Unit => return quote! { let _ = bounds; Ok(()) },
        ValueKind::Enum(variants) => {
            let arms = variants.iter().map(|variant| {
                let ident = &variant.ident;
                let name = &variant.wire_name;
                match variant.payload {
                    Some(_) => quote! {
                        Self::#ident(payload) => {
                            child_bounds.check_string_bytes(#name.len())?;
                            ::worth_foundational::facade::CanonicalValue::validate_bounds(payload, child_bounds)?;
                        },
                    },
                    None => quote! { Self::#ident => child_bounds.check_string_bytes(#name.len())?, },
                }
            });
            quote! { match self { #(#arms)* } }
        }
    };
    quote! {
        let child_bounds = bounds.descend()?;
        let _ = child_bounds;
        #children
        Ok(())
    }
}
