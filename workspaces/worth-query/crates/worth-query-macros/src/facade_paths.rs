//! The audience facades every expansion names. Derived code reaches Query
//! only through the paths an application itself imports.
use proc_macro2::TokenStream;
use quote::quote;

/// `worth_query_decl::facade`: declaration traits and macros.
pub(crate) fn declaration() -> TokenStream {
    quote!(::worth_query_decl::facade)
}

/// `worth_query_decl::facade::application_schema::fragment`.
pub(crate) fn fragment() -> TokenStream {
    quote!(::worth_query_decl::facade::application_schema::fragment)
}

/// `worth_query_host::facade`: the projection trait and its row.
pub(crate) fn host() -> TokenStream {
    quote!(::worth_query_host::facade)
}
