//! Lowers `#[derive(WorthQueryCorrection)]` to one standard recorded-inverse
//! `ApplicationCorrectionBinding` implementation.
//!
//! Hygiene guarantees:
//! - Outer `const _: ()` resolves every caller type (`schema`, `source`,
//!   `inverse`, `disclosure`, `capability`) and every caller callback
//!   (`derive`, `inverse_key`, `disclosure_parameters`, `source_definition`,
//!   `source_body`, `demand`) with zero generated parameters or visitor
//!   generics in scope.
//! - Inner `const _: ()` shadows every generated parameter/local name with a
//!   local dummy `fn` item so no outer module `const` or unit `struct` with the
//!   same name can trigger `E0530` or capture a pattern binding.
use super::grammar::{CorrectionDeclaration, CorrectionIdentity};
use super::hygiene::HygienicNames;
use crate::facade_paths::declaration;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

mod callbacks;
use callbacks::outer_callback_helpers;

pub(super) fn correction(input: &DeriveInput, spec: &CorrectionDeclaration) -> TokenStream {
    let (decl, ident) = (declaration(), &input.ident);
    let (schema, source, inverse, disclosure) =
        (&spec.schema, &spec.source, &spec.inverse, &spec.disclosure);
    let aftermath = quote!(#decl::application_aftermath);
    let op = quote!(#decl::application_operation);
    let aq = quote!(#decl::application_query);
    let aschema = quote!(#decl::application_schema);
    let names = HygienicNames::new(input, spec);
    let HygienicNames {
        schema_alias,
        source_alias,
        inverse_alias,
        disclosure_alias,
        cap_alias,
        program_alias,
        inverse_action_alias,
        source_action_alias,
        identity_const,
        call_disclosure,
        call_source_def,
        call_source_body,
        call_demand,
        call_derive,
        call_inverse_key,
        call_reapply_key,
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

    let cap_alias_def = spec
        .capability
        .as_ref()
        .map(|cap| quote!(type #cap_alias = #cap;));
    let authored_aliases = spec.authored_actions.as_ref().map(|actions| {
        let program = &actions.program;
        let inverse_action = &actions.inverse_action;
        let source_action_def = actions
            .source_action
            .as_ref()
            .map(|src| quote!(type #source_action_alias = #src;));
        quote! {
            type #program_alias = #program;
            type #inverse_action_alias = #inverse_action;
            #source_action_def
        }
    });
    let identity = identity_expr(&decl, spec);
    let outer_helpers = outer_callback_helpers(&aftermath, &op, &aq, &aschema, ident, &names, spec);
    let disclosure_body = if spec.disclosure_parameters.is_some() {
        quote!((#call_disclosure())(#original))
    } else {
        quote!(::core::result::Result::Ok(#aq::ApplicationQueryParameterSet::new()))
    };
    let source_def_body = if spec.source_definition.is_some() {
        quote!((#call_source_def())())
    } else {
        quote! {
            #aschema::ApplicationOperationRef::<
                #schema_alias,
                <Self::Source as #op::ApplicationMutationBinding<#schema_alias>>::Operation,
                <Self::Source as #op::ApplicationMutationBinding<#schema_alias>>::Input,
            >::from_declaration()
            .definition()
            .no_external_effect()
        }
    };
    let source_body_fn = spec.source_body.as_ref().map(|_| {
        quote! {
            fn source_body() -> ::core::option::Option<
                #aschema::ApplicationOperationBody<
                    #schema_alias,
                    <Self::Source as #op::ApplicationMutationBinding<#schema_alias>>::Operation,
                    <Self::Source as #op::ApplicationMutationBinding<#schema_alias>>::Input,
                >,
            > {
                ::core::option::Option::Some((#call_source_body())())
            }
        }
    });
    let inverse_key_body = if spec.inverse_key.is_some() {
        quote!((#call_inverse_key())(#key))
    } else {
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
    };
    let visit_body = spec.capability.as_ref().map_or_else(
        || quote!(#dispatch.conventional()),
        |_| quote!(#dispatch.capability::<#cap_alias>()),
    );
    let reapply_methods = spec.reapply.then(|| {
        let reapply_key_body = if spec.reapplication_key.is_some() {
            quote!((#call_reapply_key())(#key))
        } else {
            quote! {
                #key.parse().map_err(|_| {
                    #aschema::ApplicationValueEncodeDenial::CodecRejected {
                        binding_identity: #decl::portable_identity::WorthQueryPortableTypeIdentity::declared(
                            Self::IDENTITY,
                        ),
                        reason: "invalid reapplication idempotency key",
                    }
                })
            }
        };
        quote! {
            fn visit_reapplication<#visitor: #aftermath::ApplicationReapplicationVisitor<#schema_alias>>(
                #dispatch: #aftermath::ApplicationReapplicationDispatch<#schema_alias, Self, #visitor>,
            ) -> #visitor::Output {
                #visit_body
            }

            fn reapplication_key(
                #key: &str,
            ) -> ::core::result::Result<
                <Self::Source as #op::ApplicationMutationBinding<#schema_alias>>::IdempotencyKey,
                #aschema::ApplicationValueEncodeDenial,
            > {
                #reapply_key_body
            }
        }
    });
    let authored_impls = spec.authored_actions.as_ref().map(|actions| {
        let reapply_impl = actions.source_action.as_ref().map(|_| {
            quote! {
                impl #aftermath::ApplicationAuthoredBodyReapplication<#schema_alias, #program_alias>
                    for #ident
                {
                    type SourceAction = #source_action_alias;
                }
            }
        });
        quote! {
            impl #aftermath::ApplicationAuthoredBodyCorrection<#schema_alias, #program_alias>
                for #ident
            {
                type InverseAction = #inverse_action_alias;
            }
            #reapply_impl
        }
    });

    quote! {
        const _: () = {
            type #schema_alias = #schema;
            type #source_alias = #source;
            type #inverse_alias = #inverse;
            type #disclosure_alias = #disclosure;
            #cap_alias_def
            #authored_aliases
            const #identity_const: &'static str = #identity;
            #outer_helpers

            const _: () = {
                #[allow(dead_code, non_snake_case)]
                fn #bytes() {}
                #[allow(dead_code, non_snake_case)]
                fn #expected() {}
                #[allow(dead_code, non_snake_case)]
                fn #bounds() {}
                #[allow(dead_code, non_snake_case)]
                fn #original() {}
                #[allow(dead_code, non_snake_case)]
                fn #material() {}
                #[allow(dead_code, non_snake_case)]
                fn #preimage() {}
                #[allow(dead_code, non_snake_case)]
                fn #key() {}
                #[allow(dead_code, non_snake_case)]
                fn #dispatch() {}

                impl #aftermath::ApplicationCorrectionBinding<#schema_alias> for #ident {
                    const IDENTITY: &'static str = #identity_const;
                    type Source = #source_alias;
                    type InverseIntent = #inverse_alias;
                    type Disclosure = #disclosure_alias;

                    fn recover_source_input(
                        #bytes: &[u8],
                        #expected: #aschema::CanonicalDigestId,
                        #bounds: #aschema::ValueDecodeBounds,
                    ) -> ::core::result::Result<
                        #op::ApplicationEncodedInput<
                            <Self::Source as #op::ApplicationMutationBinding<#schema_alias>>::InputBinding,
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
                            <#aftermath::ApplicationInverseBinding<#schema_alias, Self> as #op::ApplicationMutationBinding<#schema_alias>>::InputBinding,
                        >,
                        #op::ApplicationRetainedInputDenial,
                    > {
                        #op::ApplicationEncodedInput::decode_retained(#bytes, #expected, #bounds)
                    }

                    fn disclosure_parameters(
                        #original: &<Self::Source as #op::ApplicationMutationBinding<#schema_alias>>::Input,
                    ) -> ::core::result::Result<
                        #aq::ApplicationQueryParameterSet<
                            <Self::Disclosure as #aq::ApplicationQueryBinding<#schema_alias>>::Query,
                        >,
                        #aftermath::ApplicationCorrectionDerivationDenial,
                    > {
                        let _ = #original;
                        #disclosure_body
                    }

                    fn source_definition() -> #aschema::ApplicationOperationDefinitionBuilder<
                        #schema_alias,
                        <Self::Source as #op::ApplicationMutationBinding<#schema_alias>>::Operation,
                        <Self::Source as #op::ApplicationMutationBinding<#schema_alias>>::Input,
                        true,
                        false,
                    > {
                        #source_def_body
                    }

                    #source_body_fn

                    fn mechanism() -> #aftermath::DeclaredCorrectionMechanism<#schema_alias> {
                        #aftermath::DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
                            #call_demand(),
                        )
                    }

                    fn derive_correction(
                        #original: &<Self::Source as #op::ApplicationMutationBinding<#schema_alias>>::Input,
                        #material: #aftermath::ApplicationCorrectionSourceMaterial<'_, #schema_alias>,
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
                        (#call_derive())(#original, #preimage)
                    }

                    fn inverse_key(
                        #key: &str,
                    ) -> ::core::result::Result<
                        <#aftermath::ApplicationInverseBinding<#schema_alias, Self> as #op::ApplicationMutationBinding<#schema_alias>>::IdempotencyKey,
                        #aschema::ApplicationValueEncodeDenial,
                    > {
                        #inverse_key_body
                    }

                    fn visit<#visitor: #aftermath::ApplicationCorrectionVisitor<#schema_alias>>(
                        #dispatch: #aftermath::ApplicationCorrectionDispatch<#schema_alias, Self, #visitor>,
                    ) -> #visitor::Output {
                        #visit_body
                    }

                    #reapply_methods
                }

                #authored_impls
            };
        };
    }
}

fn identity_expr(decl: &TokenStream, spec: &CorrectionDeclaration) -> TokenStream {
    match &spec.identity {
        CorrectionIdentity::Namespace(ns) => quote!(#decl::worth_query_identity!(#ns, binding)),
        CorrectionIdentity::Explicit(lit) => quote!(#lit),
    }
}
