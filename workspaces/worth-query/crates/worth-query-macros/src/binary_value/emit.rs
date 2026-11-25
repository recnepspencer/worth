//! Lowers `#[derive(WorthBinaryValue)]` into a versioned `BinaryValueDelegate`,
//! `ApplicationScalarValueBinding`, `ApplicationReadableScalarValueBinding`,
//! `WorthQueryDefaultValueBinding`, and `WorthBinaryWireField` using two-layer
//! `const _: ()` hygiene.
use super::grammar::{BinaryValueDeclaration, StructShape};
use super::hygiene::HygienicNames;
use crate::facade_paths::declaration;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{DeriveInput, Index};

pub(super) fn binary_value(input: &DeriveInput, spec: &BinaryValueDeclaration) -> TokenStream {
    let (decl, vis, ident, binding) = (declaration(), &input.vis, &input.ident, &spec.binding);
    let aschema = quote!(#decl::application_schema);
    let typed = quote!(#decl::typed);
    let names = HygienicNames::new(input, spec);
    let HygienicNames {
        value_alias,
        binding_alias,
        delegate_struct,
        wire_bytes_const,
        field_aliases,
        call_validate,
        value_param,
        writer_param,
        bytes_param,
        admission_param,
        raw_param,
        rest_local,
        observed_local,
        decoded_local,
        reason_param,
        ..
    } = &names;

    let identity = &spec.identity;
    let version_u8 = spec.version;
    let version_u32 = u32::from(spec.version);
    let unit_expr = spec.unit.as_ref().map_or_else(
        || quote!(::core::option::Option::None),
        |u| quote!(::core::option::Option::Some(#aschema::ApplicationUnitIdentity::declared(#u))),
    );
    let frame_expr = spec.frame.as_ref().map_or_else(
        || quote!(::core::option::Option::None),
        |f| quote!(::core::option::Option::Some(#aschema::ApplicationFrameIdentity::declared(#f))),
    );

    let field_types = spec.field_types();
    let outer_field_aliases: Vec<TokenStream> = field_aliases
        .iter()
        .zip(field_types.iter())
        .map(|(alias, ty)| quote_spanned!(ty.span()=> type #alias = #ty;))
        .collect();

    let mut wire_bytes_expr = quote!(#aschema::worth_binary_checked_wire_bytes(1usize, 0usize));
    for (alias, ty) in field_aliases.iter().zip(field_types.iter()) {
        wire_bytes_expr = quote_spanned! {ty.span()=>
            #aschema::worth_binary_checked_wire_bytes(
                #wire_bytes_expr,
                <#alias as #aschema::WorthBinaryWireField>::WIRE_BYTES,
            )
        };
    }

    let validate_helper = spec.validate.as_ref().map(|func| {
        quote! {
            fn #call_validate() -> for<'a> fn(&'a #value_alias) -> ::core::result::Result<(), &'static str> {
                #func
            }
        }
    });
    let binding_validate_body = if spec.validate.is_some() {
        quote! {
            (#call_validate())(#value_param).map_err(|#reason_param| {
                #aschema::ApplicationValueValidationDenial::rejected(Self::IDENTITY, #reason_param)
            })
        }
    } else {
        quote! {
            let _ = #value_param;
            ::core::result::Result::Ok(())
        }
    };

    let encode_fields = emit_encode_fields(spec, &names, &aschema);
    let decode_fields = emit_decode_fields(spec, &names, &aschema);
    let construct_value = emit_construct(spec, &names);
    let inner_shadows = emit_inner_shadows(&names);

    let doc = format!("Versioned binary value binding for `{ident}`.");
    quote! {
        #[doc = #doc]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #vis struct #binding;

        const _: () = {
            type #value_alias = #ident;
            type #binding_alias = #binding;
            #(#outer_field_aliases)*
            #validate_helper
            const #wire_bytes_const: usize = #wire_bytes_expr;
            let _ = #wire_bytes_const;

            const _: () = {
                #inner_shadows

                struct #delegate_struct;

                impl #aschema::BinaryValueDelegate for #delegate_struct {
                    type Value = #value_alias;
                    const MAXIMUM_WIRE_BYTES: usize = #wire_bytes_const;

                    fn encode(
                        #value_param: &Self::Value,
                        #writer_param: &mut dyn #aschema::BinaryValueWriter,
                    ) -> ::core::result::Result<(), #aschema::ValueCodecDenial> {
                        let _ = #value_param;
                        #writer_param.write(&[#version_u8])?;
                        #(#encode_fields)*
                        ::core::result::Result::Ok(())
                    }

                    fn admit(
                        #bytes_param: &[u8],
                        #admission_param: &mut #aschema::ValueDecodeAdmission<'_>,
                    ) -> ::core::result::Result<Self::Value, #aschema::ValueCodecDenial> {
                        #admission_param.admit(#wire_bytes_const, 0)?;
                        let ::core::option::Option::Some((&#observed_local, #rest_local)) =
                            #bytes_param.split_first()
                        else {
                            return ::core::result::Result::Err(#aschema::ValueCodecDenial::Malformed);
                        };
                        if #observed_local != #version_u8 {
                            return ::core::result::Result::Err(
                                #aschema::ValueCodecDenial::UnsupportedVersion {
                                    expected: #version_u32,
                                    observed: ::core::primitive::u32::from(#observed_local),
                                },
                            );
                        }
                        if #bytes_param.len() != #wire_bytes_const {
                            return ::core::result::Result::Err(#aschema::ValueCodecDenial::Malformed);
                        }
                        let _ = #rest_local;
                        #(#decode_fields)*
                        ::core::result::Result::Ok(#construct_value)
                    }
                }

                impl #aschema::ApplicationScalarValueBinding for #binding_alias {
                    type Value = #value_alias;
                    type Unit = ();
                    type Decode = #aschema::ApplicationValueDecodeAvailable;
                    type Identity = #aschema::ApplicationValueIsNotIdentity;
                    type SignedAggregate = #aschema::ApplicationValueSignedAggregateUnavailable;
                    const IDENTITY_NAME: &'static str = #identity;
                    const SCALAR_FAMILY: #typed::ScalarAspectType =
                        #typed::ScalarAspectType::InlineBytes;
                    const DOMAIN_VALUE: bool = true;
                    const SUPPORTS_ADMITTED_DECODE: bool = true;
                    const UNIT: ::core::option::Option<#aschema::ApplicationUnitIdentity> =
                        #unit_expr;
                    const FRAME: ::core::option::Option<#aschema::ApplicationFrameIdentity> =
                        #frame_expr;

                    fn canonical_contract(
                    ) -> ::core::option::Option<#aschema::ApplicationCanonicalValueContract> {
                        ::core::option::Option::Some(
                            #aschema::ApplicationCanonicalValueContract::binary::<#delegate_struct>(),
                        )
                    }

                    fn validate(
                        #value_param: &Self::Value,
                    ) -> ::core::result::Result<(), #aschema::ApplicationValueValidationDenial> {
                        #binding_validate_body
                    }

                    fn encode(
                        #value_param: &Self::Value,
                    ) -> ::core::result::Result<
                        #aschema::ApplicationValue,
                        #aschema::ApplicationValueEncodeDenial,
                    > {
                        <Self as #aschema::ApplicationScalarValueBinding>::validate(#value_param)?;
                        #aschema::encode_application_binary_value::<#delegate_struct>(
                            #value_param,
                            <Self as #aschema::ApplicationScalarValueBinding>::IDENTITY,
                        )
                    }
                }

                impl #aschema::ApplicationReadableScalarValueBinding for #binding_alias {
                    fn decode(
                        #raw_param: &#aschema::ApplicationValue,
                    ) -> ::core::result::Result<
                        Self::Value,
                        #aschema::ApplicationValueDecodeDenial,
                    > {
                        let #decoded_local =
                            #aschema::decode_application_binary_value::<#delegate_struct>(
                                #raw_param,
                                <Self as #aschema::ApplicationScalarValueBinding>::IDENTITY,
                                ::core::option::Option::None,
                            )?;
                        <Self as #aschema::ApplicationScalarValueBinding>::validate(
                            &#decoded_local,
                        )?;
                        ::core::result::Result::Ok(#decoded_local)
                    }

                    fn decode_admitted(
                        #raw_param: &#aschema::ApplicationValue,
                        #admission_param: &mut #aschema::ValueDecodeAdmission<'_>,
                    ) -> ::core::result::Result<
                        Self::Value,
                        #aschema::ApplicationValueDecodeDenial,
                    > {
                        let #decoded_local =
                            #aschema::decode_application_binary_value::<#delegate_struct>(
                                #raw_param,
                                <Self as #aschema::ApplicationScalarValueBinding>::IDENTITY,
                                ::core::option::Option::Some(#admission_param),
                            )?;
                        <Self as #aschema::ApplicationScalarValueBinding>::validate(
                            &#decoded_local,
                        )?;
                        ::core::result::Result::Ok(#decoded_local)
                    }
                }

                impl #aschema::WorthQueryDefaultValueBinding for #value_alias {
                    type Binding = #binding_alias;
                }
            };
        };
    }
}

