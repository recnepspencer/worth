use crate::harness::certification::{
    milestone_nine_five_requirements, unmet_required_assertion_classes, HostileExpectation,
    ParityAnchor, RequiredAssertionClass,
};
use crate::saved_query::SavedQueryTemporalAsyncSurfacePosture;

use super::bundle::MilestoneNineFiveHostileMatrixAdapter;

#[test]
fn milestone_nine_five_rows_enforce_required_assertion_classes() {
    let matrix =
        MilestoneNineFiveHostileMatrixAdapter::debt_close_hostile_certification_matrix_test();
    let mut covered = Vec::new();

    for row in &matrix.rows {
        let control = row.control_lane.semantic_signature();
        let hostile = row.hostile_lane.semantic_signature();
        let parity = row.parity_lane.semantic_signature();
        match row.hostile_expectation {
            HostileExpectation::EquivalentToControl => {
                assert_eq!(control, hostile, "row '{}'", row.row_name);
                covered.push(RequiredAssertionClass::Equality);
            }
            HostileExpectation::DistinctFromControl => {
                assert_ne!(control, hostile, "row '{}'", row.row_name);
                covered.push(RequiredAssertionClass::Inequality);
            }
        }
        match row.parity_anchor {
            ParityAnchor::Control => assert_eq!(parity, control, "row '{}'", row.row_name),
            ParityAnchor::Hostile => assert_eq!(parity, hostile, "row '{}'", row.row_name),
        }
    }

    for row in &matrix.rejection_rows {
        covered.push(RequiredAssertionClass::TypedFailure);
        assert!(
            row.hostile_lane.counter_snapshot.contains("residue=0"),
            "row '{}' must prove zero-residue denial",
            row.row_name
        );
        covered.push(RequiredAssertionClass::ZeroResidue);
    }

    covered.sort();
    covered.dedup();
    let missing = unmet_required_assertion_classes(
        &covered,
        milestone_nine_five_requirements().required_assertion_classes,
    );
    assert!(missing.is_empty(), "missing assertion classes: {missing:?}");
}

#[test]
fn milestone_nine_five_parity_rows_keep_canonical_identity_but_not_composition_authority() {
    let matrix =
        MilestoneNineFiveHostileMatrixAdapter::debt_close_hostile_certification_matrix_test();

    for row_name in [
        "named-scope-table-retained-derived-parity",
        "template-detail-live-artifact-parity",
        "public-bridge-bootstrap-fixed-under-template-composition",
    ] {
        let row = matrix
            .rows
            .iter()
            .find(|row| row.row_name == row_name)
            .unwrap();
        assert_eq!(
            row.control_lane.canonical_query_digest,
            row.hostile_lane.canonical_query_digest
        );
        assert_eq!(
            row.control_lane.canonical_result_shape_digest,
            row.hostile_lane.canonical_result_shape_digest
        );
        assert_eq!(
            row.control_lane.semantic_signature(),
            row.hostile_lane.semantic_signature()
        );
        assert_ne!(
            row.control_lane.composition_authority_digest,
            row.hostile_lane.composition_authority_digest
        );
        assert_ne!(
            row.control_lane.artifact_signature(),
            row.hostile_lane.artifact_signature()
        );
    }
}

#[test]
fn milestone_nine_five_projection_and_reuse_rows_shift_only_the_intended_surface() {
    let matrix =
        MilestoneNineFiveHostileMatrixAdapter::debt_close_hostile_certification_matrix_test();
    let projection = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "retained-vs-live-projection-contract-distinctness")
        .unwrap();
    assert_eq!(
        projection.control_lane.canonical_query_digest,
        projection.hostile_lane.canonical_query_digest
    );
    assert_eq!(
        projection.control_lane.view_shape_digest,
        projection.hostile_lane.view_shape_digest
    );
    assert_eq!(
        projection.control_lane.reuse_matrix_digest,
        projection.hostile_lane.reuse_matrix_digest
    );
    assert_ne!(
        projection.control_lane.projection_contract_digest,
        projection.hostile_lane.projection_contract_digest
    );

    let grouped = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "grouped-ordinary-vs-preserved-reuse-distinctness")
        .unwrap();
    assert_eq!(
        grouped.control_lane.canonical_query_digest,
        grouped.hostile_lane.canonical_query_digest
    );
    assert_eq!(
        grouped.control_lane.view_shape_digest,
        grouped.hostile_lane.view_shape_digest
    );
    assert_ne!(
        grouped.control_lane.reuse_matrix_digest,
        grouped.hostile_lane.reuse_matrix_digest
    );
    assert_ne!(
        grouped.control_lane.temporal_async_surface_posture,
        grouped.hostile_lane.temporal_async_surface_posture
    );
    assert_ne!(
        grouped.control_lane.saved_query_digest,
        grouped.hostile_lane.saved_query_digest
    );
}

#[test]
fn milestone_nine_five_rejection_rows_preserve_typed_temporal_async_targets() {
    let matrix =
        MilestoneNineFiveHostileMatrixAdapter::debt_close_hostile_certification_matrix_test();
    let grouped = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "grouped-preserved-reuse-basis-erasure-denied")
        .unwrap();
    assert_eq!(
        grouped.hostile_lane.temporal_async_surface_posture,
        SavedQueryTemporalAsyncSurfacePosture::OrdinaryOnly.as_str()
    );

    let inspector = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "inspector-target-preserved-reuse-downcast-denied")
        .unwrap();
    assert_eq!(
        inspector.hostile_lane.temporal_async_surface_posture,
        SavedQueryTemporalAsyncSurfacePosture::VisibleButDeferred.as_str()
    );
}
