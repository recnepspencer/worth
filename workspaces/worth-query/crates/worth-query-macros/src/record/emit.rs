//! The aspect marker, its `Record` shape and its column selection aliases,
//! written as the `field(..).and(..).build(..)` chain a hand-written record uses.
use super::grammar::{Column, RecordDeclaration};
use crate::facade_paths::fragment;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::DeriveInput;

pub(super) fn record(input: &DeriveInput, record: &RecordDeclaration) -> TokenStream {
    let (frag, row, vis) = (fragment(), &input.ident, &input.vis);
    let RecordDeclaration {
        bindings,
        aspect,
        aspect_name,
        ..
    } = record;
    let shape = shape(row, &record.columns);
    let members = record.columns.iter().map(|column| &column.member);
    let constructor_members = members.clone();
    let selections = selections(input, record);
    quote! {
        #vis struct #aspect;
        impl #frag::Aspect for #aspect {
            const NAME: &'static str = #aspect_name;
            type Row = #row;
        }
        impl #frag::Record<#row> for #bindings {
            fn record() -> impl #frag::Shape<#row, Self> {
                #shape.build(|#(#members,)* _: &mut <Self as #frag::BindingSet>::Work<'_>| {
                    ::core::result::Result::Ok::<_, ::core::convert::Infallible>(
                        #row { #(#constructor_members,)* },
                    )
                })
            }
        }
        #selections
    }
}

/// `field(first, ..)` followed by one `.and(next, ..)` per further column,
/// each with its declared postures.
fn shape(row: &syn::Ident, columns: &[Column]) -> TokenStream {
    let frag = fragment();
    columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            let Column {
                member,
                column: name,
                ..
            } = column;
            let read = quote_spanned!(member.span()=>
                #name, |row: &#row| ::core::clone::Clone::clone(&row.#member)
            );
            let start = if index == 0 {
                quote!(#frag::field(#read))
            } else {
                quote!(.and(#read))
            };
            let equality = column.equality.then(|| quote!(.equality_queryable(true)));
            let read_only = column.read_only.then(|| quote!(.read_only()));
            let unique = column.unique.then(|| quote!(.unique()));
            quote!(#start #equality #read_only #unique)
        })
        .collect()
}

/// One alias per column. An application uses only some of them, so an unused
/// alias is not a warning in the application's code.
fn selections(input: &DeriveInput, record: &RecordDeclaration) -> TokenStream {
    let Some((fragment_type, entity)) = &record.selections else {
        return TokenStream::new();
    };
    let (frag, vis, aspect) = (fragment(), &input.vis, &record.aspect);
    record
        .columns
        .iter()
        .filter_map(|column| {
            let (alias, stored) = (column.alias.as_ref()?, &column.stored);
            Some(quote_spanned! {column.stored.span()=>
                #[allow(dead_code)]
                #vis type #alias = #frag::FieldSelection<#fragment_type, #entity, #aspect, #stored>;
            })
        })
        .collect()
}
