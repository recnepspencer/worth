//! Slot markers, selectors, the result shape and the projection, written as
//! the public items a hand-written projection declares.
use super::grammar::{Cardinality, ProjectionDeclaration, Slot, SlotIdentity, SlotKind, Traversal};
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
    let selectors = projection.slots.iter().map(|slot| match &slot.kind {
        SlotKind::Field { .. } => field_selector(input, projection, &module, slot),
        SlotKind::Relation(_) => relation_selector(input, projection, &module, slot),
    });
    let (builder, shape) = (
        result_shape_builder(input, projection),
        result_shape(input, projection),
    );
    let (project_row, project) = (project_row(input, projection), project(input, projection));
    let projects_names = projects_names(input, projection);
    let self_binding = projection.namespace.as_ref().map(|namespace| {
        quote! {
            impl #decl::application_schema::ApplicationStructuredValueBinding for #row {
                type Value = Self;
                const IDENTITY_NAME: &'static str =
                    #decl::worth_query_identity!(#namespace, result);

                fn validate(
                    _: &Self::Value,
                ) -> ::core::result::Result<
                    (),
                    #decl::application_schema::ApplicationValueValidationDenial,
                > {
                    ::core::result::Result::Ok(())
                }
            }
        }
    });
    let doc = format!("Result slot markers of `{row}`, one per projected field.");
    quote! {
        #[doc = #doc]
        #vis mod #module {
            #(#markers)*
        }
        #(#identities)*
        #self_binding
        impl #row {
            #(#selectors)*
            #builder
            #shape
            #project_row
            #projects_names
        }
        #project
    }
}

