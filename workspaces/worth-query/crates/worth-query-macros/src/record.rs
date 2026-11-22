//! `#[derive(WorthQueryRecord)]`: an aspect and its record from one row struct.
//!
//! Struct keys: `bindings = Codecs` (the fragment's binding set), `aspect =
//! Marker` (the aspect marker this derive declares), `name = ".."` (the
//! aspect's persisted name, default the marker name) and, together,
//! `fragment = F, entity = E` to also declare one `FieldSelection` alias per
//! column, named after the column. Field keys: `column = ".."` (default the
//! field name in PascalCase), `equality`, `read_only` and `unique`.
//! `Option<T>` fields are optional columns of `T`.
use proc_macro2::TokenStream;
use syn::DeriveInput;

mod emit;
mod grammar;
#[cfg(test)]
mod tests;

pub(crate) const DERIVE: &str = "WorthQueryRecord";

/// A record shape holds at most this many fields (the declaration's arity).
const MAXIMUM_FIELDS: usize = 16;

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let fields = crate::grammar::named_fields(input, DERIVE)?;
    let record = grammar::RecordDeclaration::parse(input, fields)?;
    Ok(emit::record(input, &record))
}
