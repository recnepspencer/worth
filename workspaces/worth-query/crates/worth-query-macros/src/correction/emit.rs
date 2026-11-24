//! Lowers `#[derive(WorthQueryCorrection)]` to one standard recorded-inverse
//! `ApplicationCorrectionBinding` implementation using def-site (`mixed_site`)
//! hygiene for all generated locals, parameters, and visitor generics.
use super::grammar::{CorrectionDeclaration, CorrectionIdentity, PreImageDemandSpec};
use crate::facade_paths::declaration;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Ident};

struct HygienicNames {
    bytes: Ident,
    expected: Ident,
    bounds: Ident,
    original: Ident,
    material: Ident,
    preimage: Ident,
    key: Ident,
    dispatch: Ident,
    visitor: Ident,
}

impl HygienicNames {
    fn new() -> Self {
        let span = Span::mixed_site();
        Self {
            bytes: Ident::new("__worth_correction_bytes", span),
            expected: Ident::new("__worth_correction_expected", span),
            bounds: Ident::new("__worth_correction_bounds", span),
            original: Ident::new("__worth_correction_original", span),
            material: Ident::new("__worth_correction_material", span),
            preimage: Ident::new("__worth_correction_preimage", span),
            key: Ident::new("__worth_correction_key", span),
            dispatch: Ident::new("__worth_correction_dispatch", span),
            visitor: Ident::new("__WorthCorrectionVisitor", span),
        }
    }
}

pub(super) fn correction(input: &DeriveInput, spec: &CorrectionDeclaration) -> TokenStream {
    let (decl, ident) = (declaration(), &input.ident);
    let (schema, source, inverse, disclosure) =
        (&spec.schema, &spec.source, &spec.inverse, &spec.disclosure);
    let aftermath = quote!(#decl::application_aftermath);
    let op = quote!(#decl::application_operation);
    let aq = quote!(#decl::application_query);
    let aschema = quote!(#decl::application_schema);
    let names = HygienicNames::new();
    let HygienicNames {
        bytes,
        expected,
        bounds,
        original,
        material,
        preimage,
        key,
        dispatch,
        visitor,
    } = &names;

    let identity = identity_expr(&decl, spec);
    let disclosure_params = disclosure_params_expr(&aq, spec, original);
    let source_definition = source_definition_expr(&aschema, &op, spec);
    let source_body = source_body_method(&aschema, &op, spec);
    let demand_expr = demand_expr(&aftermath, spec);
    let derive_fn = &spec.derive_fn;
    let inverse_key = inverse_key_expr(&decl, &aschema, spec, key);
    let visit = visit_expr(spec, dispatch);

    quote! {
        impl #aftermath::ApplicationCorrectionBinding<#schema> for #ident {
            const IDENTITY: &'static str = #identity;
            type Source = #source;
            type InverseIntent = #inverse;
            type Disclosure = #disclosure;

            fn recover_source_input(
                #bytes: &[u8],
                #expected: #aschema::CanonicalDigestId,
                #bounds: #aschema::ValueDecodeBounds,
            ) -> ::core::result::Result<
                #op::ApplicationEncodedInput<
                    <Self::Source as #op::ApplicationMutationBinding<#schema>>::InputBinding,
                >,
                #op::ApplicationRetainedInputDenial,
            > {
                #op::ApplicationEncodedInput::decode_retained(#bytes, #expected, #bounds)
            }

            fn recover_inverse_input(
                #bytes: &[u8],
                #expected: #aschema::CanonicalDigestId,
                #bounds: #aschema::ValueDecodeBounds,
            ) -> ::core::result::Result<
                #op::ApplicationEncodedInput<
                    <#aftermath::ApplicationInverseBinding<#schema, Self> as #op::ApplicationMutationBinding<#schema>>::InputBinding,
                >,
                #op::ApplicationRetainedInputDenial,
            > {
                #op::ApplicationEncodedInput::decode_retained(#bytes, #expected, #bounds)
            }

            fn disclosure_parameters(
                #original: &<Self::Source as #op::ApplicationMutationBinding<#schema>>::Input,
            ) -> ::core::result::Result<
                #aq::ApplicationQueryParameterSet<
                    <Self::Disclosure as #aq::ApplicationQueryBinding<#schema>>::Query,
                >,
                #aftermath::ApplicationCorrectionDerivationDenial,
            > {
                let _ = #original;
                #disclosure_params
            }

            fn source_definition() -> #aschema::ApplicationOperationDefinitionBuilder<
                #schema,
                <Self::Source as #op::ApplicationMutationBinding<#schema>>::Operation,
                <Self::Source as #op::ApplicationMutationBinding<#schema>>::Input,
                true,
                false,
            > {
                #source_definition
            }

            #source_body

            fn mechanism() -> #aftermath::DeclaredCorrectionMechanism<#schema> {
                #aftermath::DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
                    #demand_expr,
                )
            }

            fn derive_correction(
                #original: &<Self::Source as #op::ApplicationMutationBinding<#schema>>::Input,
                #material: #aftermath::ApplicationCorrectionSourceMaterial<'_, #schema>,
            ) -> ::core::result::Result<
                Self::InverseIntent,
                #aftermath::ApplicationCorrectionDerivationDenial,
            > {
                let #aftermath::ApplicationCorrectionSourceMaterial::RecordedInverse(#preimage) =
                    #material
                else {
                    return ::core::result::Result::Err(
                        #aftermath::ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning,
                    );
                };
                #derive_fn(#original, #preimage)
            }

            fn inverse_key(
                #key: &str,
            ) -> ::core::result::Result<
                <#aftermath::ApplicationInverseBinding<#schema, Self> as #op::ApplicationMutationBinding<#schema>>::IdempotencyKey,
                #aschema::ApplicationValueEncodeDenial,
            > {
                #inverse_key
            }

            fn visit<#visitor: #aftermath::ApplicationCorrectionVisitor<#schema>>(
                #dispatch: #aftermath::ApplicationCorrectionDispatch<#schema, Self, #visitor>,
            ) -> #visitor::Output {
                #visit
            }
        }
    }
}

