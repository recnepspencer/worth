use super::super::scope::{consumption_scope_encoder, seal};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::WorthQueryEvidenceTag;

use super::super::super::contracts::ProjectionContractSupportPosture;
use super::super::super::eligibility::ProjectionConsumptionWarningKind;
use super::super::super::receipt_transitions::ProjectionConsumptionDeferredNeighborFamily;
use super::super::super::source::ProjectionSourceFamily;

pub(crate) fn compose_receipt_integrity_digest(
    fact_set_digest: &str,
    counter_snapshot_digest: &str,
    source_identity: &str,
    materialized_fact_posture_digest: Option<&str>,
) -> Result<String, IdentityDerivationDenial> {
    let mut encoder = consumption_scope_encoder("projection_consumption_receipt_integrity_v1")
        .field_shape(WorthQueryEvidenceTag::new("fact_set"), fact_set_digest)
        .field_shape(
            WorthQueryEvidenceTag::new("counters"),
            counter_snapshot_digest,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("source_identity"),
            source_identity,
        );
    if let Some(posture_digest) = materialized_fact_posture_digest {
        encoder = encoder.field_shape(
            WorthQueryEvidenceTag::new("materialized_fact_posture"),
            posture_digest,
        );
    }
    seal(encoder)
}

pub(crate) fn compose_receipt_digest(
    declaration_digest: &str,
    contract_digest: &str,
    fact_set_digest: &str,
    source_family: ProjectionSourceFamily,
    source_identity: &str,
    support_posture: &ProjectionContractSupportPosture,
    materialized_fact_posture_digest: Option<&str>,
    warning_kinds: &[ProjectionConsumptionWarningKind],
    deferred_neighbors: &[ProjectionConsumptionDeferredNeighborFamily],
) -> Result<String, IdentityDerivationDenial> {
    let mut encoder = consumption_scope_encoder("projection_consumption_receipt_v1")
        .field_shape(
            WorthQueryEvidenceTag::new("declaration"),
            declaration_digest,
        )
        .field_shape(WorthQueryEvidenceTag::new("contract"), contract_digest)
        .field_shape(WorthQueryEvidenceTag::new("fact_set"), fact_set_digest)
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
        );
    if let Some(posture_digest) = materialized_fact_posture_digest {
        encoder = encoder.field_shape(
            WorthQueryEvidenceTag::new("materialized_fact_posture"),
            posture_digest,
        );
    }
    let warnings = warning_kinds.iter().map(|warning| warning.as_str());
    let neighbors = deferred_neighbors.iter().map(|neighbor| neighbor.as_str());
    seal(
        encoder
            .field_value_sequence(WorthQueryEvidenceTag::new("warning_kind"), warnings)
            .field_value_sequence(WorthQueryEvidenceTag::new("deferred_neighbor"), neighbors),
    )
}
