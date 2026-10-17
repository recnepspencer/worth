//! Identifier classification uses Rust names, independently of raw spelling.

use proc_macro2::Ident;
use syn::ext::IdentExt;

pub(crate) fn semantic_name(identifier: &Ident) -> String {
    identifier.unraw().to_string()
}
