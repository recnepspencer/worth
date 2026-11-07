use super::super::scope::{consumption_scope_encoder, seal};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::WorthQueryEvidenceTag;

use super::super::super::contracts::ProjectionContractSupportPosture;
use super::super::super::eligibility::ProjectionConsumptionWarningKind;
use super::super::super::source::ProjectionSourceFamily;

pub(crate) fn compose_envelope_source_refs_digest(
    receipt_digest: &str,
    fact_set_digest: &str,
    contract_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        consumption_scope_encoder("projection_consumption_envelope_source_refs_v1")
            .field_shape(WorthQueryEvidenceTag::new("receipt"), receipt_digest)
            .field_shape(WorthQueryEvidenceTag::new("fact_set"), fact_set_digest)
            .field_shape(WorthQueryEvidenceTag::new("contract"), contract_digest),
    )?)
}

pub(crate) fn compose_envelope_performance_digest(
    receipt_digest: &str,
    counter_snapshot_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        consumption_scope_encoder("projection_consumption_envelope_performance_v1")
            .field_shape(WorthQueryEvidenceTag::new("receipt"), receipt_digest)
            .field_shape(
                WorthQueryEvidenceTag::new("counters"),
                counter_snapshot_digest,
            ),
    )?)
}

pub(crate) fn compose_envelope_boundary_digest(
    source_family: ProjectionSourceFamily,
    source_identity: &str,
    support_posture: &ProjectionContractSupportPosture,
    warning_kinds: &[ProjectionConsumptionWarningKind],
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        consumption_scope_encoder("projection_consumption_envelope_boundary_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("source_family"),
                source_family.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("source_identity"),
                source_identity,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("support_posture"),
                support_posture.as_str(),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("warning_kind"),
                warning_kinds.iter().map(|warning| warning.as_str()),
            ),
    )?)
}

pub(crate) fn compose_envelope_digest(
    receipt_digest: &str,
    integrity_digest: &str,
    performance_digest: &str,
    boundary_digest: &str,
    transition_rules_digest: &str,
    source_refs_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        consumption_scope_encoder("self_describing_projection_consumption_envelope_v1")
            .field_shape(WorthQueryEvidenceTag::new("receipt"), receipt_digest)
            .field_shape(WorthQueryEvidenceTag::new("integrity"), integrity_digest)
            .field_shape(
                WorthQueryEvidenceTag::new("performance"),
                performance_digest,
            )
            .field_shape(WorthQueryEvidenceTag::new("boundary"), boundary_digest)
            .field_shape(
                WorthQueryEvidenceTag::new("transitions"),
                transition_rules_digest,
            )
            .field_shape(WorthQueryEvidenceTag::new("sources"), source_refs_digest),
    )?)
}
