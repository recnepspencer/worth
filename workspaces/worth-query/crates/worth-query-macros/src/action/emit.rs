//! An action lowers to the declaration macros a hand-written action invokes.
use super::grammar::{ActionDeclaration, Scope};
use crate::facade_paths::{declaration, fragment};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::{DeriveInput, Ident};

pub(super) fn action(input: &DeriveInput, action: &ActionDeclaration) -> TokenStream {
    let (decl, intent, vis) = (declaration(), &input.ident, &input.vis);
    let input_binding = &action.input;
    let input_identity = identity(action, "input");
    let operation = operation(input, action);
    let binding = mutation_binding(input, action);
    let scope_value = scope_value(input, action);
    let input_fields = input_fields(input, action);
    let delivery = delivery(input, action);
    // An authored body reads input members; other handlers keep the default.
    let capture = action.body.then(|| quote!(, capture_input: true));
    let validate = action
        .validate
        .as_ref()
        .map(|validate| quote!(, validate: #validate));
    quote! {
        #decl::worth_query_structured_value_binding!(#vis #input_binding for #intent {
            identity: #input_identity #capture #validate
        });
        #operation
        #binding
        #scope_value
        #input_fields
        #delivery
    }
}

/// The namespace's identity for `role`, evaluated at compile time.
fn identity(action: &ActionDeclaration, role: &str) -> TokenStream {
    let (decl, namespace) = (declaration(), &action.namespace);
    let role = Ident::new(role, Span::call_site());
    quote!(#decl::worth_query_identity!(#namespace, #role))
}

/// `worth_query_operation!` with the forwarded access block and capabilities.
fn operation(input: &DeriveInput, action: &ActionDeclaration) -> TokenStream {
    let decl = declaration();
    let (vis, operation, schema, input_binding) =
        (&input.vis, &action.operation, &action.schema, &action.input);
    let access = action.access.as_ref().map_or_else(
        || {
            action.body.then(|| {
                let (selection, column) = action.scope.selected_column();
                quote!(; access { read_fields: [<#selection>::at(#column)]; })
            })
        },
        |access| Some(quote!(; access { #access })),
    );
    let capabilities = action
        .capabilities
        .iter()
        .map(|(kind, members)| quote!(; #kind [#(#members),*]));
    quote! {
        #decl::worth_query_operation!(
            #vis #operation for #schema, input #input_binding #access #(#capabilities)*
        );
    }
}

/// The contextual `worth_query_mutation_binding!`.
fn mutation_binding(input: &DeriveInput, action: &ActionDeclaration) -> TokenStream {
    let decl = declaration();
    let (vis, intent) = (&input.vis, &input.ident);
    let ActionDeclaration {
        schema,
        context,
        input: input_binding,
        operation,
        binding,
        result,
        decision,
        ..
    } = action;
    let binding_identity = identity(action, "binding");
    let handler_identity = identity(action, if action.body { "body" } else { "handler" });
    let program = action.program.then(|| quote!(program required,));
    let outputs = action.outputs.as_ref().map_or_else(
        || quote!(#decl::application_operation::NoApplicationMutationOutputs),
        |outputs| quote!(#outputs),
    );
    let scope = match &action.scope {
        Scope::Field {
            selection, column, ..
        } => quote! {
            scope #selection, field <#selection>::at(#column), value #intent::worth_query_scope,
        },
        Scope::Principal { selection, column } => quote! {
            scope #selection, principal_field <#selection>::at(#column),
        },
        Scope::Value {
            selection,
            column,
            function,
        } => quote! {
            scope #selection, field <#selection>::at(#column), value #function,
        },
    };
    quote! {
        #decl::worth_query_mutation_binding!(
            #vis #binding for #intent, schema #schema, context #context,
            identity #binding_identity, input #input_binding, operation #operation,
            result #result, decision #decision, handler identity #handler_identity,
            #program outputs #outputs, #scope
        );
    }
}

/// The scope field's value for the binding's field scope. A field whose type
/// differs from the selection's value fails to compile at that field's type.
fn scope_value(input: &DeriveInput, action: &ActionDeclaration) -> Option<TokenStream> {
    let Scope::Field {
        member,
        value,
        selection,
        ..
    } = &action.scope
    else {
        return None;
    };
    let (frag, intent) = (fragment(), &input.ident);
    let read = quote_spanned!(value.span()=> ::core::clone::Clone::clone(&input.#member));
    // The read is spanned to the field so a non-`Clone` type is refused there;
    // a `Copy` field must not then report `clone_on_copy` in the application.
    Some(quote! {
        #[allow(clippy::clone_on_copy)]
        impl #intent {
            #[doc(hidden)]
            fn worth_query_scope(input: &Self) -> <#selection as #frag::RecordSelection>::Value {
                #read
            }
        }
    })
}

/// Compile-time-typed `<field>_input()` accessors for authored body builders.
fn input_fields(input: &DeriveInput, action: &ActionDeclaration) -> Option<TokenStream> {
    if action.input_fields.is_empty() {
        return None;
    }
    let (decl, intent, vis) = (declaration(), &input.ident, &input.vis);
    let accessors = action.input_fields.iter().map(|field| {
        let accessor = format_ident!("{}_input", field.member.unraw());
        let wire = &field.wire_name;
        let value_ty = &field.value_ty;
        let carrier = if field.optional {
            quote!(#decl::application_schema::ApplicationOptionalInputField)
        } else {
            quote!(#decl::application_schema::ApplicationInputField)
        };
        quote_spanned! {value_ty.span()=>
            #[allow(dead_code)]
            #vis const fn #accessor() -> #carrier<Self, #value_ty> {
                #carrier::new(#wire)
            }
        }
    });
    Some(quote! {
        impl #intent {
            #(#accessors)*
        }
    })
}

fn delivery(input: &DeriveInput, action: &ActionDeclaration) -> Option<TokenStream> {
    let decl = declaration();
    let delivery = action.delivery.as_ref()?;
    let (vis, schema, binding) = (&input.vis, &action.schema, &action.binding);
    let completion = identity(action, "completion");
    Some(quote! {
        #vis struct #delivery;
        impl #decl::application_program::ApplicationActionDelivery<#schema, #binding> for #delivery {
            type Mode = #decl::application_program::ApplicationSourceCompletion;
            const IDENTITY: &'static str = #completion;
        }
    })
}
