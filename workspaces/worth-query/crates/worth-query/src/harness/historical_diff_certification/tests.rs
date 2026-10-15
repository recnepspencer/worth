use super::matrix::MilestoneSixHistoricalDiffCertificationAdapter;

#[test]
fn historical_diff_branch_to_branch_lane_emits_exact_phase_three_posture() {
    let matrix =
        MilestoneSixHistoricalDiffCertificationAdapter::branch_scoped_historical_and_diff_query_context_test();
    let row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "branch-to-branch-diff-shaped")
        .expect("branch-to-branch diff row should exist");

    assert_eq!(
        row.hostile_lane.comparison_basis_family, "branch_to_branch",
        "branch-to-branch diff must advertise the exact comparison family"
    );
    assert_eq!(
        row.hostile_lane.cost_class, "diff_comparison_bounded",
        "branch-to-branch diff must advertise diff cost posture"
    );
    assert_eq!(
        row.hostile_lane.budget_class, "comparison_bounded",
        "branch-to-branch diff must advertise diff budget posture"
    );
    assert_eq!(
        row.hostile_lane.prediction_drift_outcome, "within_budget",
        "branch-to-branch diff shaping must report realized prediction posture"
    );
    assert!(row
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "comparison_basis_lookups:1"));
    assert!(row
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "comparison_broadening_denials:0"));
    assert!(row
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "query_context_execution_count:2"));
    assert!(row
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "query_context_metadata_attachment_count:1"));
    assert!(row
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "query_context_executor_rediscovery:0"));
    assert!(row
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "comparison_family_rediscovery:0"));
    assert_ne!(
        row.control_lane.replay_digest, row.hostile_lane.replay_digest,
        "branch-only and diff lanes must not collapse to one replay artifact"
    );
    assert_ne!(
        row.control_lane.result_digest, row.hostile_lane.result_digest,
        "branch-only and diff lanes must not collapse to one result artifact"
    );
}

#[test]
fn historical_diff_current_to_historical_lane_emits_exact_comparison_family() {
    let matrix =
        MilestoneSixHistoricalDiffCertificationAdapter::branch_scoped_historical_and_diff_query_context_test();
    let row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "current-to-historical-diff-shaped")
        .expect("current-to-historical diff row should exist");

    assert_eq!(
        row.hostile_lane.comparison_basis_family,
        "current_to_historical"
    );
    assert_eq!(row.hostile_lane.prediction_drift_outcome, "within_budget");
    assert!(row
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "comparison_basis_lookups:1"));
    assert_eq!(
        row.hostile_lane.historical_admission_class,
        "runtime_retained"
    );
    assert_ne!(
        row.hostile_lane.materialization_path_identity, "none",
        "current-to-historical diff should preserve historical path identity"
    );
}

#[test]
fn historical_diff_rejection_rows_emit_exact_phase_three_failure_counters() {
    let matrix =
        MilestoneSixHistoricalDiffCertificationAdapter::branch_scoped_historical_and_diff_query_context_test();

    for row in &matrix.rejection_rows {
        assert!(
            !row.hostile_lane.failure_digest.is_empty(),
            "rejection rows must emit a stable failure digest"
        );
        assert!(
            !row.hostile_lane.counter_snapshot_digest.is_empty(),
            "rejection rows must emit a counter snapshot digest"
        );
    }

    let broadening = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "historical-broadening-denied")
        .expect("historical broadening rejection row should exist");
    assert_eq!(
        broadening.hostile_lane.failure_class,
        crate::harness::historical_diff_certification::HistoricalDiffFailureClass::HistoricalPathTooBroadDenied
    );
    assert!(broadening
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "historical_broadening_denials:1"));
    assert!(broadening
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "materialization_path_compatibility_checks:1"));

    let broadening = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "broadening-required-comparison-denial")
        .expect("broadening rejection row should exist");
    assert_eq!(
        broadening.hostile_lane.failure_class,
        crate::harness::historical_diff_certification::HistoricalDiffFailureClass::ComparisonBroadeningRequired
    );
    assert!(broadening
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "comparison_broadening_denials:1"));
    assert!(broadening
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "basis_substitution_denials:0"));
    assert!(broadening
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "unsupported_denials:1"));

    let shape_mismatch = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "declared-result-shape-mismatch")
        .expect("shape mismatch rejection row should exist");
    assert_eq!(
        shape_mismatch.hostile_lane.failure_class,
        crate::harness::historical_diff_certification::HistoricalDiffFailureClass::ComparisonShapeMismatch
    );
    assert!(shape_mismatch
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "comparison_broadening_denials:0"));
    assert!(shape_mismatch
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "basis_substitution_denials:0"));
}

