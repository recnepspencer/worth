use super::{
    AspectApiFinalizationCertificationAdapter,
    ASPECT_API_FINALIZATION_REQUIRED_CANONICAL_ROW_NAMES,
    ASPECT_API_FINALIZATION_REQUIRED_REJECTION_ROW_NAMES,
};
use crate::harness::certification::{
    contains_row, unmet_required_assertion_classes, HostileExpectation, ParityAnchor,
    RequiredAssertionClass,
};

#[test]
fn aspect_api_finalization_certification_adapter_emits_named_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let artifact =
        AspectApiFinalizationCertificationAdapter::public_aspect_api_finalization_artifact(
            request_execution,
        );

    assert_eq!(artifact.suite_name, "Public Aspect API Finalization Test");
    assert!(!artifact.certification_bundle_digest.is_empty());
    assert!(!artifact.coverage_matrix_digest.is_empty());
}

#[test]
fn aspect_api_finalization_matrix_covers_required_rows() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = AspectApiFinalizationCertificationAdapter::public_aspect_api_finalization_test(
        request_execution,
    );

    for row_name in ASPECT_API_FINALIZATION_REQUIRED_CANONICAL_ROW_NAMES {
        assert!(contains_row(&matrix, row_name), "missing {row_name}");
    }
    for row_name in ASPECT_API_FINALIZATION_REQUIRED_REJECTION_ROW_NAMES {
        assert!(contains_row(&matrix, row_name), "missing {row_name}");
    }
}

#[test]
fn aspect_api_finalization_rows_have_required_outputs() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = AspectApiFinalizationCertificationAdapter::public_aspect_api_finalization_test(
        request_execution,
    );

    for row in &matrix.rows {
        assert!(
            row.control_lane.has_required_outputs(),
            "row '{}'",
            row.row_name
        );
        assert!(
            row.hostile_lane.has_required_outputs(),
            "row '{}'",
            row.row_name
        );
        assert!(
            row.parity_lane.has_required_outputs(),
            "row '{}'",
            row.row_name
        );
    }
    for row in &matrix.rejection_rows {
        assert!(
            row.control_lane.has_required_outputs(),
            "row '{}'",
            row.row_name
        );
        assert!(
            row.parity_lane.has_required_outputs(),
            "row '{}'",
            row.row_name
        );
        assert!(
            !row.hostile_lane.failure_kind.is_empty(),
            "row '{}'",
            row.row_name
        );
        assert!(
            !row.hostile_lane.failure_digest.is_empty(),
            "row '{}'",
            row.row_name
        );
        assert!(
            !row.hostile_lane.support_matrix_digest.is_empty(),
            "row '{}'",
            row.row_name
        );
        assert!(
            !row.hostile_lane.mutation_surface_report_digest.is_empty(),
            "row '{}'",
            row.row_name
        );
        assert!(
            !row.hostile_lane.closeout_digest.is_empty(),
            "row '{}'",
            row.row_name
        );
    }
}

#[test]
fn aspect_api_finalization_rows_enforce_required_assertion_classes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = AspectApiFinalizationCertificationAdapter::public_aspect_api_finalization_test(
        request_execution,
    );
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
        if row.hostile_lane.failure_digest != row.control_lane.receipt_digest {
            covered.push(RequiredAssertionClass::ZeroResidue);
        }
    }

    covered.sort();
    covered.dedup();
    let missing = unmet_required_assertion_classes(
        &covered,
        &[
            RequiredAssertionClass::Equality,
            RequiredAssertionClass::Inequality,
            RequiredAssertionClass::TypedFailure,
            RequiredAssertionClass::ZeroResidue,
        ],
    );
    assert!(missing.is_empty(), "missing assertion classes: {missing:?}");
}

#[test]
fn aspect_api_finalization_clear_row_proves_touched_scope_narrowing() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = AspectApiFinalizationCertificationAdapter::public_aspect_api_finalization_test(
        request_execution,
    );
    let row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "typed-clear-narrows-by-touched-meaning")
        .expect("clear row should exist");

    assert_eq!(
        row.control_lane.mutation_surface_label,
        "workspace.update.clear"
    );
    assert_eq!(
        row.hostile_lane.mutation_surface_label,
        "workspace.update.clear"
    );
    assert_eq!(row.control_lane.routed_patch_count, 0);
    assert_eq!(row.hostile_lane.routed_patch_count, 1);
    assert_eq!(row.control_lane.affected_live_view_count, 0);
    assert_eq!(row.hostile_lane.affected_live_view_count, 1);
}

#[test]
fn aspect_api_finalization_preview_batch_row_proves_lane_isolation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = AspectApiFinalizationCertificationAdapter::public_aspect_api_finalization_test(
        request_execution,
    );
    let row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "preview-batch-lane-isolation")
        .expect("preview batch row should exist");

    assert_eq!(row.control_lane.authority_lane_label, "authoritative-truth");
    assert_eq!(row.hostile_lane.authority_lane_label, "preview-truth");
    assert_eq!(row.control_lane.mutation_surface_label, "workspace.batch");
    assert_eq!(row.hostile_lane.mutation_surface_label, "preview.batch");
    assert_eq!(row.hostile_lane.preview_residue_count, 0);
    assert_eq!(row.hostile_lane.affected_live_view_count, 0);
}

#[test]
fn aspect_api_finalization_preferred_public_story_never_requires_command_surfaces() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let artifact =
        AspectApiFinalizationCertificationAdapter::public_aspect_api_finalization_artifact(
            request_execution,
        );

    for row in &artifact.matrix.rows {
        if row.row_name == "mutation-surface-closeout-contract-sync" {
            continue;
        }
        for surface in [
            &row.control_lane.mutation_surface_label,
            &row.hostile_lane.mutation_surface_label,
            &row.parity_lane.mutation_surface_label,
        ] {
            assert!(
                !surface.contains("workspace.write"),
                "preferred public certification row '{}' must not rely on workspace.write(...): {surface}",
                row.row_name
            );
            assert!(
                !surface.contains("WorthQueryWriteCommand::"),
                "preferred public certification row '{}' must not rely on command-shaped mutation: {surface}",
                row.row_name
            );
        }
    }
}