fn emit_encode_fields(
    spec: &BinaryValueDeclaration,
    names: &HygienicNames,
    aschema: &TokenStream,
) -> Vec<TokenStream> {
    let (value_param, writer_param) = (&names.value_param, &names.writer_param);
    match &spec.shape {
        StructShape::Named(fields) => fields
            .iter()
            .zip(names.field_aliases.iter())
            .map(|(field, alias)| {
                let member = &field.ident;
                quote_spanned! {field.ty.span()=>
                    <#alias as #aschema::WorthBinaryWireField>::encode_binary_field(
                        &#value_param.#member,
                        #writer_param,
                    )?;
                }
            })
            .collect(),
        StructShape::Tuple(types) => types
            .iter()
            .enumerate()
            .zip(names.field_aliases.iter())
            .map(|((idx, ty), alias)| {
                let member = Index::from(idx);
                quote_spanned! {ty.span()=>
                    <#alias as #aschema::WorthBinaryWireField>::encode_binary_field(
                        &#value_param.#member,
                        #writer_param,
                    )?;
                }
            })
            .collect(),
        StructShape::Unit => Vec::new(),
    }
}

fn emit_decode_fields(
    spec: &BinaryValueDeclaration,
    names: &HygienicNames,
    aschema: &TokenStream,
) -> Vec<TokenStream> {
    let (rest, slice, admission) = (
        &names.rest_local,
        &names.slice_local,
        &names.admission_param,
    );
    let types = spec.field_types();
    types
        .iter()
        .zip(names.field_aliases.iter())
        .zip(names.field_locals.iter())
        .map(|((ty, alias), local)| {
            quote_spanned! {ty.span()=>
                let (#slice, #rest) = #rest.split_at(
                    <#alias as #aschema::WorthBinaryWireField>::WIRE_BYTES,
                );
                let #local = <#alias as #aschema::WorthBinaryWireField>::decode_binary_field(
                    #slice,
                    #admission,
                )?;
            }
        })
        .collect()
}

