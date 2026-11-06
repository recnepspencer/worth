use super::{
    projection_consumption_forbidden_fallback_audit, projection_consumption_public_boundary_audit,
    projection_consumption_support_matrix, ProjectionConsumptionCertifiedSourceSurface,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::ProjectionConsumptionSupportPosture;
use crate::projection_consumption::ProjectionFactKind;

#[test]
fn query_context_lane_rows_keep_lane_local_traceability_and_structural_proofs() {
    let matrix = projection_consumption_support_matrix();
    let query_context_rows = matrix
        .rows()
        .iter()
        .filter(|row| {
            matches!(
                row.certified_surface(),
                ProjectionConsumptionCertifiedSourceSurface::QueryContextCurrentWithSourceReference
                    | ProjectionConsumptionCertifiedSourceSurface::QueryContextHistoricalWithoutSourceReference
                    | ProjectionConsumptionCertifiedSourceSurface::QueryContextPreviewDerivedWithSourceReference
            )
        })
        .collect::<Vec<_>>();

    assert!(!query_context_rows.is_empty());
    assert!(query_context_rows
        .iter()
        .all(|row| row.certification_lane() == "query_context_consumption"));
    assert!(query_context_rows
        .iter()
        .all(|row| !row.structural_proof().as_str().is_empty()));
}

#[test]
fn hostile_projection_audits_stay_narrow_and_zero_fallback() -> Result<(), IdentityDerivationDenial>
{
    let boundary = projection_consumption_public_boundary_audit()?;
    let fallback = projection_consumption_forbidden_fallback_audit()?;

    assert!(boundary.rows().iter().all(|row| {
        !row.blocked_entrypoint().is_empty()
            && !row.required_artifact().is_empty()
            && !row.enforcement_proof().as_str().is_empty()
    }));
    assert_eq!(fallback.total_occurrence_count(), 0);
    assert!(fallback
        .rows()
        .iter()
        .all(|row| row.occurrence_count() == 0));

    Ok(())
}

#[test]
fn query_context_lane_keeps_historical_identity_denial_distinct_from_current_and_preview() {
    let matrix = projection_consumption_support_matrix();
    let historical_identity_row = matrix
        .rows()
        .iter()
        .find(|row| {
            row.certified_surface()
                == ProjectionConsumptionCertifiedSourceSurface::QueryContextHistoricalWithoutSourceReference
                && row.fact_kind() == ProjectionFactKind::EntityIdentity
        })
        .expect("historical query-context identity row should exist");
    let current_identity_row = matrix
        .rows()
        .iter()
        .find(|row| {
            row.certified_surface()
                == ProjectionConsumptionCertifiedSourceSurface::QueryContextCurrentWithSourceReference
                && row.fact_kind() == ProjectionFactKind::EntityIdentity
        })
        .expect("current query-context identity row should exist");
    let preview_identity_row = matrix
        .rows()
        .iter()
        .find(|row| {
            row.certified_surface()
                == ProjectionConsumptionCertifiedSourceSurface::QueryContextPreviewDerivedWithSourceReference
                && row.fact_kind() == ProjectionFactKind::EntityIdentity
        })
        .expect("preview query-context identity row should exist");
    let current_source_reference_row = matrix
        .rows()
        .iter()
        .find(|row| {
            row.certified_surface()
                == ProjectionConsumptionCertifiedSourceSurface::QueryContextCurrentWithSourceReference
                && row.fact_kind() == ProjectionFactKind::SourceReference
        })
        .expect("current query-context source-reference row should exist");
    let historical_source_reference_row = matrix
        .rows()
        .iter()
        .find(|row| {
            row.certified_surface()
                == ProjectionConsumptionCertifiedSourceSurface::QueryContextHistoricalWithoutSourceReference
                && row.fact_kind() == ProjectionFactKind::SourceReference
        })
        .expect("historical query-context source-reference row should exist");
    let preview_source_reference_row = matrix
        .rows()
        .iter()
        .find(|row| {
            row.certified_surface()
                == ProjectionConsumptionCertifiedSourceSurface::QueryContextPreviewDerivedWithSourceReference
                && row.fact_kind() == ProjectionFactKind::SourceReference
        })
        .expect("preview query-context source-reference row should exist");

    assert!(matches!(
        historical_identity_row.posture(),
        ProjectionConsumptionSupportPosture::SourceMismatch
    ));
    assert!(matches!(
        current_identity_row.posture(),
        ProjectionConsumptionSupportPosture::Admitted
    ));
    assert!(matches!(
        preview_identity_row.posture(),
        ProjectionConsumptionSupportPosture::SourceMismatch
    ));
    assert_eq!(
        historical_identity_row.admission_rule(),
        "query_context_historical_identity_is_source_mismatch"
    );
    assert_eq!(
        current_identity_row.admission_rule(),
        "support_row_derived_from_executable_support_for_kind"
    );
    assert_eq!(
        preview_identity_row.admission_rule(),
        "support_row_derived_from_executable_support_for_kind"
    );
    assert!(matches!(
        current_source_reference_row.posture(),
        ProjectionConsumptionSupportPosture::AdmittedWithWarnings(_)
    ));
    assert!(matches!(
        historical_source_reference_row.posture(),
        ProjectionConsumptionSupportPosture::SourceMismatch
    ));
    assert!(matches!(
        preview_source_reference_row.posture(),
        ProjectionConsumptionSupportPosture::AdmittedWithWarnings(_)
    ));
    assert_ne!(
        current_source_reference_row.support_digest(),
        preview_source_reference_row.support_digest(),
        "preview-derived query-context warnings must not collapse into current-row warnings"
    );
}
