//! Lowers `#[derive(WorthManagedIndex)]` into `ApplicationManagedIndexBinding<Schema>`
//! (and optional `WorthQueryApplicationManagedIndexRegistration<Schema>` when `kind = ..`
//! is declared) with two-layer `const _: ()` hygiene.
use super::grammar::{bare_ident, IndexProviderKind, ManagedIndexDeclaration, ProbeSpec};
use super::hygiene::HygienicNames;
use crate::facade_paths::{declaration, host};
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
        install_fn: _,
        setup_param: _,
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
    let (outer_install_helper, inner_registration_impl) = spec.kind.as_ref().map_or_else(
        || (TokenStream::new(), TokenStream::new()),
        |kind| emit_provider_registration(kind, &names),
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
            #outer_install_helper

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

                #inner_registration_impl
            };
        };
    }
}

fn emit_provider_registration(
    kind: &IndexProviderKind,
    names: &HygienicNames,
) -> (TokenStream, TokenStream) {
    let host_facade = host();
    let contrib = quote!(#host_facade::application_contribution);
    let pg = quote!(#host_facade::primary_graph);
    let oi = quote!(#host_facade::ordered_index);
    let (index_alias, schema_alias, install_fn, setup_param) = (
        &names.index_alias,
        &names.schema_alias,
        &names.install_fn,
        &names.setup_param,
    );
    let (provider_expr, with_watches) = match kind {
        IndexProviderKind::TextPrefix { field, exact } => {
            let ctor = if *exact {
                quote!(exact_utf8)
            } else {
                quote!(ascii_case_insensitive)
            };
            (quote!(#oi::WorthQueryTextPrefixIndex::#ctor(#field)), true)
        }
        IndexProviderKind::Interval1D { start, end, closed } => {
            let ctor = if *closed {
                quote!(closed)
            } else {
                quote!(half_open)
            };
            (
                quote!(#oi::WorthQueryInterval1DIndex::#ctor(#start, #end)),
                true,
            )
        }
        IndexProviderKind::Box2D {
            min_x,
            min_y,
            max_x,
            max_y,
            closed,
        } => {
            let ctor = if *closed {
                quote!(closed)
            } else {
                quote!(half_open)
            };
            (
                quote!(#oi::WorthQueryBox2DIndex::#ctor(#min_x, #min_y, #max_x, #max_y)),
                true,
            )
        }
        IndexProviderKind::Box3D { min, max, closed } => {
            let ctor = if *closed {
                quote!(closed)
            } else {
                quote!(half_open)
            };
            (quote!(#oi::WorthQueryBox3DIndex::#ctor(#min, #max)), true)
        }
        IndexProviderKind::Custom { provider, watches } => (quote!(#provider), *watches),
    };
    let install_call = if with_watches {
        quote! {
            #setup_param
                .managed_index_provider::<#index_alias, _>(#provider_expr)
                .with_predicate_routing()
                .install()
        }
    } else {
        quote! {
            #setup_param
                .managed_index_provider::<#index_alias, _>(#provider_expr)
                .install()
        }
    };
    let outer_helper = quote! {
        fn #install_fn(
            #setup_param: &mut #contrib::WorthQueryApplicationContributionSetup<'_, #schema_alias>,
        ) -> ::core::result::Result<(), #pg::WorthQueryPrimaryGraphInstallationDenial> {
            #install_call
        }
    };
    let inner_impl = quote! {
        impl #contrib::WorthQueryApplicationManagedIndexRegistration<#schema_alias> for #index_alias {
            fn install_managed_index(
                #setup_param: &mut #contrib::WorthQueryApplicationContributionSetup<'_, #schema_alias>,
            ) -> ::core::result::Result<(), #pg::WorthQueryPrimaryGraphInstallationDenial> {
                #install_fn(#setup_param)
            }
        }
    };
    (outer_helper, inner_impl)
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
