use crate::grammar::{
    types::{delegate, field_delegate},
    Field, Value, ValueKind, Variant, VariantPayload,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(crate) fn emit(value: &Value, admitted: bool) -> TokenStream {
    let ident = &value.ident;
    let wire_name = &value.wire_name;
    let admission = if admitted {
        quote!(Some(admission))
    } else {
        quote!(None)
    };
    let visitor = quote!(__WorthValueVisitor { bounds, admission: #admission });
    let (visit, dispatch) = match &value.kind {
        ValueKind::Struct(fields) => {
            let members = fields.iter().map(|field| &field.ident);
            let values = slots(fields);
            let body = map_fields(fields, quote!(#ident { #(#members: #values),* }));
            let names = fields.iter().map(|field| &field.wire_name);
            (
                quote!(fn visit_map<M: ::serde::de::MapAccess<'de>>(mut self, mut map: M) -> Result<Self::Value, M::Error> {
                    let child_bounds = self.bounds.descend().map_err(|cause| ::worth_foundational::facade::value_decode_error::<M::Error>(cause, self.admission.as_deref_mut()))?;
                    #body
                }),
                quote!(::serde::Deserializer::deserialize_struct(deserializer, #wire_name, &[#(#names),*], #visitor)),
            )
        }
        ValueKind::Newtype(ty) => {
            let ty = delegate(ty).expect("validated type");
            (
                quote!(fn visit_newtype_struct<D: ::serde::Deserializer<'de>>(self, decoder: D) -> Result<Self::Value, D::Error> {
                    ::serde::de::DeserializeSeed::deserialize(::worth_foundational::facade::DelegatedValueSeed::<#ty>::new(self.bounds, self.admission), decoder).map(#ident)
                }),
                quote!(::serde::Deserializer::deserialize_newtype_struct(deserializer, #wire_name, #visitor)),
            )
        }
        ValueKind::Unit => (
            quote!(fn visit_unit<E: ::serde::de::Error>(self) -> Result<Self::Value, E> { let _ = self; Ok(#ident) }),
            quote!(::serde::Deserializer::deserialize_unit_struct(deserializer, #wire_name, #visitor)),
        ),
        ValueKind::Enum(variants) => {
            let visit = external_visitor(ident, variants);
            let names = variants.iter().map(|variant| &variant.wire_name);
            (
                visit,
                quote!(::serde::Deserializer::deserialize_enum(deserializer, #wire_name, &[#(#names),*], #visitor)),
            )
        }
        ValueKind::TaggedEnum { tag, variants } => {
            let arms = variants.iter().map(|variant| {
                let member = &variant.ident;
                let name = &variant.wire_name;
                let members = variant.fields.iter().map(|field| &field.ident);
                let values = slots(&variant.fields);
                let construct = if variant.unit {
                    quote!(#ident::#member)
                } else {
                    quote!(#ident::#member { #(#members: #values),* })
                };
                let body = map_fields(&variant.fields, construct);
                quote!(#name => { #body })
            });
            (
                quote!(fn visit_map<M: ::serde::de::MapAccess<'de>>(mut self, mut map: M) -> Result<Self::Value, M::Error> {
                    let child_bounds = self.bounds.descend().map_err(|cause| ::worth_foundational::facade::value_decode_error::<M::Error>(cause, self.admission.as_deref_mut()))?;
                    let key = ::serde::de::MapAccess::next_key_seed(&mut map,
                        ::worth_foundational::facade::CanonicalValueSeed::<::std::string::String>::with_optional_admission(child_bounds, self.admission.as_deref_mut()))?;
                    if key.as_deref() != Some(#tag) { return Err(::serde::de::Error::custom(::worth_foundational::facade::ValueCodecDenial::Malformed)); }
                    let tag = ::serde::de::MapAccess::next_value_seed(&mut map,
                        ::worth_foundational::facade::CanonicalValueSeed::<::std::string::String>::with_optional_admission(child_bounds, self.admission.as_deref_mut()))?;
                    match tag.as_str() { #(#arms),*, _ => Err(::serde::de::Error::custom(::worth_foundational::facade::ValueCodecDenial::Malformed)) }
                }),
                quote!(::serde::Deserializer::deserialize_map(deserializer, #visitor)),
            )
        }
    };
    quote! {
        struct __WorthValueVisitor<'loan, 'request> {
            bounds: ::worth_foundational::facade::ValueDecodeBounds,
            admission: Option<&'loan mut ::worth_foundational::facade::ValueDecodeAdmission<'request>>,
        }
        impl<'de> ::serde::de::Visitor<'de> for __WorthValueVisitor<'_, '_> {
            type Value = #ident;
            fn expecting(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                formatter.write_str("a canonical derived value within admitted bounds")
            }
            #visit
        }
        #dispatch
    }
}
fn slots(fields: &[Field]) -> Vec<syn::Ident> {
    (0..fields.len())
        .map(|index| format_ident!("__worth_field_{index}"))
        .collect()
}
fn map_fields(fields: &[Field], construct: TokenStream) -> TokenStream {
    let slots = slots(fields);
    let types = fields.iter().map(|field| &field.ty).collect::<Vec<_>>();
    let delegates = fields.iter().map(field_delegate).collect::<Vec<_>>();
    let names = fields
        .iter()
        .map(|field| &field.wire_name)
        .collect::<Vec<_>>();
    quote! {
        #(let mut #slots: Option<#types> = None;)*
        while let Some(key) = ::serde::de::MapAccess::next_key_seed(&mut map,
            ::worth_foundational::facade::CanonicalValueSeed::<::std::string::String>::with_optional_admission(child_bounds, self.admission.as_deref_mut()))? {
            match key.as_str() {
                #(#names => {
                    if #slots.is_some() { return Err(::serde::de::Error::custom(::worth_foundational::facade::ValueCodecDenial::Malformed)); }
                    #slots = Some(::serde::de::MapAccess::next_value_seed(&mut map,
                        ::worth_foundational::facade::DelegatedValueSeed::<#delegates>::new(child_bounds, self.admission.as_deref_mut()))?);
                },)*
                _ => return Err(::serde::de::Error::custom(::worth_foundational::facade::ValueCodecDenial::Malformed)),
            }
        }
        #(let #slots = #slots.ok_or_else(|| ::serde::de::Error::custom(::worth_foundational::facade::ValueCodecDenial::Malformed))?;)*
        Ok(#construct)
    }
}
fn external_visitor(ident: &syn::Ident, variants: &[Variant]) -> TokenStream {
    let arms = variants.iter().map(|variant| {
        let member = &variant.ident;
        let name = &variant.wire_name;
        match &variant.payload {
            VariantPayload::Newtype(ty) => { let ty = delegate(ty).expect("validated type"); quote!(#name => ::serde::de::VariantAccess::newtype_variant_seed(variant,
                ::worth_foundational::facade::DelegatedValueSeed::<#ty>::new(child_bounds, self.admission.as_deref_mut())).map(#ident::#member)) },
            VariantPayload::Unit => quote!(#name => { ::serde::de::VariantAccess::unit_variant(variant)?; Ok(#ident::#member) }),
            VariantPayload::Struct(fields) => {
                let helper = crate::external_payload::ident(ident, member);
                let members = fields.iter().map(|field| &field.ident).collect::<Vec<_>>();
                quote!(#name => ::serde::de::VariantAccess::newtype_variant_seed(variant,
                    ::worth_foundational::facade::CanonicalValueSeed::<#helper>::with_optional_admission(child_bounds, self.admission.as_deref_mut()))
                    .map(|payload| #ident::#member { #(#members: payload.#members),* }))
            },
        }
    });
    quote!(fn visit_enum<A: ::serde::de::EnumAccess<'de>>(mut self, access: A) -> Result<Self::Value, A::Error> {
        let child_bounds = self.bounds.descend().map_err(|cause| ::worth_foundational::facade::value_decode_error::<A::Error>(cause, self.admission.as_deref_mut()))?;
        let (name, variant) = ::serde::de::EnumAccess::variant_seed(access,
            ::worth_foundational::facade::CanonicalValueSeed::<::std::string::String>::with_optional_admission(child_bounds, self.admission.as_deref_mut()))?;
        match name.as_str() { #(#arms),*, _ => Err(::serde::de::Error::custom(::worth_foundational::facade::ValueCodecDenial::Malformed)) }
    })
}
