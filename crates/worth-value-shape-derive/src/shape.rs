use crate::grammar::{
    types::{delegate_at, field_delegate},
    Field, Value, ValueKind, VariantPayload,
};
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn emit(value: &Value) -> TokenStream {
    let facade = &value.facade;
    let ident = &value.ident;
    let shape = shape_body(value);
    let nullable = match &value.kind {
        ValueKind::Unit => quote!(true),
        ValueKind::Newtype(ty) => {
            let ty = delegate_at(ty, facade).expect("validated type");
            quote!(<#ty as #facade::ValueSeedDelegate>::WIRE_NULLABLE)
        }
        _ => quote!(false),
    };
    let validation = validation(value);
    let decode = crate::seed::emit(value, false);
    let admitted_decode = crate::seed::emit(value, true);
    quote! {
        impl #facade::CanonicalValue for #ident {
            const SUPPORTS_ADMISSION: bool = true;
            const WIRE_NULLABLE: bool = #nullable;
            const SHAPE_BODY: #facade::ValueShapeBody = { #shape };
            fn validate_bounds(&self, bounds: #facade::ValueDecodeBounds) -> Result<(), #facade::ValueCodecDenial> { #validation }
            fn deserialize_bounded<'de, D: ::serde::Deserializer<'de>>(deserializer: D, bounds: #facade::ValueDecodeBounds) -> Result<Self, D::Error> { #decode }
            fn deserialize_admitted<'de, D: ::serde::Deserializer<'de>>(deserializer: D, bounds: #facade::ValueDecodeBounds, admission: &mut #facade::ValueDecodeAdmission<'_>) -> Result<Self, D::Error> { #admitted_decode }
            fn deserialize_reporting<'de, D: ::serde::Deserializer<'de>>(deserializer: D, bounds: #facade::ValueDecodeBounds, admission: &mut #facade::ValueDecodeAdmission<'_>) -> Result<Self, D::Error> { #admitted_decode }
        }
    }
}
fn fields_shape(fields: &[Field], facade: &syn::Path) -> TokenStream {
    let fields = fields.iter().map(|field| {
        let name = &field.wire_name;
        let ty = field_delegate(field, facade);
        quote!(#facade::ValueFieldDefinition { name: #name, value: <#ty as #facade::ValueSeedDelegate>::EDGE })
    });
    quote!(&[#(#fields),*])
}
fn shape_body(value: &Value) -> TokenStream {
    let facade = &value.facade;
    match &value.kind {
        ValueKind::Struct(fields) => {
            let fields = fields_shape(fields, facade);
            quote!(#facade::ValueShapeBody::Struct(#fields))
        }
        ValueKind::Newtype(ty) => {
            let ty = delegate_at(ty, facade).expect("validated type");
            quote!(#facade::ValueShapeBody::Transparent(<#ty as #facade::ValueSeedDelegate>::EDGE))
        }
        ValueKind::Unit => {
            quote!(#facade::ValueShapeBody::Primitive(#facade::ValuePrimitive::Unit))
        }
        ValueKind::Enum(variants) => {
            let variants = variants.iter().map(|variant| {
                let name = &variant.wire_name;
                let payload = match &variant.payload {
                    VariantPayload::Newtype(ty) => {
                        let ty = delegate_at(ty, facade).expect("validated type");
                        quote!(Some(<#ty as #facade::ValueSeedDelegate>::EDGE))
                    }
                    VariantPayload::Unit => quote!(None),
                    VariantPayload::Struct(_) => {
                        let helper = crate::external_payload::ident(&value.ident, &variant.ident);
                        quote!(Some(#facade::ValueShapeEdge::value::<#helper>()))
                    }
                };
                quote!(#facade::ValueVariantDefinition { name: #name, payload: #payload })
            });
            quote!(#facade::ValueShapeBody::Enum(&[#(#variants),*]))
        }
        ValueKind::TaggedEnum { tag, variants } => {
            let variants = variants.iter().map(|variant| {
                let name = &variant.wire_name;
                let fields = fields_shape(&variant.fields, facade);
                quote!(#facade::ValueTaggedVariantDefinition { name: #name, fields: #fields })
            });
            quote!(#facade::ValueShapeBody::TaggedEnum { tag: #tag, variants: &[#(#variants),*] })
        }
    }
}
fn validate_fields(fields: &[Field], bindings: bool, facade: &syn::Path) -> TokenStream {
    let statements = fields.iter().map(|field| {
        let member = &field.ident;
        let name = &field.wire_name;
        let ty = field_delegate(field, facade);
        let value = if bindings {
            quote!(#member)
        } else {
            quote!(&self.#member)
        };
        quote! {
            child_bounds.check_string_bytes(#name.len())?;
            <#ty as #facade::ValueSeedDelegate>::validate(#value, child_bounds)?;
        }
    });
    quote!(#(#statements)*)
}
fn validation(value: &Value) -> TokenStream {
    let facade = &value.facade;
    let body = match &value.kind {
        ValueKind::Struct(fields) => validate_fields(fields, false, facade),
        ValueKind::Newtype(ty) => {
            let ty = delegate_at(ty, facade).expect("validated type");
            return quote!(<#ty as #facade::ValueSeedDelegate>::validate(&self.0, bounds));
        }
        ValueKind::Unit => return quote!(let _ = bounds; Ok(())),
        ValueKind::Enum(variants) => {
            let arms = variants.iter().map(|variant| {
                let member = &variant.ident;
                let name = &variant.wire_name;
                match &variant.payload {
                    VariantPayload::Newtype(ty) => {
                        let ty = delegate_at(ty, facade).expect("validated type");
                        quote!(Self::#member(payload) => {
                            child_bounds.check_string_bytes(#name.len())?;
                            <#ty as #facade::ValueSeedDelegate>::validate(payload, child_bounds)?;
                        })
                    }
                    VariantPayload::Unit => {
                        quote!(Self::#member => child_bounds.check_string_bytes(#name.len())?)
                    }
                    VariantPayload::Struct(fields) => {
                        let members = fields.iter().map(|field| &field.ident);
                        let validate = validate_fields(fields, true, facade);
                        quote!(Self::#member { #(#members),* } => {
                            child_bounds.check_string_bytes(#name.len())?;
                            let child_bounds = child_bounds.descend()?;
                            #validate
                        })
                    }
                }
            });
            quote!(match self { #(#arms),* })
        }
        ValueKind::TaggedEnum { tag, variants } => {
            let arms = variants.iter().map(|variant| {
                let member = &variant.ident;
                let name = &variant.wire_name;
                let fields = variant.fields.iter().map(|field| &field.ident);
                let pattern = if variant.unit {
                    quote!()
                } else {
                    quote!({ #(#fields),* })
                };
                let validate = validate_fields(&variant.fields, true, facade);
                quote!(Self::#member #pattern => {
                    child_bounds.check_string_bytes(#name.len())?;
                    #validate
                })
            });
            quote!(child_bounds.check_string_bytes(#tag.len())?; match self { #(#arms),* })
        }
    };
    quote!(let child_bounds = bounds.descend()?; let _ = child_bounds; #body Ok(()))
}
