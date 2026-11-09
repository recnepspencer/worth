//! Source and performed action DATA share the same checked reconstruction owner.
use super::super::WorthQueryCorrectionDerivationDenial as Denial;
use worth_foundational::facade::{CanonicalDigestId, ValueDecodeBounds};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationEncodedInput, ApplicationRetainedInput, ApplicationRetainedInputDenial,
    },
    application_schema::ApplicationStructuredValueBinding,
};

pub(in crate::domain_computation::primary_graph::product_operation::correction_source) fn recover<
    Binding: ApplicationStructuredValueBinding,
>(
    retained: &ApplicationRetainedInput<Binding>,
    factory: impl FnOnce(
        &[u8],
        CanonicalDigestId,
        ValueDecodeBounds,
    )
        -> Result<ApplicationEncodedInput<Binding>, ApplicationRetainedInputDenial>,
) -> Result<ApplicationEncodedInput<Binding>, Denial> {
    let bytes = retained.recorded_bytes();
    let input = factory(
        bytes,
        CanonicalDigestId::new(*retained.identity()),
        ValueDecodeBounds::wire_bytes(bytes.len()),
    )
    .map_err(Denial::OriginalInputDecode)?;
    if input.identity() != retained.identity() {
        return Err(Denial::OriginalInputDecode(
            ApplicationRetainedInputDenial::IdentityMismatch,
        ));
    }
    Ok(input)
}
#[cfg(test)]
mod tests;
