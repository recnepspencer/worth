use super::super::scope::{compose_sequence_digest, consumption_scope_encoder, seal};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::WorthQueryEvidenceTag;

use super::super::super::facts::ProjectionFactKind;
use super::super::super::source::ProjectionSourceFamily;

pub(crate) fn compose_eligibility_admitted_digest(
    declaration_digest: &str,
    warning_count: usize,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        consumption_scope_encoder("projection_consumption_eligibility_admitted_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("declaration"),
                declaration_digest,
            )
            .field_usize(WorthQueryEvidenceTag::new("warning_count"), warning_count)
            .field_shape(WorthQueryEvidenceTag::new("posture"), "admitted"),
    )?)
}

pub(crate) fn compose_eligibility_deferred_failure_digest(
    declaration_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        consumption_scope_encoder("projection_consumption_eligibility_deferred_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("declaration"),
                declaration_digest,
            )
            .field_shape(WorthQueryEvidenceTag::new("failure"), "deferred"),
    )?)
}

pub(crate) fn compose_eligibility_source_mismatch_failure_digest(
    declaration_digest: &str,
    source_family: ProjectionSourceFamily,
    fact_kind: ProjectionFactKind,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        consumption_scope_encoder("projection_consumption_eligibility_source_mismatch_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("declaration"),
                declaration_digest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("source_family"),
                source_family.as_str(),
            )
            .field_shape(WorthQueryEvidenceTag::new("fact_kind"), fact_kind.as_str())
            .field_shape(WorthQueryEvidenceTag::new("failure"), "source_mismatch"),
    )?)
}

pub(crate) fn compose_eligibility_denied_failure_digest(
    declaration_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        consumption_scope_encoder("projection_consumption_eligibility_denied_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("declaration"),
                declaration_digest,
            )
            .field_shape(WorthQueryEvidenceTag::new("failure"), "visibility_denied"),
    )?)
}

pub(crate) fn compose_eligibility_warning_kinds_digest(
    warnings: &[super::super::super::eligibility::ProjectionConsumptionWarningKind],
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_sequence_digest(
        "projection_consumption_eligibility_warnings_v1",
        "warning_kind",
        warnings.iter().map(|warning| warning.as_str()),
    )?)
}