fn identity_expr(decl: &TokenStream, spec: &CorrectionDeclaration) -> TokenStream {
    match &spec.identity {
        CorrectionIdentity::Namespace(ns) => quote!(#decl::worth_query_identity!(#ns, binding)),
        CorrectionIdentity::Explicit(lit) => quote!(#lit),
    }
}

fn disclosure_params_expr(
    aq: &TokenStream,
    spec: &CorrectionDeclaration,
    original: &Ident,
) -> TokenStream {
    spec.disclosure_parameters.as_ref().map_or_else(
        || quote!(::core::result::Result::Ok(#aq::ApplicationQueryParameterSet::new())),
        |func| quote!(#func(#original)),
    )
}

fn source_definition_expr(
    aschema: &TokenStream,
    op: &TokenStream,
    spec: &CorrectionDeclaration,
) -> TokenStream {
    let schema = &spec.schema;
    spec.source_definition.as_ref().map_or_else(
        || {
            quote! {
                #aschema::ApplicationOperationRef::<
                    #schema,
                    <Self::Source as #op::ApplicationMutationBinding<#schema>>::Operation,
                    <Self::Source as #op::ApplicationMutationBinding<#schema>>::Input,
                >::from_declaration()
                .definition()
                .no_external_effect()
            }
        },
        |func| quote!(#func()),
    )
}

fn source_body_method(
    aschema: &TokenStream,
    op: &TokenStream,
    spec: &CorrectionDeclaration,
) -> Option<TokenStream> {
    let schema = &spec.schema;
    spec.source_body.as_ref().map(|func| {
        quote! {
            fn source_body() -> ::core::option::Option<
                #aschema::ApplicationOperationBody<
                    #schema,
                    <Self::Source as #op::ApplicationMutationBinding<#schema>>::Operation,
                    <Self::Source as #op::ApplicationMutationBinding<#schema>>::Input,
                >,
            > {
                ::core::option::Option::Some(#func())
            }
        }
    })
}

fn demand_expr(aftermath: &TokenStream, spec: &CorrectionDeclaration) -> TokenStream {
    match &spec.demand {
        PreImageDemandSpec::Function(func) => quote!(#func()),
        PreImageDemandSpec::Fields { fields, bytes } => quote! {
            #aftermath::DeclaredPreImageDemand::new(
                [#(#aftermath::DeclaredPreImageLocus::from_field(#fields)),*],
                #bytes,
            )
        },
    }
}

fn inverse_key_expr(
    decl: &TokenStream,
    aschema: &TokenStream,
    spec: &CorrectionDeclaration,
    key: &Ident,
) -> TokenStream {
    spec.inverse_key.as_ref().map_or_else(
        || {
            quote! {
                #key.parse().map_err(|_| {
                    #aschema::ApplicationValueEncodeDenial::CodecRejected {
                        binding_identity: #decl::portable_identity::WorthQueryPortableTypeIdentity::declared(
                            Self::IDENTITY,
                        ),
                        reason: "invalid inverse idempotency key",
                    }
                })
            }
        },
        |func| quote!(#func(#key)),
    )
}

fn visit_expr(spec: &CorrectionDeclaration, dispatch: &Ident) -> TokenStream {
    spec.capability.as_ref().map_or_else(
        || quote!(#dispatch.conventional()),
        |capability| quote!(#dispatch.capability::<#capability>()),
    )
}