/// `fn selector<Q>()`: the query-bound selector of one field slot.
fn field_selector(
    input: &DeriveInput,
    projection: &ProjectionDeclaration,
    module: &syn::Ident,
    slot: &Slot,
) -> TokenStream {
    let (query, vis, schema) = (declaration(), &input.vis, &projection.schema);
    let SlotKind::Field {
        selection,
        column,
        optional,
    } = &slot.kind
    else {
        unreachable!("field selectors select fields")
    };
    let (selector, output, marker) = (&slot.selector, &slot.output, &slot.marker);
    let reference = if *optional {
        quote!(#query::application_query::ApplicationQueryOptionalResultFieldRef)
    } else {
        quote!(#query::application_query::ApplicationQueryResultFieldRef)
    };
    let doc = format!("The `{}` slot of query `Q`.", output.value());
    quote_spanned! {selection.span()=>
        #[doc = #doc]
        #vis fn #selector<Q>() -> #reference<Q, #module::#marker, #schema, #selection> {
            #reference::new(#output, <#selection>::at(#column))
        }
    }
}

/// `fn selector<Q>()`: the query-bound selector of one traversed relation.
/// A forward traversal leaves the root along a relation it declares; a
/// reverse one arrives at the root from the relation's declaring entity.
fn relation_selector(
    input: &DeriveInput,
    projection: &ProjectionDeclaration,
    module: &syn::Ident,
    slot: &Slot,
) -> TokenStream {
    let SlotKind::Relation(traversal) = &slot.kind else {
        unreachable!("relation selectors select relations")
    };
    let (decl, frag, vis) = (declaration(), fragment(), &input.vis);
    let (schema, relation) = (&projection.schema, &traversal.relation);
    let (selector, output, marker) = (&slot.selector, &slot.output, &slot.marker);
    let to = quote!(<#relation as #frag::Relation>::To);
    let (from, direction, toward) = match &traversal.reverse_from {
        None => (&projection.root, quote!(ForwardResultTraversal), "forward"),
        Some(from) => (from, quote!(ReverseResultTraversal), "reverse"),
    };
    let (count, many) = match traversal.cardinality {
        Cardinality::One => (quote!(ExactlyOneResult), "one"),
        Cardinality::Optional => (quote!(OptionalOneResult), "optional"),
        Cardinality::Many => (quote!(ManyResults), "many"),
    };
    let constructor = format_ident!("{toward}_{many}");
    let query = quote!(#decl::application_query);
    let reference = quote!(#query::ApplicationQueryResultRelationRef);
    let doc = format!("The `{}` relation slot of query `Q`.", output.value());
    quote_spanned! {relation.span()=>
        #[doc = #doc]
        #vis fn #selector<Q>() -> #reference<
            Q, #module::#marker, #schema, #relation, #from, #to, #query::#direction, #query::#count,
        > {
            #reference::#constructor(
                #output,
                #decl::application_schema::ApplicationRelationRef::from_schema_identifiers(
                    <#relation as #frag::Relation>::NAME,
                    <#from as #frag::Entity>::NAME,
                    <#to as #frag::Entity>::NAME,
                    #frag::relation_integrity::<#relation>(),
                ),
            )
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

/// `fn result_shape_builder<Q, B>()`: every slot, rooted at the declared
/// entity and identified by `B`; a relation slot nests its child's builder.
fn result_shape_builder(input: &DeriveInput, projection: &ProjectionDeclaration) -> TokenStream {
    let (decl, frag, vis) = (declaration(), fragment(), &input.vis);
    let (schema, root) = (&projection.schema, &projection.root);
    let binding = quote!(#decl::application_schema::ApplicationStructuredValueBinding);
    let members = projection.slots.iter().map(|slot| {
        let selector = &slot.selector;
        match &slot.kind {
            SlotKind::Field { optional: true, .. } => {
                quote!(.optional_field(Self::#selector::<Q>()))
            }
            SlotKind::Field { .. } => quote!(.field(Self::#selector::<Q>())),
            SlotKind::Relation(Traversal { child, nested, .. }) => quote_spanned! {child.span()=>
                .relation(
                    Self::#selector::<Q>(),
                    <#child>::result_shape_builder::<Q, #nested>(),
                )
            },
        }
    });
    quote! {
        /// The unbuilt result shape of query `Q` identified by binding `B`,
        /// which a parent projection nests under one of its relation slots.
        #vis fn result_shape_builder<Q, B>() -> #decl::application_query::ApplicationQueryResultShapeBuilder<
            #schema, Q, #root, <B as #binding>::Value, B,
        >
        where
            Q: #decl::application_query::ApplicationQueryMarkerIdentity<#schema>,
            B: #binding,
        {
            #decl::application_query::ApplicationQueryResultShapeBuilder::new(
                #decl::application_schema::ApplicationEntityRef::from_schema_identifier(
                    <#root as #frag::Entity>::NAME,
                ),
            )
            #(#members)*
        }
    }
}

/// `fn result_shape<Q>()`: every slot, rooted at the declared root entity.
fn result_shape(input: &DeriveInput, projection: &ProjectionDeclaration) -> TokenStream {
    let (decl, vis) = (declaration(), &input.vis);
    let (schema, root) = (&projection.schema, &projection.root);
    let bounds = query_bounds(projection);
    let marker = quote!(#decl::application_query::ApplicationQueryMarkerIdentity<#schema>);
    quote! {
        /// The result shape of query `Q`, rooted at the declared root entity.
        #vis fn result_shape<Q>() -> #decl::application_query::TypedApplicationQueryResultShape<
            #schema,
            Q,
            #root,
            Self,
            <Q as #marker>::ResultBinding,
        >
        where
            #bounds
        {
            Self::result_shape_builder::<Q, <Q as #marker>::ResultBinding>().build()
        }
    }
}

/// `fn project_row<Q>()` reads each slot into its field; a field whose type
/// differs from its selection's value fails to compile at that field's type.
fn project_row(input: &DeriveInput, projection: &ProjectionDeclaration) -> TokenStream {
    let (decl, graph, vis, schema) = (declaration(), host(), &input.vis, &projection.schema);
    let denial = quote!(#graph::primary_graph::WorthQueryApplicationProjectionDenial);
    let reads = projection.slots.iter().map(|slot| {
        let (member, selector, span) = (&slot.member, &slot.selector, slot.value.span());
        let read = match &slot.kind {
            SlotKind::Field { optional: true, .. } => {
                quote_spanned!(span=> row.optional_field(Self::#selector::<Q>())?)
            }
            SlotKind::Field { .. } => quote_spanned!(span=> row.field(Self::#selector::<Q>())?),
            SlotKind::Relation(Traversal {
                child, cardinality, ..
            }) => {
                let child = quote_spanned!(span=> <#child>::project_row::<Q>);
                match cardinality {
                    Cardinality::One => quote!(#child(&row.one(Self::#selector::<Q>())?)?),
                    Cardinality::Optional => quote! {
                        row.optional(Self::#selector::<Q>())?
                            .map(|related| #child(&related))
                            .transpose()?
                    },
                    Cardinality::Many => quote! {
                        row.many(Self::#selector::<Q>())?
                            .iter()
                            .map(|related| #child(&related))
                            .collect::<::core::result::Result<_, #denial>>()?
                    },
                }
            }
        };
        quote!(#member: #read)
    });
    let root = &projection.root;
    let names = projection.names.iter().map(|name| {
        let (member, read) = (&name.member, name_read(name.optional, name.group));
        // Spanned to the field's type, where a name of another kind is
        // refused; a group name's kind is the field's own.
        if name.group {
            quote_spanned!(name.value.span()=> #member: row.#read()?)
        } else {
            quote_spanned!(name.value.span()=> #member: row.#read::<#root>()?)
        }
    });
    let reads = reads.chain(names);
    quote! {
        /// Reads one row of query `Q`, at the root or under a parent's relation.
        #vis fn project_row<Q>(
            row: &#graph::primary_graph::WorthQueryApplicationProjectionRow<'_, #schema, Q>,
        ) -> ::core::result::Result<Self, #denial>
        where
            Q: #decl::application_query::ApplicationQueryMarkerIdentity<#schema>,
        {
            ::core::result::Result::Ok(Self { #(#reads,)* })
        }
    }
}

/// `name::<Root>()`, or `optional_name::<Root>()` for an `Option<..>` field;
/// `group_name()` / `optional_group_name()` for a group name.
fn name_read(optional: bool, group: bool) -> syn::Ident {
    let read = if group { "group_name" } else { "name" };
    match optional {
        true => format_ident!("optional_{read}"),
        false => format_ident!("{read}"),
    }
}

/// `const PROJECTS_PERSISTENT_NAMES`: whether this row or a related row it
/// nests projects a persistent name, so a read observes names only then.
fn projects_names(input: &DeriveInput, projection: &ProjectionDeclaration) -> TokenStream {
    let (vis, own) = (&input.vis, !projection.names.is_empty());
    let nested = projection.slots.iter().filter_map(|slot| match &slot.kind {
        SlotKind::Relation(Traversal { child, .. }) => Some(child),
        SlotKind::Field { .. } => None,
    });
    quote! {
        /// Whether this projection, or a related row it nests, projects a
        /// persistent name.
        #vis const PROJECTS_PERSISTENT_NAMES: bool =
            #own #(|| <#nested>::PROJECTS_PERSISTENT_NAMES)*;
    }
}

/// The projection of every query whose result binding carries this row.
fn project(input: &DeriveInput, projection: &ProjectionDeclaration) -> TokenStream {
    let (graph, row, schema) = (host(), &input.ident, &projection.schema);
    let bounds = query_bounds(projection);
    quote! {
        impl<Q> #graph::primary_graph::WorthQueryApplicationProjection<#schema, Q> for #row
        where
            #bounds
        {
            const PROJECTS_PERSISTENT_NAMES: bool = <#row>::PROJECTS_PERSISTENT_NAMES;
            fn project(
                row: &#graph::primary_graph::WorthQueryApplicationProjectionRow<'_, #schema, Q>,
            ) -> ::core::result::Result<
                Self,
                #graph::primary_graph::WorthQueryApplicationProjectionDenial,
            > {
                Self::project_row(row)
            }
        }
    }
}
