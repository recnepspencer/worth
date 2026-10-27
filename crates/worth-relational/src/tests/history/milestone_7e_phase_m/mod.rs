use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
mod drift_support;
mod fixtures;

use crate::facade::durability::RecoveryFailureClass;
use crate::facade::inspection::{
    RelationalMergeSupportInspectionAbsenceKind,
    RelationalMergeSupportInspectionCompatibilityPosture, RelationalMergeSupportInspectionDenial,
    RelationalMergeSupportInspectionRow,
};
use crate::tests::support::checkpoint_and_recover_with;

use drift_support::{
    branch_basis_drifted_authority, collaboration_family_drift, correspondence_drifted_authority,
    proof_packet_drifted_authority, schema_drifted_authority, strategy_drifted_authority,
};
use fixtures::{
    alternate_branch_basis, execute_feature_merge, published_merge_authority,
    runtime_with_collaboration_merge_history, snapshot_from_authority, snapshot_from_summary,
};

#[test]
fn merge_collaboration_truth_replay_equivalence_survives_live_publication_and_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_collaboration_merge_history();
    let outcome = execute_feature_merge(execution, &runtime);
    let live = snapshot_from_summary(&runtime, &outcome.execution_summary);
    let published = snapshot_from_authority(
        &runtime,
        published_merge_authority(execution, &runtime, outcome.commit.commit.commit_id),
    );
    let (_recovery, recovered) = checkpoint_and_recover_with(
        execution,
        &runtime,
        runtime_with_collaboration_merge_history,
    );
    let recovered_snapshot = snapshot_from_authority(
        &recovered,
        published_merge_authority(execution, &recovered, outcome.commit.commit.commit_id),
    );

    assert_eq!(
        live.authority.execution_summary,
        published.authority.execution_summary
    );
    assert_eq!(
        live.authority.execution_summary,
        recovered_snapshot.authority.execution_summary
    );
    assert_eq!(
        live.authority.execution_summary.request,
        published.authority.execution_summary.request
    );
    assert_eq!(
        live.authority.execution_summary.branch_basis,
        published.authority.execution_summary.branch_basis
    );
    assert_eq!(
        live.authority.execution_summary.correspondence_witness,
        published.authority.execution_summary.correspondence_witness
    );
    assert_eq!(
        live.authority
            .execution_summary
            .schema_reconciliation_witness,
        published
            .authority
            .execution_summary
            .schema_reconciliation_witness
    );
    assert_eq!(
        live.authority.execution_summary.strategy_witness,
        published.authority.execution_summary.strategy_witness
    );
    assert_eq!(live.canonical_basis, published.canonical_basis);
    assert_eq!(
        published.canonical_basis,
        recovered_snapshot.canonical_basis
    );
    assert_eq!(live.support, published.support);
    assert_eq!(published.support, recovered_snapshot.support);
    assert!(published.support.rows().iter().any(|row| matches!(
        row,
        RelationalMergeSupportInspectionRow::Compatibility {
            posture: RelationalMergeSupportInspectionCompatibilityPosture::UnavailablePhaseDependency,
            absence: Some(
                RelationalMergeSupportInspectionAbsenceKind::MissingCompatibilityWitnessPhaseDependency
            ),
            ..
        }
    )));
}

