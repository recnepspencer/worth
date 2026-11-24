//! Lowers `#[derive(WorthManagedIndex)]` into `ApplicationManagedIndexBinding<Schema>`
//! with two-layer `const _: ()` hygiene and checked binary-source-tree defaults.
use super::grammar::{bare_ident, ManagedIndexDeclaration, ProbeSpec};
use super::hygiene::HygienicNames;
use crate::facade_paths::declaration;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{DeriveInput, Expr};

pub(super) fn managed_index(input: &DeriveInput, spec: &ManagedIndexDeclaration) -> TokenStream {
    let (decl, ident) = (declaration(), &input.ident);
    let aq = quote!(#decl::application_query);
    let aschema = quote!(#decl::application_schema);
    let names = HygienicNames::new(input, spec);
    let HygienicNames {
        index_alias,
        schema_alias,
        entity_alias,
        source_alias,
        probe_alias,
        source_scope_const,
        structure_const,
        membership_const,
        maintenance_const,
        refinement_const,
    } = &names;

    let (schema, entity, source) = (&spec.schema, &spec.entity, &spec.source);
    let probe_type = match &spec.probe {
        ProbeSpec::ValueType(ty) => quote_spanned! {ty.span()=>
            <#ty as #aschema::WorthQueryDefaultValueBinding>::Binding
        },
        ProbeSpec::BindingType(ty) => quote_spanned!(ty.span()=> #ty),
    };
    let (name, provider, version) = (&spec.name, &spec.provider, spec.provider_version);
    let source_scope_val = source_scope_expr(&aq, spec.source_scope.as_ref());
    let structure_val = structure_expr(&aq, spec.structure.as_ref());
    let membership_val = membership_expr(&aq, spec.membership.as_ref());
    let maintenance_val = maintenance_expr(&aq, spec.maintenance.as_ref());
    let refinement_val = spec.refinement.as_ref().map_or_else(
        || quote!(::core::option::Option::None),
        |expr| quote!(#expr),
    );

    quote! {
        const _: () = {
            type #index_alias = #ident;
            type #schema_alias = #schema;
            type #entity_alias = #entity;
            type #source_alias = #source;
            type #probe_alias = #probe_type;
            const #source_scope_const: #aq::ApplicationManagedIndexSourceScope = #source_scope_val;
            const #structure_const: #aq::ApplicationManagedIndexSourceStructure = #structure_val;
            const #membership_const: #aq::ApplicationManagedIndexMembership = #membership_val;
            const #maintenance_const: #aq::ApplicationManagedIndexMaintenance = #maintenance_val;
            const #refinement_const: ::core::option::Option<#aq::ApplicationManagedIndexRefinement> =
                #refinement_val;

            const _: () = {
                impl #aq::ApplicationManagedIndexBinding<#schema_alias> for #index_alias {
                    type SourceBinding = #source_alias;
                    type ProbeBinding = #probe_alias;
                    type Entity = #entity_alias;
                    const NAME: &'static str = #name;
                    const PROVIDER: &'static str = #provider;
                    const PROVIDER_VERSION: u32 = #version;
                    const SOURCE_SCOPE: #aq::ApplicationManagedIndexSourceScope =
                        #source_scope_const;
                    const STRUCTURE: #aq::ApplicationManagedIndexSourceStructure = #structure_const;
                    const MEMBERSHIP: #aq::ApplicationManagedIndexMembership = #membership_const;
                    const MAINTENANCE: #aq::ApplicationManagedIndexMaintenance = #maintenance_const;
                    const REFINEMENT: ::core::option::Option<
                        #aq::ApplicationManagedIndexRefinement,
                    > = #refinement_const;
                }
            };
        };
    }
}

fn source_scope_expr(aq: &TokenStream, expr: Option<&Expr>) -> TokenStream {
    match expr {
        None => quote!(#aq::ApplicationManagedIndexSourceScope::SameAdmittedScope),
        Some(expr) => match bare_ident(expr) {
            Some(ident) if ident == "SameAdmittedScope" => {
                quote!(#aq::ApplicationManagedIndexSourceScope::SameAdmittedScope)
            }
            _ => quote!(#expr),
        },
    }
}

fn structure_expr(aq: &TokenStream, expr: Option<&Expr>) -> TokenStream {
    match expr {
        None => quote!(#aq::ApplicationManagedIndexSourceStructure::BinarySourceTree),
        Some(expr) => match bare_ident(expr) {
            Some(ident) if matches!(ident.to_string().as_str(), "BinarySourceTree" | "Opaque") => {
                quote!(#aq::ApplicationManagedIndexSourceStructure::#ident)
            }
            _ => quote!(#expr),
        },
    }
}

fn membership_expr(aq: &TokenStream, expr: Option<&Expr>) -> TokenStream {
    match expr {
        None => quote!(#aq::ApplicationManagedIndexMembership::CheckedSummarySourceTree),
        Some(expr) => match bare_ident(expr) {
            Some(ident)
                if matches!(
                    ident.to_string().as_str(),
                    "CheckedSummarySourceTree" | "Candidates"
                ) =>
            {
                quote!(#aq::ApplicationManagedIndexMembership::#ident)
            }
            _ => quote!(#expr),
        },
    }
}

fn maintenance_expr(aq: &TokenStream, expr: Option<&Expr>) -> TokenStream {
    match expr {
        None => quote!(#aq::ApplicationManagedIndexMaintenance::Native {
            cold: #aq::ApplicationManagedIndexColdSource::AdmitRebuild,
        }),
        Some(expr) => match bare_ident(expr) {
            Some(ident) if ident == "WhenRequired" => {
                quote!(#aq::ApplicationManagedIndexMaintenance::WhenRequired)
            }
            Some(ident) if ident == "NativeAdmitRebuild" => {
                quote!(#aq::ApplicationManagedIndexMaintenance::Native {
                    cold: #aq::ApplicationManagedIndexColdSource::AdmitRebuild,
                })
            }
            Some(ident) if ident == "NativeRefuseCold" => {
                quote!(#aq::ApplicationManagedIndexMaintenance::Native {
                    cold: #aq::ApplicationManagedIndexColdSource::RefuseCold,
                })
            }
            _ => quote!(#expr),
        },
    }
}
