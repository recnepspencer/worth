//! `#[derive(WorthManagedIndex)]`: generates `ApplicationManagedIndexBinding<Schema>`
//! with checked binary-source-tree defaults and two-layer `const _: ()` hygiene.
mod emit;
mod grammar;
mod hygiene;
#[cfg(test)]
mod tests;

use proc_macro2::TokenStream;
use syn::DeriveInput;

pub(crate) const DERIVE: &str = "WorthManagedIndex";

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let spec = grammar::ManagedIndexDeclaration::parse(input)?;
    Ok(emit::managed_index(input, &spec))
}