#[test]
fn merge_collaboration_support_surface_reflects_correspondence_posture_over_real_history() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_collaboration_merge_history();
    let outcome = execute_feature_merge(execution, &runtime);
    let snapshot = snapshot_from_authority(
        &runtime,
        published_merge_authority(execution, &runtime, outcome.commit.commit.commit_id),
    );
    let witness = &snapshot.authority.execution_summary.correspondence_witness;
    let row = snapshot
        .support
        .rows()
        .iter()
        .find_map(|row| match row {
            RelationalMergeSupportInspectionRow::Correspondence {
                witness_digest,
                admitted_count,
                denied_count,
                unavailable_count,
                sample_posture,
                ..
            } => Some((
                witness_digest.as_deref(),
                *admitted_count,
                *denied_count,
                *unavailable_count,
                *sample_posture,
            )),
            _ => None,
        })
        .expect("correspondence support row");

    let denied_count = witness
        .rows()
        .iter()
        .filter(|row| {
            !matches!(
                row.posture(),
                crate::facade::merge::RelationalMergeCorrespondenceWitnessPosture::Admitted
                    | crate::facade::merge::RelationalMergeCorrespondenceWitnessPosture::UnavailableMissingTarget
            )
        })
        .count();
    let unavailable_count = witness
        .rows()
        .iter()
        .filter(|row| {
            matches!(
                row.posture(),
                crate::facade::merge::RelationalMergeCorrespondenceWitnessPosture::UnavailableMissingTarget
            )
        })
        .count();

    assert_eq!(row.0, Some(witness.witness_digest()));
    assert_eq!(row.1, witness.admitted_rows().count());
    assert_eq!(row.2, denied_count);
    assert_eq!(row.3, unavailable_count);
    assert_eq!(row.4, witness.rows().first().map(|entry| entry.posture()));
}

#[test]
fn merge_collaboration_truth_localizes_single_family_drift() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_collaboration_merge_history();
    let outcome = execute_feature_merge(execution, &runtime);
    let baseline = published_merge_authority(execution, &runtime, outcome.commit.commit.commit_id);

    assert_eq!(
        collaboration_family_drift(
            &baseline,
            &branch_basis_drifted_authority(&baseline, alternate_branch_basis(&runtime)),
        ),
        vec!["branch_basis"]
    );
    assert_eq!(
        collaboration_family_drift(&baseline, &proof_packet_drifted_authority(&baseline)),
        vec!["proof_packet"]
    );
    assert_eq!(
        collaboration_family_drift(&baseline, &correspondence_drifted_authority(&baseline)),
        vec!["correspondence_witness"]
    );
    assert_eq!(
        collaboration_family_drift(&baseline, &schema_drifted_authority(&baseline)),
        vec!["schema_reconciliation_witness"]
    );
    assert_eq!(
        collaboration_family_drift(&baseline, &strategy_drifted_authority(&baseline)),
        vec!["strategy_witness"]
    );
}

#[test]
fn merge_collaboration_truth_denies_cross_family_witness_mismatch() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_collaboration_merge_history();
    let outcome = execute_feature_merge(execution, &runtime);
    let baseline = published_merge_authority(execution, &runtime, outcome.commit.commit.commit_id);

    for drifted in [
        correspondence_drifted_authority(&baseline),
        schema_drifted_authority(&baseline),
        strategy_drifted_authority(&baseline),
    ] {
        let denied = runtime
            .inspect_what_happened()
            .prepare_published_merge_support_inspection_witness(&drifted);
        assert!(matches!(
            denied,
            Err(RelationalMergeSupportInspectionDenial::InconsistentRetainedProofAuthority)
        ));
    }

    crate::tests::support::attached_test_medium(&runtime).rewrite_commit_envelope(
        execution,
        outcome.commit.commit.commit_id,
        |merge_entry| {
            merge_entry
                .merge_execution_authority
                .as_mut()
                .expect("merge execution authority")
                .execution_summary
                .schema_reconciliation_witness = schema_drifted_authority(&baseline)
                .execution_summary
                .schema_reconciliation_witness;
        },
    );

    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = runtime_with_collaboration_merge_history();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::ReplayFailure);
    assert_eq!(error.history_drift_class, None);
    assert!(error
        .detail
        .contains("failed to reconstruct merge execution summary"));
}

#[test]
fn merge_collaboration_truth_denies_summary_only_and_planner_shortcuts() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_collaboration_merge_history();
    let outcome = execute_feature_merge(execution, &runtime);
    crate::tests::support::attached_test_medium(&runtime).rewrite_commit_envelope(
        execution,
        outcome.commit.commit.commit_id,
        |merge_entry| {
            merge_entry
                .merge_execution_authority
                .as_mut()
                .expect("merge execution authority")
                .execution_summary
                .execution_digest = "forged-summary-digest".to_string();
        },
    );

    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = runtime_with_collaboration_merge_history();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::ReplayFailure);
    assert_eq!(error.history_drift_class, None);
    assert!(error
        .detail
        .contains("failed to reconstruct merge execution summary"));
}
