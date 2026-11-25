//! Default expression lowering for `#[derive(WorthQuery)]` definition builders.
use proc_macro2::TokenStream;
use quote::quote;
use syn::meta::ParseNestedMeta;
use syn::{Expr, Ident, Path, Type};

use crate::grammar::value;

pub(super) fn validator(meta: &ParseNestedMeta<'_>) -> syn::Result<Path> {
    value::<Path>(meta).map_err(|error| {
        syn::Error::new(
            error.span(),
            "`validate` takes a function path, such as `validate = rules::check_query`",
        )
    })
}

pub(super) fn managed_index_root_type(schema: &Type, index: &Type) -> Type {
    syn::parse_quote! {
        <#index as ::worth_query_decl::facade::application_query::ApplicationManagedIndexBinding<#schema>>::Entity
    }
}

fn bare_ident(expr: &Expr) -> Option<&Ident> {
    let Expr::Path(path) = expr else {
        return None;
    };
    if path.qself.is_none() && path.path.leading_colon.is_none() && path.path.segments.len() == 1 {
        let seg = &path.path.segments[0];
        if seg.arguments.is_empty() {
            return Some(&seg.ident);
        }
    }
    None
}

pub(super) fn cardinality_expr(expr: Option<&Expr>) -> TokenStream {
    let aq = quote!(::worth_query_decl::facade::application_query);
    match expr {
        None => quote!(#aq::ApplicationQueryCardinality::Many),
        Some(expr) => match bare_ident(expr) {
            Some(ident)
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

pub(super) fn disclosure_expr(expr: Option<&Expr>) -> TokenStream {
    let aq = quote!(::worth_query_decl::facade::application_query);
    match expr {
        None => quote!(#aq::ApplicationQueryDisclosureContract::public()),
        Some(expr) => match bare_ident(expr) {
            Some(ident) if ident == "Public" => {
                quote!(#aq::ApplicationQueryDisclosureContract::public())
            }
            _ => quote!(#expr),
        },
    }
}

pub(super) fn basis_expr(expr: Option<&Expr>) -> TokenStream {
    let aq = quote!(::worth_query_decl::facade::application_query);
    match expr {
        None => quote!(#aq::ApplicationQueryBasisSupport::current_and_pinned()),
        Some(expr) => match bare_ident(expr) {
            Some(ident) if ident == "CurrentAndPinned" => {
                quote!(#aq::ApplicationQueryBasisSupport::current_and_pinned())
            }
            Some(ident) if ident == "CurrentOnly" => {
                quote!(#aq::ApplicationQueryBasisSupport::current_only())
            }
            _ => quote!(#expr),
        },
    }
}

pub(super) fn lanes_expr(expr: Option<&Expr>) -> TokenStream {
    let aq = quote!(::worth_query_decl::facade::application_query);
    match expr {
        None => quote!(#aq::ApplicationQueryLaneEligibility::one_shot()),
        Some(expr) => match bare_ident(expr) {
            Some(ident) if ident == "OneShot" => {
                quote!(#aq::ApplicationQueryLaneEligibility::one_shot())
            }
            Some(ident) if ident == "OneShotAndLive" => {
                quote!(#aq::ApplicationQueryLaneEligibility::one_shot().with_live())
            }
            _ => quote!(#expr),
        },
    }
}
