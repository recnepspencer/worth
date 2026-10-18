use super::matrix::MilestoneSevenIdentityEvolutionCertificationAdapter;
use crate::harness::certification::HostileExpectation;

#[test]
fn identity_evolution_rows_preserve_expected_semantics() {
    let matrix =
        MilestoneSevenIdentityEvolutionCertificationAdapter::lineage_and_correspondence_query_parity_test();

    let replacement = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "replacement-continuity-explicitness")
        .expect("replacement row should exist");
    assert_eq!(
        replacement.hostile_expectation,
        HostileExpectation::DistinctFromControl
    );
    assert_eq!(
        replacement.control_lane.outcome_family,
        "singular_identity_continuity"
    );
    assert_ne!(
        replacement.control_lane.result_digest,
        replacement.hostile_lane.result_digest
    );

    let split = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "split-successor-explicitness")
        .expect("split row should exist");
    assert_eq!(
        split.hostile_lane.outcome_family,
        "plural_identity_successor_set"
    );
    assert!(split
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "split_successor_fanout_width:2"));

    let branch_local = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "branch-local-divergence-explicitness")
        .expect("branch-local row should exist");
    assert_eq!(
        branch_local.hostile_lane.branch_locality_class,
        "branch_local_only"
    );
    assert!(branch_local
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "branch_local_divergence_count:1"));

    let ambiguous = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "ambiguous-correspondence-explicitness")
        .expect("ambiguity row should exist");
    assert_eq!(ambiguous.hostile_lane.outcome_family, "ambiguity");
    assert_eq!(
        ambiguous.hostile_lane.prediction_drift_outcome,
        "width_drift_detected"
    );

    let identity_break = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "identity-break-explicitness")
        .expect("identity-break row should exist");
    assert_eq!(identity_break.hostile_lane.outcome_family, "identity_break");
    assert_eq!(
        identity_break
            .hostile_lane
            .inspector_identity_classification,
        "identity_break"
    );
    assert!(identity_break
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "identity_break_count:1"));

    let inspector_consumption = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "identity-aware-inspector-consumption-parity")
        .expect("identity-aware inspector row should exist");
    assert_eq!(
        inspector_consumption.control_lane.outcome_family,
        inspector_consumption.hostile_lane.outcome_family
    );
    assert_eq!(
        inspector_consumption.control_lane.result_digest,
        inspector_consumption.hostile_lane.result_digest
    );
    assert_eq!(
        inspector_consumption
            .control_lane
            .inspector_identity_classification,
        inspector_consumption
            .hostile_lane
            .inspector_identity_classification
    );
    assert_eq!(
        inspector_consumption.control_lane.branch_locality_class,
        inspector_consumption.hostile_lane.branch_locality_class
    );
    assert_eq!(
        inspector_consumption
            .control_lane
            .inspector_replay_stable_digest,
        inspector_consumption
            .hostile_lane
            .inspector_replay_stable_digest
    );
    assert_ne!(
        inspector_consumption.control_lane.inspector_identity_digest,
        inspector_consumption.hostile_lane.inspector_identity_digest
    );
    assert_eq!(
        inspector_consumption.hostile_lane.inspector_identity_digest,
        inspector_consumption.parity_lane.inspector_identity_digest
    );

    let disagreement = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "lineage-versus-structural-disagreement-explicitness")
        .expect("disagreement row should exist");
    assert_eq!(disagreement.control_lane.outcome_family, "identity_break");
    assert_eq!(
        disagreement.hostile_lane.outcome_family,
        "advisory_identity_candidate_set"
    );

    let replay = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "lineage-replay-parity")
        .expect("replay row should exist");
    assert_eq!(
        replay.control_lane.result_digest,
        replay.hostile_lane.result_digest
    );
    assert_eq!(
        replay.control_lane.replay_digest,
        replay.parity_lane.replay_digest
    );
    let replay_classification = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "lineage-replay-preserves-classification")
        .expect("replay classification row should exist");
    assert_eq!(
        replay_classification.control_lane.outcome_family,
        replay_classification.hostile_lane.outcome_family
    );
    assert_eq!(
        replay_classification.control_lane.branch_locality_class,
        replay_classification.hostile_lane.branch_locality_class
    );

    let preview = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "preview-to-authoritative-identity-comparison")
        .expect("preview comparison row should exist");
    assert_eq!(
        preview.hostile_lane.outcome_family,
        "singular_identity_continuity"
    );
    assert_eq!(
        preview.hostile_lane.branch_locality_class,
        "cross_branch_authoritative"
    );

    let width = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "identity-evolution-width-drift-explicitness")
        .expect("width drift row should exist");
    assert_eq!(
        width.hostile_lane.prediction_drift_outcome,
        "width_drift_detected"
    );
    assert!(width
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "lineage_width_drift_count:1"));

    let lineage_contract = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "lineage-complexity-contract-parity")
        .expect("lineage contract row should exist");
    assert_ne!(
        lineage_contract.control_lane.complexity_contract_digest,
        lineage_contract.hostile_lane.complexity_contract_digest
    );

    let correspondence_contract = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "correspondence-complexity-contract-parity")
        .expect("correspondence contract row should exist");
    assert_ne!(
        correspondence_contract
            .control_lane
            .complexity_contract_digest,
        correspondence_contract
            .hostile_lane
            .complexity_contract_digest
    );
    assert_eq!(
        correspondence_contract.control_lane.complexity_status,
        "debt"
    );
    assert_eq!(
        correspondence_contract.hostile_lane.complexity_status,
        "debt"
    );

    let status = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "complexity-status-honesty")
        .expect("status row should exist");
    assert_eq!(status.control_lane.complexity_status, "verified");
    assert_eq!(status.hostile_lane.complexity_status, "debt");
}

#[test]
fn identity_evolution_rejection_rows_bind_typed_failures() {
    let matrix =
        MilestoneSevenIdentityEvolutionCertificationAdapter::lineage_and_correspondence_query_parity_test();

    let unsupported_lineage = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "unsupported-lineage-traversal-family")
        .expect("unsupported lineage row should exist");
    assert!(unsupported_lineage
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "admission_failure_class:unsupported_lineage_traversal_family"));

    let unsupported_comparison = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "unsupported-correspondence-family")
        .expect("unsupported comparison row should exist");
    assert!(unsupported_comparison
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "admission_failure_class:unsupported_comparison_basis_family"));

    let advisory = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "advisory-as-authoritative-forbidden")
        .expect("advisory row should exist");
    assert!(advisory
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "advisory_as_authoritative_denial_count:1"));

    let fallback = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "lineage-to-correspondence-fallback-forbidden")
        .expect("fallback row should exist");
    assert!(fallback
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "lineage_to_correspondence_fallback_count:0"));

    let branch_crossing = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "branch-crossing-lineage-forbidden")
        .expect("branch crossing row should exist");
    assert!(branch_crossing
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "branch_crossing_denial_count:1"));

    let broad = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "broad-lineage-scan-forbidden")
        .expect("broad scan row should exist");
    assert!(broad
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "broad_lineage_scan_denial_count:1"));

    let contract = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "complexity-contract-violation-denied")
        .expect("contract row should exist");
    assert!(contract
        .hostile_lane
        .exact_counter_values
        .iter()
        .any(|value| value == "complexity_contract_violation_denial_count:1"));
}
