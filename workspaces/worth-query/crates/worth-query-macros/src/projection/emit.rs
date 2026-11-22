//! Slot markers, selectors, the result shape and the projection, written as
//! the public items a hand-written projection declares.
use super::grammar::{ProjectionDeclaration, Slot, SlotIdentity};
use crate::case::snake_case;
use crate::facade_paths::{declaration, fragment, host};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::DeriveInput;

pub(super) fn projection(input: &DeriveInput, projection: &ProjectionDeclaration) -> TokenStream {
    let (decl, row, vis) = (declaration(), &input.ident, &input.vis);
    let module = format_ident!("{}_slots", snake_case(&row.to_string()), span = row.span());
    let markers = projection.slots.iter().map(|slot| {
        let (marker, doc) = (&slot.marker, format!("The `{}` slot.", slot.output.value()));
        quote!(#[doc = #doc] pub struct #marker;)
    });
    let identities = projection.slots.iter().map(|slot| {
        let marker = &slot.marker;
        let identity = match &slot.identity {
            SlotIdentity::Derived(namespace) => {
                let output = &slot.output;
                quote!(#decl::worth_query_identity!(#namespace, slot #output))
            }
            SlotIdentity::Explicit(identity) => quote!(#identity),
        };
        quote!(#decl::worth_query_portable_type!(#module::#marker => #identity);)
    });
    let selectors = projection
        .slots
        .iter()
        .map(|slot| selector(input, projection, &module, slot));
    let shape = result_shape(input, projection);
    let project = project(input, projection);
    let doc = format!("Result slot markers of `{row}`, one per projected field.");
    quote! {
        #[doc = #doc]
        #vis mod #module {
            #(#markers)*
        }
        #(#identities)*
        impl #row {
            #(#selectors)*
            #shape
        }
        #project
    }
}

/// `fn field<Q>()`: the query-bound selector of one slot.
fn selector(
    input: &DeriveInput,
    projection: &ProjectionDeclaration,
    module: &syn::Ident,
    slot: &Slot,
) -> TokenStream {
    let (query, vis, schema) = (declaration(), &input.vis, &projection.schema);
    let Slot {
        member,
        output,
        marker,
        selection,
        column,
        ..
    } = slot;
    let reference = if slot.optional {
        quote!(#query::application_query::ApplicationQueryOptionalResultFieldRef)
    } else {
        quote!(#query::application_query::ApplicationQueryResultFieldRef)
    };
    let doc = format!("The `{}` slot of query `Q`.", output.value());
    quote_spanned! {selection.span()=>
        #[doc = #doc]
        #vis fn #member<Q>() -> #reference<Q, #module::#marker, #schema, #selection> {
            #reference::new(#output, <#selection>::at(#column))
        }
    }
}

fn query_bounds(projection: &ProjectionDeclaration) -> TokenStream {
    let (decl, schema) = (declaration(), &projection.schema);
    let marker = quote!(#decl::application_query::ApplicationQueryMarkerIdentity<#schema>);
    quote! {
        Q: #marker,
        <Q as #marker>::ResultBinding:
            #decl::application_schema::ApplicationStructuredValueBinding<Value = Self>,
    }
}

/// `fn result_shape<Q>()`: every slot, rooted at the declared entity.
fn result_shape(input: &DeriveInput, projection: &ProjectionDeclaration) -> TokenStream {
    let (decl, frag, vis) = (declaration(), fragment(), &input.vis);
    let (schema, root) = (&projection.schema, &projection.root);
    let bounds = query_bounds(projection);
    let fields = projection.slots.iter().map(|slot| {
        let member = &slot.member;
        if slot.optional {
            quote!(.optional_field(Self::#member::<Q>()))
        } else {
            quote!(.field(Self::#member::<Q>()))
        }
    });
    quote! {
        /// The result shape of query `Q`, rooted at the declared root entity.
        #vis fn result_shape<Q>() -> #decl::application_query::TypedApplicationQueryResultShape<
            #schema,
            Q,
            #root,
            Self,
            <Q as #decl::application_query::ApplicationQueryMarkerIdentity<#schema>>::ResultBinding,
        >
        where
            #bounds
        {
            #decl::application_query::ApplicationQueryResultShapeBuilder::new(
                #decl::application_schema::ApplicationEntityRef::from_schema_identifier(
                    <#root as #frag::Entity>::NAME,
                ),
            )
            #(#fields)*
            .build()
        }
    }
}

/// The projection reads each slot into its field; a field whose type differs
/// from its selection's value fails to compile at that field's type.
fn project(input: &DeriveInput, projection: &ProjectionDeclaration) -> TokenStream {
    let (graph, row, schema) = (host(), &input.ident, &projection.schema);
    let bounds = query_bounds(projection);
    let reads = projection.slots.iter().map(|slot| {
        let member = &slot.member;
        let read = if slot.optional {
            quote_spanned!(slot.value.span()=> row.optional_field(Self::#member::<Q>())?)
        } else {
            quote_spanned!(slot.value.span()=> row.field(Self::#member::<Q>())?)
        };
        quote!(#member: #read)
    });
    quote! {
        impl<Q> #graph::primary_graph::WorthQueryApplicationProjection<#schema, Q> for #row
        where
            #bounds
        {
            fn project(
                row: &#graph::primary_graph::WorthQueryApplicationProjectionRow<'_, #schema, Q>,
            ) -> ::core::result::Result<
                Self,
                #graph::primary_graph::WorthQueryApplicationProjectionDenial,
            > {
                ::core::result::Result::Ok(Self { #(#reads,)* })
            }
        }
    }
}