fn emit_construct(spec: &BinaryValueDeclaration, names: &HygienicNames) -> TokenStream {
    let value_alias = &names.value_alias;
    match &spec.shape {
        StructShape::Named(fields) => {
            let inits = fields
                .iter()
                .zip(names.field_locals.iter())
                .map(|(field, local)| {
                    let member = &field.ident;
                    quote!(#member: #local)
                });
            quote!(#value_alias { #(#inits),* })
        }
        StructShape::Tuple(_) => {
            let inits = names.field_locals.iter().enumerate().map(|(i, local)| {
                let idx = Index::from(i);
                quote!(#idx: #local)
            });
            quote!(#value_alias { #(#inits),* })
        }
        StructShape::Unit => quote!(#value_alias {}),
    }
}

fn emit_inner_shadows(names: &HygienicNames) -> TokenStream {
    let mut idents = vec![
        &names.value_param,
        &names.writer_param,
        &names.bytes_param,
        &names.admission_param,
        &names.raw_param,
        &names.rest_local,
        &names.observed_local,
        &names.slice_local,
        &names.decoded_local,
        &names.reason_param,
    ];
    idents.extend(names.field_locals.iter());
    quote! {
        #(
            #[allow(dead_code, non_snake_case)]
            fn #idents() {}
        )*
    }
}
