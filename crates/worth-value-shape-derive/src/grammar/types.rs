use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Type};

pub(crate) fn field_delegate(field: &super::Field, facade: &syn::Path) -> TokenStream {
    match &field.delegate {
        Some(delegate) => quote!(#delegate),
        None => delegate_at(&field.ty, facade).expect("validated field type"),
    }
}

pub(crate) fn delegate(ty: &Type) -> syn::Result<TokenStream> {
    delegate_at(ty, &syn::parse_quote!(::worth_foundational::facade))
}

pub(crate) fn delegate_at(ty: &Type, facade: &syn::Path) -> syn::Result<TokenStream> {
    match ty {
        Type::Path(path) if path.qself.is_none() => {
            let segment = path.path.segments.last().expect("type path");
            let name = segment.ident.to_string();
            match &segment.arguments {
                syn::PathArguments::None => {
                    if name == "usize" {
                        return Ok(quote!(#facade::UsizeDelegate));
                    }
                    if matches!(
                        name.as_str(),
                        "Self" | "f32" | "f64" | "isize" | "u128" | "i128"
                    ) {
                        return Err(syn::Error::new_spanned(ty, "unsupported canonical primitive or Self shorthand; use a concrete derived type, fixed-width supported integer, or bounded owner codec"));
                    }
                    if path
                        .path
                        .segments
                        .iter()
                        .any(|part| !part.arguments.is_empty())
                    {
                        return Err(unsupported(ty));
                    }
                    Ok(quote!(#facade::ValueDelegate<#ty>))
                }
                syn::PathArguments::AngleBracketed(arguments) if arguments.args.len() == 1 => {
                    let Some(syn::GenericArgument::Type(child)) = arguments.args.first() else {
                        return Err(unsupported(ty));
                    };
                    let child = delegate_at(child, facade)?;
                    match name.as_str() {
                        "Vec" => Ok(quote!(#facade::SequenceDelegate<#child>)),
                        "Option" => Ok(quote!(#facade::OptionalDelegate<#child>)),
                        "Box" => Ok(quote!(#facade::BoxDelegate<#child>)),
                        _ => Err(unsupported(ty)),
                    }
                }
                _ => Err(unsupported(ty)),
            }
        }
        Type::Tuple(tuple) if tuple.elems.is_empty() => Ok(quote!(#facade::ValueDelegate<()>)),
        Type::Array(array) => {
            let child = delegate_at(&array.elem, facade)?;
            let syn::Expr::Lit(length) = &array.len else {
                return Err(unsupported(ty));
            };
            let syn::Lit::Int(length) = &length.lit else {
                return Err(unsupported(ty));
            };
            Ok(quote!(#facade::ArrayDelegate<#child, #length>))
        }
        Type::Tuple(tuple) if tuple.elems.len() == 2 => {
            let left = delegate_at(&tuple.elems[0], facade)?;
            let right = delegate_at(&tuple.elems[1], facade)?;
            Ok(quote!(#facade::PairDelegate<#left, #right>))
        }
        _ => Err(unsupported(ty)),
    }
}
fn unsupported(ty: &Type) -> syn::Error {
    syn::Error::new(ty.span(), "unsupported WorthValue shape; use concrete derived values, Vec, Option, Box, literal fixed arrays or an exact two-element tuple; other containers require an admitted owner codec")
}
