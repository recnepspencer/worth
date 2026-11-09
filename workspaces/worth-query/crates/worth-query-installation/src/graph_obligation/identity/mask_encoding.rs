//! A complete explicit field selection is one marker against its exact contract.
use crate::canonical_digest_derivation::InstallationCanonicalIdentityBasis;
use crate::domain_operation::WorthQueryOperationNativeProjectionContract;
use worth_foundational::facade::{AspectShape, CanonicalDigestDerivationDenial};

pub(super) fn encode_mask(
    basis: &mut InstallationCanonicalIdentityBasis,
    prefix: &str,
    projection: &WorthQueryOperationNativeProjectionContract,
) -> Result<(), CanonicalDigestDerivationDenial> {
    if selects_every_declared_field(projection) {
        basis.text(
            format!("{prefix}.mask.complete-fields"),
            "complete-fields-v1",
        )
    } else {
        basis.embedded_basis(&format!("{prefix}.mask"), projection.canonical_mask_basis())
    }
}

// Both inventories are canonical and the mask was admitted against the contract.
// Comparing the keys, rather than only their count, preserves exact selection.
fn selects_every_declared_field(projection: &WorthQueryOperationNativeProjectionContract) -> bool {
    let AspectShape::Struct(shape) = projection.contract().shape() else {
        return false;
    };
    let paths = projection.mask().paths();
    !projection.mask().is_whole_aspect()
        && paths.len() == shape.fields().len()
        && paths
            .iter()
            .zip(shape.fields())
            .all(|(path, field)| path.fields() == std::slice::from_ref(field.key()))
}
