//! Outer-block typed callback thunks for `#[derive(WorthQueryCorrection)]`,
//! evaluated outside the inner local-shadowing scope.
use super::super::grammar::{CorrectionDeclaration, PreImageDemandSpec};
use super::super::hygiene::HygienicNames;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(super) fn outer_callback_helpers(
    aftermath: &TokenStream,
    op: &TokenStream,
    aq: &TokenStream,
    aschema: &TokenStream,
    ident: &Ident,
    names: &HygienicNames,
    spec: &CorrectionDeclaration,
) -> TokenStream {
    let HygienicNames {
        schema_alias,
        source_alias,
        inverse_alias,
        disclosure_alias,
        call_disclosure,
        call_source_def,
        call_source_body,
        call_demand,
        call_derive,
        call_inverse_key,
        call_reapply_key,
        ..
    } = names;
    let derive_fn = &spec.derive_fn;
    let disclosure_helper = spec.disclosure_parameters.as_ref().map(|func| {
        quote! {
            fn #call_disclosure() -> for<'a> fn(
                &'a <#source_alias as #op::ApplicationMutationBinding<#schema_alias>>::Input,
            ) -> ::core::result::Result<
                #aq::ApplicationQueryParameterSet<
                    <#disclosure_alias as #aq::ApplicationQueryBinding<#schema_alias>>::Query,
                >,
                #aftermath::ApplicationCorrectionDerivationDenial,
            > {
                #func
            }
        }
    });
    let source_def_helper = spec.source_definition.as_ref().map(|func| {
        quote! {
            fn #call_source_def() -> fn() -> #aschema::ApplicationOperationDefinitionBuilder<
                #schema_alias,
                <#source_alias as #op::ApplicationMutationBinding<#schema_alias>>::Operation,
                <#source_alias as #op::ApplicationMutationBinding<#schema_alias>>::Input,
                true,
                false,
            > {
                #func
            }
        }
    });
    let source_body_helper = spec.source_body.as_ref().map(|func| {
        quote! {
            fn #call_source_body() -> fn() -> #aschema::ApplicationOperationBody<
                #schema_alias,
                <#source_alias as #op::ApplicationMutationBinding<#schema_alias>>::Operation,
                <#source_alias as #op::ApplicationMutationBinding<#schema_alias>>::Input,
            > {
                #func
            }
        }
    });
    let demand_helper = match &spec.demand {
        PreImageDemandSpec::Function(func) => quote! {
            fn #call_demand() -> #aftermath::DeclaredPreImageDemand<#schema_alias> {
                (#func)()
            }
        },
        PreImageDemandSpec::Fields { fields, bytes } => quote! {
            fn #call_demand() -> #aftermath::DeclaredPreImageDemand<#schema_alias> {
                #aftermath::DeclaredPreImageDemand::new(
                    [#(#aftermath::DeclaredPreImageLocus::from_field(#fields)),*],
                    #bytes,
                )
            }
        },
    };
    let inverse_key_helper = spec.inverse_key.as_ref().map(|func| {
        quote! {
            fn #call_inverse_key() -> for<'a> fn(
                &'a str,
            ) -> ::core::result::Result<
                <#aftermath::ApplicationInverseBinding<#schema_alias, #ident> as #op::ApplicationMutationBinding<#schema_alias>>::IdempotencyKey,
                #aschema::ApplicationValueEncodeDenial,
            > {
                #func
            }
        }
    });
    let reapply_key_helper = spec.reapplication_key.as_ref().map(|func| {
        quote! {
            fn #call_reapply_key() -> for<'a> fn(
                &'a str,
            ) -> ::core::result::Result<
                <#source_alias as #op::ApplicationMutationBinding<#schema_alias>>::IdempotencyKey,
                #aschema::ApplicationValueEncodeDenial,
            > {
                #func
            }
        }
    });
    quote! {
        #disclosure_helper
        #source_def_helper
        #source_body_helper
        #demand_helper
        fn #call_derive() -> for<'a, 'b> fn(
            &'a <#source_alias as #op::ApplicationMutationBinding<#schema_alias>>::Input,
            &'b dyn #aftermath::ApplicationCorrectionPreImage<#schema_alias>,
        ) -> ::core::result::Result<
            #inverse_alias,
            #aftermath::ApplicationCorrectionDerivationDenial,
        > {
            #derive_fn
        }
        #inverse_key_helper
        #reapply_key_helper
    }
}
