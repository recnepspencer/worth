//! `#[derive(WorthQueryCorrection)]`: standard recorded-inverse
//! `ApplicationCorrectionBinding` from one unit marker struct.
//!
//! Required keys: `schema = Schema`, either `namespace = NS` (optionally
//! narrowed by `member` / `member = ".."` so the correction shares the
//! application's single namespace with its other derives) or
//! `identity = ".."`, `source = SourceBinding`, `inverse = InverseIntent`,
//! `disclosure = DisclosureBinding`, `derive = path::to::fn`, and either
//! `demand = path::to::fn` or `preimage_fields(expr, ..), preimage_bytes = expr`.
//!
//! Optional keys: `source_body = path::to::fn`, `source_definition = path::to::fn`,
//! `disclosure_parameters = path::to::fn`, `inverse_key = path::to::fn`, and
//! `capability = CapabilityMarker` (defaulting to conventional admission).
use proc_macro2::TokenStream;
use syn::DeriveInput;

mod emit;
mod grammar;
mod hygiene;
#[cfg(test)]
mod tests;

pub(crate) const DERIVE: &str = "WorthQueryCorrection";

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    crate::grammar::unit_struct(input, DERIVE)?;
    let correction = grammar::CorrectionDeclaration::parse(input)?;
    Ok(emit::correction(input, &correction))
}
