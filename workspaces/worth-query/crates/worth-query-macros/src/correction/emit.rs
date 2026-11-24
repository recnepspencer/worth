//! Lowers `#[derive(WorthQueryCorrection)]` to one standard recorded-inverse
//! `ApplicationCorrectionBinding` implementation.
use super::grammar::{CorrectionDeclaration, CorrectionIdentity, PreImageDemandSpec};
use crate::facade_paths::declaration;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub(super) fn correction(input: &DeriveInput, spec: &CorrectionDeclaration) -> TokenStream {
    let (decl, ident) = (declaration(), &input.ident);
    let (schema, source, inverse, disclosure) =
        (&spec.schema, &spec.source, &spec.inverse, &spec.disclosure);
    let aftermath = quote!(#decl::application_aftermath);
    let op = quote!(#decl::application_operation);
    let aq = quote!(#decl::application_query);
    let aschema = quote!(#decl::application_schema);

    let identity = identity_expr(&decl, spec);
    let disclosure_params = disclosure_params_expr(&aq, spec);
    let source_definition = source_definition_expr(&aschema, &op, spec);
    let source_body = source_body_method(&aschema, &op, spec);
    let demand_expr = demand_expr(&aftermath, spec);
    let derive_fn = &spec.derive_fn;
    let inverse_key = inverse_key_expr(&decl, &aschema, spec);
    let visit = visit_expr(spec);

    quote! {
        impl #aftermath::ApplicationCorrectionBinding<#schema> for #ident {
            const IDENTITY: &'static str = #identity;
            type Source = #source;
            type InverseIntent = #inverse;
            type Disclosure = #disclosure;

            fn recover_source_input(
                __worth_correction_bytes: &[u8],
                __worth_correction_expected: #aschema::CanonicalDigestId,
                __worth_correction_bounds: #aschema::ValueDecodeBounds,
            ) -> ::core::result::Result<
                #op::ApplicationEncodedInput<
                    <Self::Source as #op::ApplicationMutationBinding<#schema>>::InputBinding,
                >,
                #op::ApplicationRetainedInputDenial,
            > {
                #op::ApplicationEncodedInput::decode_retained(
                    __worth_correction_bytes,
                    __worth_correction_expected,
                    __worth_correction_bounds,
                )
            }

            fn recover_inverse_input(
                __worth_correction_bytes: &[u8],
                __worth_correction_expected: #aschema::CanonicalDigestId,
                __worth_correction_bounds: #aschema::ValueDecodeBounds,
            ) -> ::core::result::Result<
                #op::ApplicationEncodedInput<
                    <#aftermath::ApplicationInverseBinding<#schema, Self> as #op::ApplicationMutationBinding<#schema>>::InputBinding,
                >,
                #op::ApplicationRetainedInputDenial,
            > {
                #op::ApplicationEncodedInput::decode_retained(
                    __worth_correction_bytes,
                    __worth_correction_expected,
                    __worth_correction_bounds,
                )
            }

            fn disclosure_parameters(
                __worth_correction_original: &<Self::Source as #op::ApplicationMutationBinding<#schema>>::Input,
            ) -> ::core::result::Result<
                #aq::ApplicationQueryParameterSet<
                    <Self::Disclosure as #aq::ApplicationQueryBinding<#schema>>::Query,
                >,
                #aftermath::ApplicationCorrectionDerivationDenial,
            > {
                let _ = __worth_correction_original;
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
                __worth_correction_original: &<Self::Source as #op::ApplicationMutationBinding<#schema>>::Input,
                __worth_correction_material: #aftermath::ApplicationCorrectionSourceMaterial<'_, #schema>,
            ) -> ::core::result::Result<
                Self::InverseIntent,
                #aftermath::ApplicationCorrectionDerivationDenial,
            > {
                let #aftermath::ApplicationCorrectionSourceMaterial::RecordedInverse(
                    __worth_correction_preimage,
                ) = __worth_correction_material
                else {
                    return ::core::result::Result::Err(
                        #aftermath::ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning,
                    );
                };
                #derive_fn(__worth_correction_original, __worth_correction_preimage)
            }

            fn inverse_key(
                __worth_correction_key: &str,
            ) -> ::core::result::Result<
                <#aftermath::ApplicationInverseBinding<#schema, Self> as #op::ApplicationMutationBinding<#schema>>::IdempotencyKey,
                #aschema::ApplicationValueEncodeDenial,
            > {
                #inverse_key
            }

            fn visit<__WorthCorrectionVisitor: #aftermath::ApplicationCorrectionVisitor<#schema>>(
                __worth_correction_dispatch: #aftermath::ApplicationCorrectionDispatch<
                    #schema,
                    Self,
                    __WorthCorrectionVisitor,
                >,
            ) -> __WorthCorrectionVisitor::Output {
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

fn disclosure_params_expr(aq: &TokenStream, spec: &CorrectionDeclaration) -> TokenStream {
    spec.disclosure_parameters.as_ref().map_or_else(
        || quote!(::core::result::Result::Ok(#aq::ApplicationQueryParameterSet::new())),
        |func| quote!(#func(__worth_correction_original)),
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
) -> TokenStream {
    spec.inverse_key.as_ref().map_or_else(
        || {
            quote! {
                __worth_correction_key.parse().map_err(|_| {
                    #aschema::ApplicationValueEncodeDenial::CodecRejected {
                        binding_identity: #decl::portable_identity::WorthQueryPortableTypeIdentity::declared(
                            Self::IDENTITY,
                        ),
                        reason: "invalid inverse idempotency key",
                    }
                })
            }
        },
        |func| quote!(#func(__worth_correction_key)),
    )
}

fn visit_expr(spec: &CorrectionDeclaration) -> TokenStream {
    spec.capability.as_ref().map_or_else(
        || quote!(__worth_correction_dispatch.conventional()),
        |capability| quote!(__worth_correction_dispatch.capability::<#capability>()),
    )
}
