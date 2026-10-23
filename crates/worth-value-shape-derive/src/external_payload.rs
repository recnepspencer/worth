//! Named enum payloads lower through the same record grammar, without a second field list.
use crate::grammar::{Value, ValueKind, VariantPayload};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(crate) fn ident(value: &syn::Ident, variant: &syn::Ident) -> syn::Ident {
    format_ident!(
        "__WorthValue{}{}Payload",
        value.to_string().trim_start_matches("r#"),
        variant.to_string().trim_start_matches("r#")
    )
}

pub(crate) fn emit(value: &Value) -> TokenStream {
    let ValueKind::Enum(variants) = &value.kind else {
        return TokenStream::new();
    };
    let helpers = variants.iter().filter_map(|variant| {
        let VariantPayload::Struct(fields) = &variant.payload else {
            return None;
        };
        let helper = ident(&value.ident, &variant.ident);
        let members = fields.iter().map(|field| &field.ident);
        let types = fields.iter().map(|field| &field.ty);
        let payload = Value {
            ident: helper.clone(),
            facade: value.facade.clone(),
            wire_name: variant.wire_name.clone(),
            kind: ValueKind::Struct(fields.clone()),
        };
        let serialize = crate::serialize::emit(&payload);
        let canonical = crate::shape::emit(&payload);
        Some(quote! {
            #[doc(hidden)]
            struct #helper { #(#members: #types),* }
            #serialize
            #canonical
        })
    });
    quote!(#(#helpers)*)
}
