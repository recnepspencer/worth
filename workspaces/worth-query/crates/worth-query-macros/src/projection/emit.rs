//! Slot markers, selectors, the result shape and the projection, written as
//! the public items a hand-written projection declares with full identifier hygiene.
use super::grammar::{Cardinality, ProjectionDeclaration, Slot, SlotIdentity, SlotKind, Traversal};
use super::hygiene::ProjectionHygiene;
use crate::case::snake_case;
use crate::facade_paths::{declaration, fragment, host};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::DeriveInput;

pub(super) fn projection(input: &DeriveInput, projection: &ProjectionDeclaration) -> TokenStream {
    let (decl, row, vis) = (declaration(), &input.ident, &input.vis);
    let names = ProjectionHygiene::new(input, projection);
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
        SlotKind::Field { .. } => field_selector(input, projection, &module, slot, &names),
        SlotKind::Relation(_) => relation_selector(input, projection, &module, slot, &names),
    });
    let (builder, shape) = (
        result_shape_builder(input, projection, &names),
        result_shape(input, projection, &names),
    );
    let (project_row, project) = (
        project_row(input, projection, &names),
        project(input, projection, &names),
    );
    let projects_names = projects_names(input, projection);
    let self_binding = projection.namespace.as_ref().map(|namespace| {
        quote! {
            impl #decl::application_schema::ApplicationStructuredValueBinding for #row {
                type Value = Self;
                const IDENTITY_NAME: &'static str = #decl::worth_query_identity!(#namespace, result);
                fn validate(_: &Self::Value) -> ::core::result::Result<(), #decl::application_schema::ApplicationValueValidationDenial> {
                    ::core::result::Result::Ok(())
                }
            }
        }
    });
    let (schema, root) = (&projection.schema, &projection.root);
    let (schema_alias, root_alias, row_local, related_local) = (
        &names.schema_alias,
        &names.root_alias,
        &names.row_local,
        &names.related_local,
    );
    let doc = format!("Result slot markers of `{row}`, one per projected field.");
    quote! {
        #[doc = #doc]
        #vis mod #module {
            #(#markers)*
        }
        #(#identities)*
        #self_binding
        const _: () = {
            type #schema_alias = #schema;
            type #root_alias = #root;

            const _: () = {
                #[allow(dead_code, non_snake_case)]
                fn #row_local() {}
                #[allow(dead_code, non_snake_case)]
                fn #related_local() {}

                impl #row {
                    #(#selectors)*
                    #builder
                    #shape
                    #project_row
                    #projects_names
                }
                #project
            };
        };
    }
}

/// `fn selector<Q>()`: the query-bound selector of one field slot.
fn field_selector(
    input: &DeriveInput,
    _projection: &ProjectionDeclaration,
    module: &syn::Ident,
    slot: &Slot,
    names: &ProjectionHygiene,
) -> TokenStream {
    let (query, vis) = (declaration(), &input.vis);
    let (schema_alias, q_ty) = (&names.schema_alias, &names.q_ty);
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
    let doc = format!("The `{}` slot of a query.", output.value());
    quote_spanned! {selection.span()=>
        #[doc = #doc]
        #vis fn #selector<#q_ty>() -> #reference<#q_ty, #module::#marker, #schema_alias, #selection> {
            #reference::new(#output, <#selection>::at(#column))
        }
    }
}

