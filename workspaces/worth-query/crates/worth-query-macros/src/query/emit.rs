//! Lowers `#[derive(WorthQuery)]` into the public declaration macros,
//! hygienic parameter bindings with exact-type nominal witnesses, and typed
//! query-definition defaults.
use super::emit_definition::{definition_builder, managed_index_intent_impl};
use super::grammar::{QueryDeclaration, QueryScope};
use super::hygiene::HygienicNames;
use crate::case::snake_case;
use crate::facade_paths::{declaration, fragment};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{DeriveInput, Ident};

pub(super) fn query(input: &DeriveInput, decl_spec: &QueryDeclaration) -> TokenStream {
    let names = HygienicNames::new(input, decl_spec);
    let marker = query_marker(input, decl_spec);
    let request_binding = request_input_binding(input, decl_spec);
    let params = parameter_items(input, decl_spec, &names);
    let scope_fn = scope_extractor(input, decl_spec, &names);
    let binding = query_binding(input, decl_spec);
    let definition = definition_builder(input, decl_spec, &names);
    let managed_index_intent = managed_index_intent_impl(input, decl_spec, &names);
    quote! {
        #marker
        #request_binding
        #params
        #scope_fn
        #binding
        #definition
        #managed_index_intent
    }
}

fn identity(decl_spec: &QueryDeclaration, role: &str) -> TokenStream {
    let (decl, namespace) = (declaration(), &decl_spec.namespace);
    let role = Ident::new(role, Span::call_site());
    quote!(#decl::worth_query_identity!(#namespace, #role))
}

fn query_marker(input: &DeriveInput, decl_spec: &QueryDeclaration) -> TokenStream {
    let (decl, frag, vis) = (declaration(), fragment(), &input.vis);
    let (query, schema, result, query_name) = (
        &decl_spec.query,
        &decl_spec.schema,
        &decl_spec.result,
        &decl_spec.query_name,
    );
    let query_identity = identity(decl_spec, "query");
    let (selection, _) = decl_spec.scope.selected_column();
    let scope_entity = quote!(<#selection as #frag::RecordSelection>::Entity);
    let scope_identity = decl_spec.scope_name.as_ref().map_or_else(
        || quote!(<#scope_entity as #frag::Entity>::NAME),
        |explicit| quote!(#explicit),
    );
    let parameters = decl_spec
        .parameters_binding
        .as_ref()
        .map(|binding| quote!(parameters #binding,));
    quote! {
        #decl::worth_query_application_query!(
            #vis #query for #schema,
            identity #query_identity,
            #parameters
            result #result,
            scope #scope_entity => #scope_identity,
            name #query_name
        );
    }
}

fn request_input_binding(input: &DeriveInput, decl_spec: &QueryDeclaration) -> TokenStream {
    let (decl, request, vis) = (declaration(), &input.ident, &input.vis);
    let input_binding = &decl_spec.input;
    let request_identity = identity(decl_spec, "request");
    let validate = decl_spec
        .validate
        .as_ref()
        .map(|validate| quote!(, validate: #validate));
    quote! {
        #decl::worth_query_structured_value_binding!(#vis #input_binding for #request {
            identity: #request_identity #validate
        });
    }
}

fn parameter_items(
    input: &DeriveInput,
    decl_spec: &QueryDeclaration,
    names: &HygienicNames,
) -> Option<TokenStream> {
    if decl_spec.params.is_empty() {
        return None;
    }
    let (decl, frag, request, vis) = (declaration(), fragment(), &input.ident, &input.vis);
    let query = &decl_spec.query;
    let module = format_ident!(
        "{}_params",
        snake_case(&request.to_string()),
        span = request.span()
    );
    let bindings = decl_spec
        .bindings
        .as_ref()
        .map_or_else(|| quote!(#frag::NativeScalars), |ty| quote!(#ty));
    let (schema_alias, bindings_alias, params_local, input_local, value_local, witness_local) = (
        &names.schema_alias,
        &names.bindings_alias,
        &names.params_local,
        &names.input_local,
        &names.value_local,
        &names.witness_local,
    );
    let schema = &decl_spec.schema;
    let markers = decl_spec.params.iter().map(|param| {
        let marker = &param.marker;
        let doc = format!(
            "Marker for the `{}` query parameter.",
            param.wire_name.value()
        );
        quote!(#[doc = #doc] pub struct #marker;)
    });
    let scalar_bindings: Vec<TokenStream> = decl_spec
        .params
        .iter()
        .map(|param| {
            let scalar_ty = param.optional_inner.as_ref().unwrap_or(&param.value_ty);
            if let Some(explicit) = param.binding.as_ref() {
                quote!(#explicit)
            } else if param.is_probe
                && let Some(mi) = decl_spec.managed_index.as_ref()
            {
                let idx = &mi.index;
                quote!(<#idx as #decl::application_query::ApplicationManagedIndexBinding<#schema_alias>>::ProbeBinding)
            } else {
                quote!(<#bindings_alias as #frag::Scalar<#scalar_ty>>::Binding)
            }
        })
        .collect();
    let accessors = decl_spec
        .params
        .iter()
        .zip(scalar_bindings.iter())
        .map(|(param, scalar_binding)| {
            let (selector, marker, wire, ty) = (
                &param.selector,
                &param.marker,
                &param.wire_name,
                &param.value_ty,
            );
            let doc = format!("Typed parameter reference for `{}`.", wire.value());
            if param.optional_inner.is_some() {
                quote_spanned! {ty.span()=>
                    #[doc = #doc]
                    #[allow(dead_code)]
                    #vis const fn #selector(
                    ) -> #decl::application_query::ApplicationQueryParameterRef<
                        #query, #module::#marker, #scalar_binding, true,
                    > {
                        #decl::application_query::ApplicationQueryParameterRef::optional(#wire)
                    }
                }
            } else {
                quote_spanned! {ty.span()=>
                    #[doc = #doc]
                    #[allow(dead_code)]
                    #vis const fn #selector(
                    ) -> #decl::application_query::ApplicationQueryParameterRef<
                        #query, #module::#marker, #scalar_binding,
                    > {
                        #decl::application_query::ApplicationQueryParameterRef::from_query_identifier(#wire)
                    }
                }
            }
        });
    let binds = decl_spec
        .params
        .iter()
        .zip(scalar_bindings.iter())
        .map(|(param, scalar_binding)| {
            let (member, selector, ty) = (&param.member, &param.selector, &param.value_ty);
            let expected_scalar = quote!(
                <#scalar_binding as #decl::application_schema::ApplicationScalarValueBinding>::Value
            );
            if param.optional_inner.is_some() {
                quote_spanned! {ty.span()=>
                    let _: fn(*mut #ty) -> *mut #ty =
                        (|#witness_local| #witness_local)
                            as fn(*mut ::core::option::Option<#expected_scalar>) -> *mut ::core::option::Option<#expected_scalar>;
                    let #value_local: #ty =
                        <#ty as ::core::clone::Clone>::clone(&#input_local.#member);
                    #params_local = #params_local.bind_optional(
                        #query::#selector(),
                        #value_local,
                    )?;
                }
            } else {
                quote_spanned! {ty.span()=>
                    let _: fn(*mut #ty) -> *mut #ty =
                        (|#witness_local| #witness_local)
                            as fn(*mut #expected_scalar) -> *mut #expected_scalar;
                    let #value_local: #ty =
                        <#ty as ::core::clone::Clone>::clone(&#input_local.#member);
                    #params_local = #params_local.bind(
                        #query::#selector(),
                        #value_local,
                    )?;
                }
            }
        });
    let doc = format!("Named query parameter markers of `{request}`.");
    Some(quote! {
        #[doc = #doc]
        #vis mod #module {
            #(#markers)*
        }
        const _: () = {
            type #schema_alias = #schema;
            type #bindings_alias = #bindings;

            const _: () = {
                #[allow(dead_code, non_snake_case)]
                fn #params_local() {}
                #[allow(dead_code, non_snake_case)]
                fn #input_local() {}
                #[allow(dead_code, non_snake_case)]
                fn #value_local() {}
                #[allow(dead_code, non_snake_case)]
                fn #witness_local() {}

                impl #query {
                    #(#accessors)*
                }
                #[allow(clippy::clone_on_copy)]
                impl #request {
                    #[doc(hidden)]
                    fn worth_query_try_parameters(
                        #input_local: &Self,
                    ) -> ::core::result::Result<
                        #decl::application_query::ApplicationQueryParameterSet<#query>,
                        #decl::application_schema::ApplicationValueEncodeDenial,
                    > {
                        let mut #params_local =
                            #decl::application_query::ApplicationQueryParameterSet::new();
                        #(#binds)*
                        ::core::result::Result::Ok(#params_local)
                    }
                }
            };
        };
    })
}

fn scope_extractor(
    input: &DeriveInput,
    decl_spec: &QueryDeclaration,
    names: &HygienicNames,
) -> Option<TokenStream> {
    let QueryScope::Field {
        member,
        value_ty,
        selection,
        ..
    } = &decl_spec.scope
    else {
        return None;
    };
    let (frag, request, input_local) = (fragment(), &input.ident, &names.input_local);
    let read = quote_spanned!(value_ty.span()=> #input_local.#member);
    Some(quote! {
        const _: () = {
            #[allow(dead_code, non_snake_case)]
            fn #input_local() {}
            impl #request {
                #[doc(hidden)]
                fn worth_query_scope(#input_local: Self) -> <#selection as #frag::RecordSelection>::Value {
                    #read
                }
            }
        };
    })
}

fn query_binding(input: &DeriveInput, decl_spec: &QueryDeclaration) -> TokenStream {
    let (decl, request, vis) = (declaration(), &input.ident, &input.vis);
    let QueryDeclaration {
        schema,
        context,
        query,
        input: input_binding,
        binding,
        limits,
        ..
    } = decl_spec;
    let binding_identity = identity(decl_spec, "binding");
    let parameters_clause = if let Some(custom_fn) = &decl_spec.parameters_fn {
        quote!(parameters => #custom_fn,)
    } else if !decl_spec.params.is_empty() {
        quote!(try_parameters => #request::worth_query_try_parameters,)
    } else {
        TokenStream::new()
    };
    let scope = match &decl_spec.scope {
        QueryScope::Field {
            selection, column, ..
        } => quote! {
            scope #selection, field <#selection>::at(#column), value #request::worth_query_scope,
        },
        QueryScope::Principal { selection, column } => quote! {
            scope #selection, principal_field <#selection>::at(#column),
        },
        QueryScope::Value {
            selection,
            column,
            function,
        } => quote! {
            scope #selection, field <#selection>::at(#column), value #function,
        },
    };
    quote! {
        #decl::worth_query_query_binding!(
            #vis #binding for #request, schema #schema, context #context,
            identity #binding_identity, input #input_binding, query #query,
            #parameters_clause
            #scope
            limits results #limits
        );
    }
}