#[test]
fn historical_diff_zero_rediscovery_and_metadata_identity_are_explicit() {
    let matrix =
        MilestoneSixHistoricalDiffCertificationAdapter::branch_scoped_historical_and_diff_query_context_test();

    for row in &matrix.rows {
        for lane in [&row.control_lane, &row.hostile_lane, &row.parity_lane] {
            assert!(
                lane.exact_counter_values
                    .iter()
                    .any(|value| value == "basis_rediscovery:0"),
                "basis rediscovery must stay zero on admitted lanes"
            );
            assert!(
                lane.exact_counter_values
                    .iter()
                    .any(|value| value == "historical_path_rediscovery:0"),
                "historical-path rediscovery must stay zero on admitted lanes"
            );
            assert!(
                lane.exact_counter_values
                    .iter()
                    .any(|value| value == "comparison_family_rediscovery:0"),
                "comparison-family rediscovery must stay zero on admitted lanes"
            );
        }
    }

    let historical = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "historical-materialization-path-explicitness")
        .expect("historical materialization row should exist");
    assert_eq!(
        historical.control_lane.query_digest, historical.hostile_lane.query_digest,
        "historical materialization explicitness must preserve canonical query identity"
    );
    assert_ne!(
        historical.control_lane.basis_digest, historical.hostile_lane.basis_digest,
        "historical materialization explicitness must compare distinct bases"
    );
    assert_ne!(
        historical.hostile_lane.materialization_path_identity, "none",
        "historical basis lanes must carry materialization-path identity"
    );

    let preview = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "preview-derived-historical-basis-explicitness")
        .expect("preview-derived row should exist");
    assert_ne!(
        preview.hostile_lane.preview_provenance_identity, "none",
        "preview-derived basis lanes must carry preview provenance identity"
    );
}

#[test]
fn historical_diff_parity_rows_prove_expected_equality_and_distinction() {
    let matrix =
        MilestoneSixHistoricalDiffCertificationAdapter::branch_scoped_historical_and_diff_query_context_test();

    let parity = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "result-shape-parity-across-basis-variants")
        .expect("result-shape parity row should exist");
    assert_eq!(
        parity.control_lane.query_digest,
        parity.hostile_lane.query_digest
    );
    assert_ne!(
        parity.control_lane.basis_digest,
        parity.hostile_lane.basis_digest
    );
    assert_eq!(
        parity.control_lane.result_shape_width,
        parity.hostile_lane.result_shape_width
    );
    assert_eq!(
        parity.control_lane.result_shape_digest,
        parity.hostile_lane.result_shape_digest
    );
    assert_eq!(
        parity.control_lane.replay_digest,
        parity.parity_lane.replay_digest
    );

    let current_vs_branch = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "current-vs-branch-basis-explicitness")
        .expect("current-vs-branch row should exist");
    assert_eq!(
        current_vs_branch.control_lane.query_digest,
        current_vs_branch.hostile_lane.query_digest
    );
    assert_ne!(
        current_vs_branch.control_lane.basis_digest,
        current_vs_branch.hostile_lane.basis_digest
    );
    assert_ne!(
        current_vs_branch.control_lane.replay_digest,
        current_vs_branch.hostile_lane.replay_digest
    );

    let preview = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "preview-derived-historical-basis-explicitness")
        .expect("preview-derived row should exist");
    assert_ne!(
        preview.control_lane.preview_provenance_identity,
        preview.hostile_lane.preview_provenance_identity
    );
}
