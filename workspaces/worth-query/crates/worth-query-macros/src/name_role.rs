//! `#[derive(WorthQueryNameRole)]`: a name scheme's roles from one enum.
//!
//! A unit variant is one role; a variant carrying one `u32` is one role
//! repeated by a numeric index. Variant keys: `#[worth_query(name = "..")]`
//! (the role's persisted name, default the variant name) and
//! `#[depends_on(Variant, ..)]` (roles whose same-occurrence names this
//! role's births may reference). An unknown, repeated or self dependency and
//! any dependency cycle are refused at the offending name.
//!
//! The derive also orders the enum (`Ord` and `PartialOrd`) by canonical role
//! order and then by index, so the enum derives neither itself.
use proc_macro2::TokenStream;
use syn::DeriveInput;

mod emit;
mod grammar;
#[cfg(test)]
mod tests;

pub(crate) const DERIVE: &str = "WorthQueryNameRole";

/// The attribute naming the roles a variant depends on.
pub(crate) const DEPENDS_ON: &str = "depends_on";

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let roles = grammar::RoleDeclaration::parse(input)?;
    Ok(emit::roles(input, &roles))
}
