//! The aspect marker, its `Record` shape and its column selection aliases,
//! written as the `field(..).and(..).build(..)` chain a hand-written record uses.
use super::grammar::{Column, RecordDeclaration};
use crate::case::screaming_snake_case;
use crate::facade_paths::fragment;
use crate::grammar::member_name;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::{DeriveInput, Ident};

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
        // Column reads are spanned to their fields so a non-`Clone` column is
        // refused there; a `Copy` column must not report `clone_on_copy`.
        #[allow(clippy::clone_on_copy)]
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

/// One alias per column, plus zero-string typed selectors on both the row
/// (`Row::select_<field>()`, `Row::<FIELD>`) and its aspect marker
/// (`Aspect::<field>()`, `Aspect::<FIELD>`). An application uses only some of
/// them, so unused items never warn.
fn selections(input: &DeriveInput, record: &RecordDeclaration) -> TokenStream {
    let Some((fragment_type, entity)) = &record.selections else {
        return TokenStream::new();
    };
    let (frag, row, vis, aspect) = (fragment(), &input.ident, &input.vis, &record.aspect);
    let columns: Vec<_> = record
        .columns
        .iter()
        .filter_map(|column| {
            let alias = column.alias.as_ref()?;
            let stored = &column.stored;
            let name = &column.column;
            let member = &column.member;
            let select_fn = format_ident!("select_{}", member.unraw());
            let const_name = Ident::new(&screaming_snake_case(&member_name(member)), member.span());
            Some((alias, stored, name, member, select_fn, const_name))
        })
        .collect();
    let aliases = columns.iter().map(|(alias, stored, ..)| {
        quote_spanned! {stored.span()=>
            #[allow(dead_code)]
            #vis type #alias = #frag::FieldSelection<#fragment_type, #entity, #aspect, #stored>;
        }
    });
    let row_selectors = columns.iter().map(|(alias, _, name, _, select_fn, const_name)| {
        quote! {
            #[allow(dead_code)]
            #vis const #const_name: #alias = <#alias>::at(#name);
            #[allow(dead_code)]
            #vis const fn #select_fn() -> #alias {
                <#alias>::at(#name)
            }
        }
    });
    let aspect_selectors = columns.iter().map(|(alias, _, name, member, _, const_name)| {
        quote! {
            #[allow(dead_code)]
            #vis const #const_name: #alias = <#alias>::at(#name);
            #[allow(dead_code)]
            #vis const fn #member() -> #alias {
                <#alias>::at(#name)
            }
        }
    });
    quote! {
        #(#aliases)*
        impl #row {
            #(#row_selectors)*
        }
        impl #aspect {
            #(#aspect_selectors)*
        }
    }
}
