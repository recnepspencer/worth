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
    recover_bytes(retained.recorded_bytes(), *retained.identity(), factory)
}
pub(in crate::domain_computation::primary_graph::product_operation::correction_source) fn recover_bytes<
    Binding: ApplicationStructuredValueBinding,
>(
    bytes: &[u8],
    identity: [u8; 32],
    factory: impl FnOnce(
        &[u8],
        CanonicalDigestId,
        ValueDecodeBounds,
    )
        -> Result<ApplicationEncodedInput<Binding>, ApplicationRetainedInputDenial>,
) -> Result<ApplicationEncodedInput<Binding>, Denial> {
    let input = factory(
        bytes,
        CanonicalDigestId::new(identity),
        ValueDecodeBounds::wire_bytes(bytes.len()),
    )
    .map_err(Denial::OriginalInputDecode)?;
    if input.identity() != &identity {
        return Err(Denial::OriginalInputDecode(
            ApplicationRetainedInputDenial::IdentityMismatch,
        ));
    }
    Ok(input)
}
#[cfg(test)]
mod tests;
