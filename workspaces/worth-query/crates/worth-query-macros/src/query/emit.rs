//! Lowers `#[derive(WorthQuery)]` into the public declaration macros and
//! typed query-definition defaults.
use super::grammar::{bare_ident, QueryDeclaration, QueryScope};
use crate::case::snake_case;
use crate::facade_paths::{declaration, fragment};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{DeriveInput, Expr, Ident};

pub(super) fn query(input: &DeriveInput, decl_spec: &QueryDeclaration) -> TokenStream {
    let marker = query_marker(input, decl_spec);
    let request_binding = request_input_binding(input, decl_spec);
    let params = parameter_items(input, decl_spec);
    let scope_fn = scope_extractor(input, decl_spec);
    let binding = query_binding(input, decl_spec);
    let definition = definition_builder(input, decl_spec);
    quote! {
        #marker
        #request_binding
        #params
        #scope_fn
        #binding
        #definition
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

fn parameter_items(input: &DeriveInput, decl_spec: &QueryDeclaration) -> Option<TokenStream> {
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
    let markers = decl_spec.params.iter().map(|param| {
        let marker = &param.marker;
        let doc = format!(
            "Marker for the `{}` query parameter.",
            param.wire_name.value()
        );
        quote!(#[doc = #doc] pub struct #marker;)
    });
    let accessors = decl_spec.params.iter().map(|param| {
        let (selector, marker, wire, ty) = (
            &param.selector,
            &param.marker,
            &param.wire_name,
            &param.value_ty,
        );
        let scalar_binding = param.binding.as_ref().map_or_else(
            || quote!(<#bindings as #frag::Scalar<#ty>>::Binding),
            |explicit| quote!(#explicit),
        );
        let doc = format!("Typed parameter reference for `{}`.", wire.value());
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
    });
    let binds = decl_spec.params.iter().map(|param| {
        let (member, selector, ty) = (&param.member, &param.selector, &param.value_ty);
        quote_spanned! {ty.span()=>
            __worth_params = __worth_params.bind(
                #query::#selector(),
                ::core::clone::Clone::clone(&input.#member),
            )?;
        }
    });
    let doc = format!("Named query parameter markers of `{request}`.");
    Some(quote! {
        #[doc = #doc]
        #vis mod #module {
            #(#markers)*
        }
        impl #query {
            #(#accessors)*
        }
        #[allow(clippy::clone_on_copy)]
        impl #request {
            #[doc(hidden)]
            fn worth_query_try_parameters(
                input: &Self,
            ) -> ::core::result::Result<
                #decl::application_query::ApplicationQueryParameterSet<#query>,
                #decl::application_schema::ApplicationValueEncodeDenial,
            > {
                let mut __worth_params =
                    #decl::application_query::ApplicationQueryParameterSet::new();
                #(#binds)*
                ::core::result::Result::Ok(__worth_params)
            }
        }
    })
}

fn scope_extractor(input: &DeriveInput, decl_spec: &QueryDeclaration) -> Option<TokenStream> {
    let QueryScope::Field {
        member,
        value_ty,
        selection,
        ..
    } = &decl_spec.scope
    else {
        return None;
    };
    let (frag, request) = (fragment(), &input.ident);
    let read = quote_spanned!(value_ty.span()=> input.#member);
    Some(quote! {
        impl #request {
            #[doc(hidden)]
            fn worth_query_scope(input: Self) -> <#selection as #frag::RecordSelection>::Value {
                #read
            }
        }
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

fn definition_builder(input: &DeriveInput, decl_spec: &QueryDeclaration) -> Option<TokenStream> {
    let def = decl_spec.definition.as_ref()?;
    let (decl, frag, vis) = (declaration(), fragment(), &input.vis);
    let (schema, query, result) = (&decl_spec.schema, &decl_spec.query, &decl_spec.result);
    let root = &def.root;
    let aq = quote!(#decl::application_query);
    let aschema = quote!(#decl::application_schema);
    let param_val = quote!(
        <<Self as #aq::ApplicationQueryMarkerIdentity<#schema>>::ParameterBinding
            as #aschema::ApplicationStructuredValueBinding>::Value
    );
    let result_val = quote!(<#result as #aschema::ApplicationStructuredValueBinding>::Value);
    let scope_entity = quote!(<Self as #aq::ApplicationQueryMarkerIdentity<#schema>>::Scope);
    let cardinality = cardinality_expr(def.cardinality.as_ref());
    let disclosure = disclosure_expr(def.disclosure.as_ref());
    let basis = basis_expr(def.basis_support.as_ref());
    let lanes = lanes_expr(def.lanes.as_ref());
    let auth = def.ability.as_ref().map_or_else(
        || quote!(.public()),
        |ability| quote!(.requires_ability(#ability)),
    );
    let params = decl_spec.params.iter().map(|param| {
        let selector = &param.selector;
        quote!(.parameter(Self::#selector()))
    });
    let default_definition = (!def.explicit_shape).then(|| {
        quote! {
            /// Builds the query definition using the projection row's derived
            /// `result_shape()` and the query's declared defaults and parameters.
            #[allow(dead_code)]
            #vis fn definition() -> #aq::ApplicationQueryDefinitionBuilder<
                #schema,
                Self,
                #param_val,
                #result_val,
                #scope_entity,
            > {
                Self::definition_with(<#result_val>::result_shape::<Self>())
            }
        }
    });
    Some(quote! {
        impl #query {
            #default_definition

            /// Builds the query definition with an explicit result shape and
            /// the query's declared defaults and parameters.
            #[allow(dead_code)]
            #vis fn definition_with(
                shape: #aq::TypedApplicationQueryResultShape<
                    #schema,
                    Self,
                    #root,
                    #result_val,
                    #result,
                >,
            ) -> #aq::ApplicationQueryDefinitionBuilder<
                #schema,
                Self,
                #param_val,
                #result_val,
                #scope_entity,
            > {
                #aq::ApplicationQueryDefinitionBuilder::declare(Self::reference())
                    .root(#aschema::ApplicationEntityRef::from_schema_identifier(
                        <#root as #frag::Entity>::NAME,
                    ))
                    .scope(#aschema::ApplicationEntityRef::from_schema_identifier(
                        <#scope_entity as #frag::Entity>::NAME,
                    ))
                    .result_shape(shape)
                    .cardinality(#cardinality)
                    .disclosure(#disclosure)
                    .basis_support(#basis)
                    .lanes(#lanes)
                    #auth
                    #(#params)*
            }
        }
    })
}

fn cardinality_expr(expr: Option<&Expr>) -> TokenStream {
    let aq = quote!(::worth_query_decl::facade::application_query);
    match expr {
        None => quote!(#aq::ApplicationQueryCardinality::Many),
        Some(expr) => match bare_ident(expr) {
            Some((ident, _))
                if matches!(
                    ident.to_string().as_str(),
                    "Many" | "ExactlyOne" | "OptionalOne"
                ) =>
            {
                quote!(#aq::ApplicationQueryCardinality::#ident)
            }
            _ => quote!(#expr),
        },
    }
}

fn disclosure_expr(expr: Option<&Expr>) -> TokenStream {
    let aq = quote!(::worth_query_decl::facade::application_query);
    match expr {
        None => quote!(#aq::ApplicationQueryDisclosureContract::public()),
        Some(expr) => match bare_ident(expr) {
            Some((ident, _)) if ident == "Public" => {
                quote!(#aq::ApplicationQueryDisclosureContract::public())
            }
            _ => quote!(#expr),
        },
    }
}

fn basis_expr(expr: Option<&Expr>) -> TokenStream {
    let aq = quote!(::worth_query_decl::facade::application_query);
    match expr {
        None => quote!(#aq::ApplicationQueryBasisSupport::current_and_pinned()),
        Some(expr) => match bare_ident(expr) {
            Some((ident, _)) if ident == "CurrentAndPinned" => {
                quote!(#aq::ApplicationQueryBasisSupport::current_and_pinned())
            }
            Some((ident, _)) if ident == "CurrentOnly" => {
                quote!(#aq::ApplicationQueryBasisSupport::current_only())
            }
            _ => quote!(#expr),
        },
    }
}

fn lanes_expr(expr: Option<&Expr>) -> TokenStream {
    let aq = quote!(::worth_query_decl::facade::application_query);
    match expr {
        None => quote!(#aq::ApplicationQueryLaneEligibility::one_shot()),
        Some(expr) => match bare_ident(expr) {
            Some((ident, _)) if ident == "OneShot" => {
                quote!(#aq::ApplicationQueryLaneEligibility::one_shot())
            }
            _ => quote!(#expr),
        },
    }
}
