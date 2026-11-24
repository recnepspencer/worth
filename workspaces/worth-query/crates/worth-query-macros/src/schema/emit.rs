//! Each role lowers to the public schema traits a hand-written marker implements.
use super::grammar::{Reference, RelationLaws, Role, SchemaMarker};
use crate::facade_paths::{declaration, fragment};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitStr, Type};

pub(super) fn marker(input: &DeriveInput, marker: &SchemaMarker) -> TokenStream {
    let name = marker
        .name
        .clone()
        .unwrap_or_else(|| LitStr::new(&input.ident.to_string(), input.ident.span()));
    match &marker.role {
        Role::Root {
            owner,
            bindings,
            members,
        } => root(input, &name, owner.as_ref(), bindings.as_ref(), members),
        Role::Fragment { bindings, members } => {
            fragment_impl(input, &name, bindings.as_ref(), members)
        }
        Role::Entity {
            aspects,
            relations,
            reference,
        } => entity(input, &name, aspects, relations, reference.as_ref()),
        Role::Relation {
            to,
            reference,
            laws,
        } => relation(input, &name, to, reference.as_ref(), laws),
    }
}

fn root(
    input: &DeriveInput,
    name: &LitStr,
    owner: Option<&LitStr>,
    bindings: Option<&Type>,
    members: &[Type],
) -> TokenStream {
    let (decl, frag, ident) = (declaration(), fragment(), &input.ident);
    let schema = quote!(#decl::application_schema);
    let fragment = fragment_impl(input, name, bindings, members);
    let owner = owner.map(|owner| quote!(const OWNER: &'static str = #owner;));
    quote! {
        #fragment
        impl #schema::ApplicationSchema for #ident {
            #owner
            const NAME: &'static str = <Self as #frag::Fragment>::NAME;
            fn declaration() -> ::core::result::Result<
                #schema::ApplicationSchemaDeclaration<Self>,
                #schema::ApplicationSchemaDeclarationDenial,
            > {
                #frag::declare(Self)
            }
            fn visit_correction<
                Visitor: #decl::application_aftermath::ApplicationCorrectionVisitor<Self>,
            >(
                source_binding: ::core::any::TypeId,
                visitor: Visitor,
            ) -> Visitor::Output {
                #frag::visit_correction(Self, source_binding, visitor)
            }
        }
        impl #schema::ApplicationSchemaComposition for #ident {
            type Contributions = <Self as #frag::Fragment>::Members;
        }
    }
}

fn fragment_impl(
    input: &DeriveInput,
    name: &LitStr,
    bindings: Option<&Type>,
    members: &[Type],
) -> TokenStream {
    let (frag, ident) = (fragment(), &input.ident);
    let bindings = bindings.map_or_else(|| quote!(#frag::NoBindings), |ty| quote!(#ty));
    quote! {
        impl #frag::Fragment for #ident {
            const NAME: &'static str = #name;
            type Bindings = #bindings;
            type Members = (#(#members,)*);
        }
    }
}

fn entity(
    input: &DeriveInput,
    name: &LitStr,
    aspects: &[Type],
    relations: &[Type],
    reference: Option<&Reference>,
) -> TokenStream {
    let (decl, frag, ident, vis) = (declaration(), fragment(), &input.ident, &input.vis);
    let reference = reference.map(|Reference { schema, fragment }| {
        quote! {
            impl #ident {
                #vis fn reference() -> #decl::application_schema::ApplicationEntityRef<#schema, Self> {
                    #frag::entity(#fragment, Self)
                }
            }
        }
    });
    quote! {
        impl #frag::Entity for #ident {
            const NAME: &'static str = #name;
            type Aspects = (#(#aspects,)*);
            type Relations = (#(#relations,)*);
        }
        #reference
    }
}

pub(crate) fn relation(
    input: &DeriveInput,
    name: &LitStr,
    to: &Type,
    reference: Option<&(syn::Path, Reference)>,
    laws: &RelationLaws,
) -> TokenStream {
    let (decl, frag, ident, vis) = (declaration(), fragment(), &input.ident, &input.vis);
    let schema = quote!(#decl::application_schema);
    let cardinality = laws
        .cardinality
        .as_ref()
        .map(|value| quote!(const CARDINALITY: #schema::ApplicationRelationCardinality = #value;));
    let acyclic = laws.acyclic.then(|| quote!(const ACYCLIC: bool = true;));
    let self_edges = laws
        .self_edges
        .as_ref()
        .map(|value| quote!(const SELF_EDGES: bool = #value;));
    let unique = laws.unique.as_ref().map(|value| {
        quote!(const UNIQUE: ::core::option::Option<#schema::ApplicationRelationUniquenessScope> = ::core::option::Option::Some(#value);)
    });
    let deletion = laws
        .deletion
        .as_ref()
        .map(|value| quote!(const DELETION: #schema::ApplicationRelationDeletionPolicy = #value;));
    let cross_context = laws.cross_context.as_ref().map(|value| {
        quote!(const CROSS_CONTEXT: #schema::ApplicationRelationCrossContextPolicy = #value;)
    });
    let symmetric = laws.symmetric.then(|| quote!(const SYMMETRIC: bool = true;));
    let orbit = laws.orbit.then(|| quote!(const ORBIT: bool = true;));
    let same_partition = laws
        .same_partition_endpoints
        .then(|| quote!(const SAME_PARTITION_ENDPOINTS: bool = true;));
    let reaches_at_least = laws.reaches_at_least.as_ref().map(|value| {
        quote!(const REACHES_AT_LEAST: ::core::option::Option<#schema::ApplicationRelationMinimumReachability> = ::core::option::Option::Some(#value);)
    });
    let reference = reference.map(|(from, Reference { schema, fragment })| {
        quote! {
            impl #ident {
                #vis fn reference(
                ) -> #decl::application_schema::ApplicationRelationRef<#schema, Self, #from, #to> {
                    #frag::relation(#fragment, #from, Self)
                }
            }
        }
    });
    quote! {
        impl #frag::Relation for #ident {
            const NAME: &'static str = #name;
            type To = #to;
            #cardinality
            #acyclic
            #self_edges
            #unique
            #deletion
            #cross_context
            #symmetric
            #orbit
            #same_partition
            #reaches_at_least
        }
        #reference
    }
}
