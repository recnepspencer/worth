use super::super::scope::{consumption_scope_encoder, seal};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::WorthQueryEvidenceTag;

use super::super::super::contracts::BoundProjectionFactFamily;
use super::super::super::facts::ProjectionFactRequest;
use super::super::super::source::ProjectionSourceReferenceIdentity;
use super::super::super::DeclaredNativeFactContract;

pub(crate) fn compose_fact_request_entry_digest(
    request: &ProjectionFactRequest,
    native_contract: Option<&DeclaredNativeFactContract>,
) -> Result<String, IdentityDerivationDenial> {
    let mut encoder = consumption_scope_encoder("projection_fact_request_entry_v1")
        .field_shape(WorthQueryEvidenceTag::new("kind"), request.kind().as_str());
    if let Some(field) = request.field_path() {
        encoder = encoder.field_shape(
            WorthQueryEvidenceTag::new("field"),
            field.terminal_projection_for_boundary(),
        );
    }
    if let Some(native_contract) = native_contract {
        encoder = encoder.field_shape(
            WorthQueryEvidenceTag::new("native_contract"),
            native_contract.canonical_contract_material(),
        );
    }
    Ok(seal(encoder)?)
}

pub(crate) fn compose_projection_source_reference_entry_digest(
    identity: &ProjectionSourceReferenceIdentity,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        consumption_scope_encoder("projection_source_reference_entry_v1")
            .field_shape(WorthQueryEvidenceTag::new("label"), identity.label())
            .field_shape(WorthQueryEvidenceTag::new("identity"), identity.identity()),
    )?)
}

pub(crate) fn compose_bound_fact_family_entry_digest(
    fact_family: &BoundProjectionFactFamily,
) -> Result<String, IdentityDerivationDenial> {
    let mut encoder = consumption_scope_encoder("projection_bound_fact_family_entry_v1")
        .field_shape(
            WorthQueryEvidenceTag::new("kind"),
            fact_family.kind().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("support_posture"),
            fact_family.support_posture().as_str(),
        );
    if let Some(field_path) = fact_family.field_path() {
        encoder = encoder.field_shape(
            WorthQueryEvidenceTag::new("field"),
            field_path.terminal_projection_for_boundary(),
        );
    }
    if let Some(native_contract) = fact_family.native_contract() {
        encoder = encoder.field_shape(
            WorthQueryEvidenceTag::new("native_contract"),
            native_contract.canonical_contract_material(),
        );
    }
    Ok(seal(encoder)?)
}
