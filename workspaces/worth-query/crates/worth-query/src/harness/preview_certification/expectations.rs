use super::model::{
    PreviewCertificationMatrix, PreviewCertificationRow, PreviewLaneEvaluationClass,
    PreviewRejectionRow,
};
use super::row_catalog::{
    PreviewCanonicalRowSpec, PreviewRejectionRowSpec, PREVIEW_CANONICAL_ROW_SPECS,
    PREVIEW_REJECTION_ROW_SPECS,
};
use crate::harness::certification::HostileExpectation;

pub(super) fn assert_matrix_meets_required_rows(matrix: &PreviewCertificationMatrix) {
    assert!(matrix
        .rows
        .iter()
        .all(|row| row.control_lane.has_required_outputs()));
    assert!(matrix
        .rejection_rows
        .iter()
        .all(|row| row.hostile_lane.has_required_outputs()));
    assert!(matrix.rows.iter().all(|row| row
        .control_lane
        .counters
        .preview_lifecycle_rediscovery_count()
        == 0));
    assert!(matrix.rows.iter().all(|row| row
        .control_lane
        .counters
        .preview_executor_rediscovery_count()
        == 0));
    assert_canonical_row_expectations(matrix);
    assert_rejection_row_expectations(matrix);
}

fn assert_canonical_row_expectations(matrix: &PreviewCertificationMatrix) {
    for spec in PREVIEW_CANONICAL_ROW_SPECS {
        let row = canonical_row_for_expectation(matrix, spec.row_name);
        assert_eq!(row.perturbation_class, spec.perturbation_class);
        assert_eq!(row.hostile_expectation, spec.hostile_expectation);
        if let Some(hostile_eval) = spec.hostile_evaluation_class {
            assert_eq!(row.hostile_lane.evaluation_class, hostile_eval);
        }
        assert_canonical_evaluation_expectation(row, spec);
        assert_canonical_counter_expectations(row, spec.row_name);
    }
}

fn assert_canonical_evaluation_expectation(
    row: &PreviewCertificationRow,
    spec: &PreviewCanonicalRowSpec,
) {
    if spec.hostile_expectation == HostileExpectation::DistinctFromControl {
        assert_ne!(
            row.control_lane.binding_digest,
            row.hostile_lane.binding_digest
        );
        if spec.row_name != "preview-live-drift-explicitness" {
            assert_ne!(
                row.control_lane.evaluation_class,
                row.hostile_lane.evaluation_class
            );
        }
    } else if !matches!(
        spec.row_name,
        "preview-promotion-comparison-parity"
            | "preview-comparison-shape-proof-width"
            | "preview-live-admission-parity"
            | "preview-workflow-foundation-admission"
            | "preview-workflow-foundation-no-rescan"
    ) {
        assert_eq!(
            row.control_lane.evaluation_class,
            row.hostile_lane.evaluation_class
        );
    }
    match spec.row_name {
        "preview-promotion-comparison-parity"
        | "preview-comparison-shape-proof-width"
        | "preview-live-admission-parity"
        | "preview-live-drift-explicitness"
        | "preview-workflow-foundation-admission"
        | "preview-workflow-foundation-no-rescan"
        | "preview-work-avoided-counter-parity" => {
            assert_eq!(
                row.control_lane.evaluation_class,
                PreviewLaneEvaluationClass::PromotionEligible
            );
            assert!(!row.control_lane.workflow_foundation_digest.is_empty());
        }
        _ => assert_eq!(
            row.control_lane.evaluation_class,
            PreviewLaneEvaluationClass::ReadOnly
        ),
    }
}

fn assert_canonical_counter_expectations(row: &PreviewCertificationRow, row_name: &str) {
    if row_name == "preview-work-avoided-counter-parity" {
        for lane in [&row.control_lane, &row.hostile_lane, &row.parity_lane] {
            assert_eq!(
                lane.execution_counters
                    .preview_work_avoided_by_explicit_basis_count(),
                1
            );
        }
    }
    if matches!(
        row_name,
        "preview-promotion-comparison-parity" | "preview-comparison-shape-proof-width"
    ) {
        assert!(row.control_lane.promotion_parity_digest.is_some());
        assert!(row.control_lane.comparison_counters.is_some());
    }
    if matches!(
        row_name,
        "preview-live-admission-parity" | "preview-live-drift-explicitness"
    ) {
        assert!(row.control_lane.preview_live_digest.is_some());
        assert!(row.control_lane.preview_live_counters.is_some());
    }
    if row_name == "preview-live-drift-explicitness" {
        assert_ne!(
            row.control_lane.preview_live_digest,
            row.hostile_lane.preview_live_digest
        );
        assert_eq!(
            row.hostile_lane
                .preview_live_counters
                .as_ref()
                .expect("rebind lane should retain preview-live counters")
                .preview_live_rebind_available_count(),
            1
        );
    }
}

fn assert_rejection_row_expectations(matrix: &PreviewCertificationMatrix) {
    for spec in PREVIEW_REJECTION_ROW_SPECS {
        let row = rejection_row_for_expectation(matrix, spec.row_name);
        assert_eq!(row.perturbation_class, spec.perturbation_class);
        assert_eq!(row.hostile_lane.failure_class, spec.failure_class);
        assert!(
            row.hostile_lane.counters.is_some()
                || row.hostile_lane.execution_counters.is_some()
                || row.hostile_lane.comparison_counters.is_some()
                || row.hostile_lane.preview_live_counters.is_some()
        );
        assert_rejection_counter_expectations(row, spec);
    }
}

fn assert_rejection_counter_expectations(
    row: &PreviewRejectionRow,
    spec: &PreviewRejectionRowSpec,
) {
    match spec.row_name {
        "preview-broad-fallback-forbidden" => assert_eq!(
            row.hostile_lane
                .counters
                .as_ref()
                .expect("broad-fallback denial should retain binding counters")
                .preview_broad_fallback_denial_count(),
            1
        ),
        "read-only-preview-writeback-foundation-forbidden" => assert_eq!(
            row.hostile_lane
                .execution_counters
                .as_ref()
                .expect("workflow-foundation authority denial should retain execution counters",)
                .preview_workflow_foundation_denial_count(),
            1
        ),
        "preview-live-drift-denied" => assert_eq!(
            row.hostile_lane
                .preview_live_counters
                .as_ref()
                .expect("preview-live drift denial should retain live counters")
                .preview_live_drift_denial_count(),
            1
        ),
        "preview-live-broad-fallback-forbidden" => assert_eq!(
            row.hostile_lane
                .preview_live_counters
                .as_ref()
                .expect("preview-live broad fallback should retain live counters")
                .preview_live_broad_fallback_denial_count(),
            1
        ),
        _ => {}
    }
}

fn canonical_row_for_expectation<'a>(
    matrix: &'a PreviewCertificationMatrix,
    row_name: &str,
) -> &'a PreviewCertificationRow {
    matrix
        .rows
        .iter()
        .find(|row| row.row_name == row_name)
        .unwrap_or_else(|| panic!("missing preview canonical row {row_name}"))
}

fn rejection_row_for_expectation<'a>(
    matrix: &'a PreviewCertificationMatrix,
    row_name: &str,
) -> &'a PreviewRejectionRow {
    matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == row_name)
        .unwrap_or_else(|| panic!("missing preview rejection row {row_name}"))
}
