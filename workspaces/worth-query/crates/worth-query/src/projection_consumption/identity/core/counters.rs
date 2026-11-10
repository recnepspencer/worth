use super::super::scope::{consumption_scope_encoder, seal};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::WorthQueryEvidenceTag;

use super::super::super::consumed::ProjectionFactExtractionCounters;
use super::super::super::facts::ProjectionMaterializedFactPostureKind;

pub(crate) fn compose_extraction_counters_digest(
    counters: &ProjectionFactExtractionCounters,
) -> Result<String, IdentityDerivationDenial> {
    seal(
        consumption_scope_encoder("projection_fact_extraction_counters_v1")
            .field_usize(
                WorthQueryEvidenceTag::new("declared"),
                counters.declared_fact_family_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("admitted"),
                counters.admitted_fact_family_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("extracted"),
                counters.extracted_fact_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("row_width"),
                counters.source_row_width_consumed(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("evidence_width"),
                counters.source_evidence_lookup_width(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("authority_reopen"),
                counters.authority_reopen_count(),
            ),
    )
}

pub(crate) fn compose_materialized_fact_posture_digest(
    kind: ProjectionMaterializedFactPostureKind,
    lower_declaration_digest: &str,
    basis_digest: &str,
    support_evidence_digest: &str,
    runtime_origin_digest: Option<&str>,
) -> Result<String, IdentityDerivationDenial> {
    let mut encoder = consumption_scope_encoder("projection_materialized_fact_posture_v1")
        .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
        .field_shape(
            WorthQueryEvidenceTag::new("lower_declaration"),
            lower_declaration_digest,
        )
        .field_shape(WorthQueryEvidenceTag::new("basis"), basis_digest)
        .field_shape(
            WorthQueryEvidenceTag::new("support"),
            support_evidence_digest,
        );
    if let Some(runtime_origin) = runtime_origin_digest {
        encoder = encoder.field_shape(WorthQueryEvidenceTag::new("runtime_origin"), runtime_origin);
    }
    seal(encoder)
}