/// `fn selector<Q>()`: the query-bound selector of one traversed relation.
fn relation_selector(
    input: &DeriveInput,
    _projection: &ProjectionDeclaration,
    module: &syn::Ident,
    slot: &Slot,
    names: &ProjectionHygiene,
) -> TokenStream {
    let SlotKind::Relation(traversal) = &slot.kind else {
        unreachable!("relation selectors select relations")
    };
    let (decl, frag, vis) = (declaration(), fragment(), &input.vis);
    let (schema_alias, root_alias, q_ty) = (&names.schema_alias, &names.root_alias, &names.q_ty);
    let relation = &traversal.relation;
    let (selector, output, marker) = (&slot.selector, &slot.output, &slot.marker);
    let to = quote!(<#relation as #frag::Relation>::To);
    let (from, direction, toward) = match &traversal.reverse_from {
        None => (
            quote!(#root_alias),
            quote!(ForwardResultTraversal),
            "forward",
        ),
        Some(from) => (quote!(#from), quote!(ReverseResultTraversal), "reverse"),
    };
    let (count, many) = match traversal.cardinality {
        Cardinality::One => (quote!(ExactlyOneResult), "one"),
        Cardinality::Optional => (quote!(OptionalOneResult), "optional"),
        Cardinality::Many => (quote!(ManyResults), "many"),
    };
    let constructor = format_ident!("{toward}_{many}");
    let query = quote!(#decl::application_query);
    let reference = quote!(#query::ApplicationQueryResultRelationRef);
    let doc = format!("The `{}` relation slot of a query.", output.value());
    quote_spanned! {relation.span()=>
        #[doc = #doc]
        #vis fn #selector<#q_ty>() -> #reference<
            #q_ty, #module::#marker, #schema_alias, #relation, #from, #to, #query::#direction, #query::#count,
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

fn query_bounds(names: &ProjectionHygiene) -> TokenStream {
    let decl = declaration();
    let (schema_alias, q_ty) = (&names.schema_alias, &names.q_ty);
    let marker = quote!(#decl::application_query::ApplicationQueryMarkerIdentity<#schema_alias>);
    quote! {
        #q_ty: #marker,
        <#q_ty as #marker>::ResultBinding:
            #decl::application_schema::ApplicationStructuredValueBinding<Value = Self>,
    }
}

/// `fn result_shape_builder<Q, B>()`: every slot, rooted at the declared
/// entity and identified by `B`; a relation slot nests its child's builder.
fn result_shape_builder(
    input: &DeriveInput,
    projection: &ProjectionDeclaration,
    names: &ProjectionHygiene,
) -> TokenStream {
    let (decl, frag, vis) = (declaration(), fragment(), &input.vis);
    let (schema_alias, root_alias, q_ty, b_ty) = (
        &names.schema_alias,
        &names.root_alias,
        &names.q_ty,
        &names.b_ty,
    );
    let binding = quote!(#decl::application_schema::ApplicationStructuredValueBinding);
    let members = projection.slots.iter().map(|slot| {
        let selector = &slot.selector;
        match &slot.kind {
            SlotKind::Field { optional: true, .. } => {
                quote!(.optional_field(Self::#selector::<#q_ty>()))
            }
            SlotKind::Field { .. } => quote!(.field(Self::#selector::<#q_ty>())),
            SlotKind::Relation(Traversal { child, nested, .. }) => quote_spanned! {child.span()=>
                .relation(
                    Self::#selector::<#q_ty>(),
                    <#child>::result_shape_builder::<#q_ty, #nested>(),
                )
            },
        }
    });
    quote! {
        /// The unbuilt result shape of a query identified by a result binding,
        /// which a parent projection nests under one of its relation slots.
        #vis fn result_shape_builder<#q_ty, #b_ty>() -> #decl::application_query::ApplicationQueryResultShapeBuilder<
            #schema_alias, #q_ty, #root_alias, <#b_ty as #binding>::Value, #b_ty,
        >
        where
            #q_ty: #decl::application_query::ApplicationQueryMarkerIdentity<#schema_alias>,
            #b_ty: #binding,
        {
            #decl::application_query::ApplicationQueryResultShapeBuilder::new(
                #decl::application_schema::ApplicationEntityRef::from_schema_identifier(
                    <#root_alias as #frag::Entity>::NAME,
                ),
            )
            #(#members)*
        }
    }
}

/// `fn result_shape<Q>()`: every slot, rooted at the declared root entity.
fn result_shape(
    input: &DeriveInput,
    _projection: &ProjectionDeclaration,
    names: &ProjectionHygiene,
) -> TokenStream {
    let (decl, vis) = (declaration(), &input.vis);
    let (schema_alias, root_alias, q_ty) = (&names.schema_alias, &names.root_alias, &names.q_ty);
    let bounds = query_bounds(names);
    let marker = quote!(#decl::application_query::ApplicationQueryMarkerIdentity<#schema_alias>);
    quote! {
        /// The result shape of a query, rooted at the declared root entity.
        #vis fn result_shape<#q_ty>() -> #decl::application_query::TypedApplicationQueryResultShape<
            #schema_alias,
            #q_ty,
            #root_alias,
            Self,
            <#q_ty as #marker>::ResultBinding,
        >
        where
            #bounds
        {
            Self::result_shape_builder::<#q_ty, <#q_ty as #marker>::ResultBinding>().build()
        }
    }
}

/// `fn project_row<Q>()` reads each slot into its field; a field whose type
/// differs from its selection's value fails to compile at that field's type.
fn project_row(
    input: &DeriveInput,
    projection: &ProjectionDeclaration,
    names: &ProjectionHygiene,
) -> TokenStream {
    let (decl, graph, vis) = (declaration(), host(), &input.vis);
    let (schema_alias, root_alias, q_ty, row_local, related_local) = (
        &names.schema_alias,
        &names.root_alias,
        &names.q_ty,
        &names.row_local,
        &names.related_local,
    );
    let denial = quote!(#graph::primary_graph::WorthQueryApplicationProjectionDenial);
    let reads = projection.slots.iter().map(|slot| {
        let (member, selector, span) = (&slot.member, &slot.selector, slot.value.span());
        let read = match &slot.kind {
            SlotKind::Field { optional: true, .. } => {
                quote_spanned!(span=> #row_local.optional_field(Self::#selector::<#q_ty>())?)
            }
            SlotKind::Field { .. } => {
                quote_spanned!(span=> #row_local.field(Self::#selector::<#q_ty>())?)
            }
            SlotKind::Relation(Traversal {
                child, cardinality, ..
            }) => {
                let child = quote_spanned!(span=> <#child>::project_row::<#q_ty>);
                match cardinality {
                    Cardinality::One => {
                        quote!(#child(&#row_local.one(Self::#selector::<#q_ty>())?)?)
                    }
                    Cardinality::Optional => quote! {
                        #row_local.optional(Self::#selector::<#q_ty>())?
                            .map(|#related_local| #child(&#related_local))
                            .transpose()?
                    },
                    Cardinality::Many => quote! {
                        #row_local.many(Self::#selector::<#q_ty>())?
                            .iter()
                            .map(|#related_local| #child(&#related_local))
                            .collect::<::core::result::Result<_, #denial>>()?
                    },
                }
            }
        };
        quote!(#member: #read)
    });
    let names_reads = projection.names.iter().map(|name| {
        let (member, read) = (&name.member, name_read(name.optional, name.group));
        if name.group {
            quote_spanned!(name.value.span()=> #member: #row_local.#read()?)
        } else {
            quote_spanned!(name.value.span()=> #member: #row_local.#read::<#root_alias>()?)
        }
    });
    let reads = reads.chain(names_reads);
    quote! {
        /// Reads one row of a query, at the root or under a parent's relation.
        #vis fn project_row<#q_ty>(
            #row_local: &#graph::primary_graph::WorthQueryApplicationProjectionRow<'_, #schema_alias, #q_ty>,
        ) -> ::core::result::Result<Self, #denial>
        where
            #q_ty: #decl::application_query::ApplicationQueryMarkerIdentity<#schema_alias>,
        {
            ::core::result::Result::Ok(Self { #(#reads,)* })
        }
    }
}

fn name_read(optional: bool, group: bool) -> syn::Ident {
    let read = if group { "group_name" } else { "name" };
    if optional {
        format_ident!("optional_{read}")
    } else {
        format_ident!("{read}")
    }
}

fn projects_names(input: &DeriveInput, projection: &ProjectionDeclaration) -> TokenStream {
    let (vis, own) = (&input.vis, !projection.names.is_empty());
    let own_groups = projection.names.iter().any(|name| name.group);
    let nested = projection.slots.iter().filter_map(|slot| match &slot.kind {
        SlotKind::Relation(Traversal { child, .. }) => Some(child),
        SlotKind::Field { .. } => None,
    });
    let nested_groups = nested.clone();
    quote! {
        /// Whether this projection, or a related row it nests, projects a persistent name.
        #vis const PROJECTS_PERSISTENT_NAMES: bool = #own #(|| <#nested>::PROJECTS_PERSISTENT_NAMES)*;
        /// Whether this projection, or a related row it nests, projects a group name.
        #vis const PROJECTS_GROUP_NAMES: bool = #own_groups #(|| <#nested_groups>::PROJECTS_GROUP_NAMES)*;
    }
}

fn project(
    input: &DeriveInput,
    projection: &ProjectionDeclaration,
    names: &ProjectionHygiene,
) -> TokenStream {
    let (graph, row) = (host(), &input.ident);
    let (schema_alias, q_ty, row_local) = (&names.schema_alias, &names.q_ty, &names.row_local);
    let bounds = query_bounds(names);
    let members = projection
        .slots
        .iter()
        .map(|s| &s.member)
        .chain(projection.names.iter().map(|n| &n.member));
    quote! {
        impl #graph::primary_graph::KnownProjectedHeapBytes for #row {
            fn known_heap_bytes(&self) -> usize {
                0usize #(.saturating_add(#graph::primary_graph::KnownProjectedHeapBytes::known_heap_bytes(&self.#members)))*
            }
        }
        impl<#q_ty> #graph::primary_graph::WorthQueryApplicationProjection<#schema_alias, #q_ty> for #row
        where #bounds {
            const PROJECTS_PERSISTENT_NAMES: bool = <#row>::PROJECTS_PERSISTENT_NAMES;
            const PROJECTS_GROUP_NAMES: bool = <#row>::PROJECTS_GROUP_NAMES;
            fn project(#row_local: &#graph::primary_graph::WorthQueryApplicationProjectionRow<'_, #schema_alias, #q_ty>) -> ::core::result::Result<Self, #graph::primary_graph::WorthQueryApplicationProjectionDenial> {
                Self::project_row(#row_local)
            }
            fn projected_heap_bytes(&self) -> usize {
                <#row as #graph::primary_graph::KnownProjectedHeapBytes>::known_heap_bytes(self)
            }
        }
    }
}
