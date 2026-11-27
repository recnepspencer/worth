//! `#[derive(WorthQueryVocabulary)]`: a closed, code-identified enum and its
//! identity scalar binding.
//!
//! Every variant is a unit variant with an explicit integer code
//! (`Variant = 3`); codes are the durable wire values and must be distinct.
//! Enum keys: `identity = ".."` (required, the binding's portable identity),
//! `binding = Name` (default `<Enum>Binding`) and `bindings = Codecs` (also
//! registers the enum as a `Scalar` of that fragment binding set).
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeSet;
use syn::spanned::Spanned;
use syn::{DeriveInput, Expr, ExprLit, Fields, Ident, Lit, LitStr, Path};

use crate::facade_paths::{declaration, fragment};
use crate::grammar::{enum_variants, unknown_key, value, visit_keys, Once};

pub(crate) const DERIVE: &str = "WorthQueryVocabulary";
const KEYS: &[&str] = &["identity", "binding", "bindings"];

struct Vocabulary {
    identity: LitStr,
    binding: Ident,
    bindings: Option<Path>,
    variants: Vec<(Ident, u64)>,
}

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let vocabulary = parse(input)?;
    Ok(emit(input, &vocabulary))
}

fn parse(input: &DeriveInput) -> syn::Result<Vocabulary> {
    let mut identity = Once::<LitStr>::new("identity");
    let mut binding = Once::<Ident>::new("binding");
    let mut bindings = Once::<Path>::new("bindings");
    visit_keys(&input.attrs, |key, meta| match key {
        "identity" => identity.set(meta, value(meta)?),
        "binding" => binding.set(meta, value(meta)?),
        "bindings" => bindings.set(meta, value(meta)?),
        _ => Err(unknown_key(meta, DERIVE, KEYS)),
    })?;
    let mut codes = BTreeSet::new();
    let mut variants = Vec::new();
    for variant in enum_variants(input, DERIVE, "a vocabulary value")? {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new(
                variant.fields.span(),
                "a vocabulary value is a unit variant",
            ));
        }
        let code = match &variant.discriminant {
            Some((
                _,
                Expr::Lit(ExprLit {
                    lit: Lit::Int(code),
                    ..
                }),
            )) => code.base10_parse()?,
            _ => {
                return Err(syn::Error::new(
                    variant.ident.span(),
                    "write the durable code of every vocabulary value, such as `Variant = 1`",
                ))
            }
        };
        if !codes.insert(code) {
            return Err(syn::Error::new(
                variant.ident.span(),
                format!("vocabulary code {code} is used twice"),
            ));
        }
        variants.push((variant.ident.clone(), code));
    }
    Ok(Vocabulary {
        identity: identity.required(input.ident.span(), DERIVE)?,
        binding: binding
            .optional()
            .unwrap_or_else(|| format_ident!("{}Binding", input.ident)),
        bindings: bindings.optional(),
        variants,
    })
}

fn emit(input: &DeriveInput, vocabulary: &Vocabulary) -> TokenStream {
    let (decl, vis, value_type) = (declaration(), &input.vis, &input.ident);
    let schema = quote!(#decl::application_schema);
    let Vocabulary {
        identity,
        binding,
        bindings,
        variants,
    } = vocabulary;
    let names = variants
        .iter()
        .map(|(variant, _)| variant)
        .collect::<Vec<_>>();
    let codes = variants.iter().map(|(_, code)| code).collect::<Vec<_>>();
    let scalar = bindings.as_ref().map(|bindings| {
        let fragment = fragment();
        quote! {
            impl #fragment::Scalar<#value_type> for #bindings {
                type Binding = #binding;
            }
        }
    });
    quote! {
        impl ::core::convert::From<#value_type> for u64 {
            fn from(value: #value_type) -> u64 {
                match value { #(#value_type::#names => #codes,)* }
            }
        }

        impl ::core::convert::TryFrom<u64> for #value_type {
            type Error = &'static str;

            fn try_from(code: u64) -> ::core::result::Result<Self, Self::Error> {
                match code {
                    #(#codes => ::core::result::Result::Ok(Self::#names),)*
                    _ => ::core::result::Result::Err("unknown vocabulary code"),
                }
            }
        }

        impl #schema::ApplicationValueVocabulary for #value_type {
            const VARIANTS: &'static [Self] = &[#(Self::#names),*];

            fn code(self) -> u64 {
                u64::from(self)
            }

            fn from_code(code: u64) -> ::core::option::Option<Self> {
                <Self as ::core::convert::TryFrom<u64>>::try_from(code).ok()
            }
        }

        /// The identity scalar binding of the vocabulary's durable codes.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #vis struct #binding;

        impl #binding {
            fn decode_code(
                code: u64,
            ) -> ::core::result::Result<#value_type, #schema::ApplicationValueDecodeDenial> {
                <#value_type as #schema::ApplicationValueVocabulary>::from_code(code).ok_or(
                    #schema::ApplicationValueDecodeDenial::CodecRejected {
                        binding_identity:
                            <Self as #schema::ApplicationScalarValueBinding>::IDENTITY,
                    },
                )
            }
        }

        impl #schema::ApplicationScalarValueBinding for #binding {
            type Value = #value_type;
            type Unit = ();
            type Decode = #schema::ApplicationValueDecodeAvailable;
            type Identity = #schema::ApplicationValueIsIdentity;
            type SignedAggregate = #schema::ApplicationValueSignedAggregateUnavailable;
            const IDENTITY_NAME: &'static str = #identity;
            const SCALAR_FAMILY: #decl::typed::ScalarAspectType =
                <#schema::U64ApplicationValueBinding as #schema::ApplicationScalarValueBinding>::SCALAR_FAMILY;
            const SUPPORTS_ADMITTED_DECODE: bool = true;
            const NATIVE_INPUT_ADMISSION: ::core::option::Option<#schema::ApplicationNativeInputDecodeProfile> =
                <#schema::U64ApplicationValueBinding as #schema::ApplicationScalarValueBinding>::NATIVE_INPUT_ADMISSION;

            fn validate(
                value: &#value_type,
            ) -> ::core::result::Result<(), #schema::ApplicationValueValidationDenial> {
                <#schema::U64ApplicationValueBinding as #schema::ApplicationScalarValueBinding>::validate(
                    &u64::from(*value),
                )
            }

            fn encode(
                value: &#value_type,
            ) -> ::core::result::Result<#schema::ApplicationValue, #schema::ApplicationValueEncodeDenial> {
                <#schema::U64ApplicationValueBinding as #schema::ApplicationScalarValueBinding>::encode(
                    &u64::from(*value),
                )
            }
        }

        impl #schema::ApplicationReadableScalarValueBinding for #binding {
            fn decode(
                value: &#schema::ApplicationValue,
            ) -> ::core::result::Result<#value_type, #schema::ApplicationValueDecodeDenial> {
                <#schema::U64ApplicationValueBinding as #schema::ApplicationReadableScalarValueBinding>::decode(value)
                    .and_then(Self::decode_code)
            }

            fn decode_admitted(
                value: &#schema::ApplicationValue,
                work: &mut #schema::ValueDecodeAdmission<'_>,
            ) -> ::core::result::Result<#value_type, #schema::ApplicationValueDecodeDenial> {
                <#schema::U64ApplicationValueBinding as #schema::ApplicationReadableScalarValueBinding>::decode_admitted(value, work)
                    .and_then(Self::decode_code)
            }
        }

        impl #schema::ApplicationIdentityScalarValueBinding for #binding {}

        #scalar
    }
}
