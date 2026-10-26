use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use crate::facade::history::BranchId;
use crate::facade::lineage::{
    HistoricalResolutionBoundednessBasis, HistoricalResolutionDigestMode,
    HistoricalResolutionRequest,
};
use crate::tests::support::*;

#[test]
fn topology_identity_survival_preserves_reidentification_truth_across_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "topology-source");
    let entity = changed_entities(&created)[0];
    let start_lineage = runtime
        .lineage_access()
        .for_record(execution, entity)
        .unwrap()
        .unwrap()
        .lineage_id;

    let replacement = update_entity(execution, &runtime, entity, "topology-source-updated");
    let replaced_entity = changed_entities(&replacement)[0];
    let replacement_lineage = runtime
        .lineage_access()
        .for_record(execution, replaced_entity)
        .unwrap()
        .unwrap()
        .lineage_id;
    let resolution =
        runtime
            .lineage_access()
            .resolve_historical_lineage(HistoricalResolutionRequest {
                branch_id: BranchId("main".to_string()),
                lineage_id: start_lineage,
                boundedness_basis: HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
            });

    assert_eq!(
        resolution.digest_basis().digest_mode(),
        HistoricalResolutionDigestMode::ExactDigestCanonicalOrder
    );
    assert!(!resolution.resolved.is_empty());
    assert_eq!(resolution.resolved[0], replacement_lineage);
    assert_eq!(resolution.metrics.resolved_lineage_count, 1);

    let history = runtime
        .lineage_access()
        .entity_aspect_history(
            execution,
            HistoricalResolutionRequest {
                branch_id: BranchId("main".to_string()),
                lineage_id: start_lineage,
                boundedness_basis: HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
            },
            None,
        )
        .unwrap();
    let history = history.expect("lineage aspect history");
    assert_lineage_history_origin_invariants(&history.entries, start_lineage);

    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    let recovered_resolution =
        recovered
            .lineage_access()
            .resolve_historical_lineage(HistoricalResolutionRequest {
                branch_id: BranchId("main".to_string()),
                lineage_id: start_lineage,
                boundedness_basis: HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
            });
    assert_eq!(
        recovered_resolution.digest_basis(),
        resolution.digest_basis()
    );
    assert_eq!(recovered_resolution.resolved, resolution.resolved);
    assert_eq!(
        recovered_resolution.traversed_event_ids,
        resolution.traversed_event_ids
    );

    assert_recovered_commit_truth_matches(
        execution,
        &runtime,
        &recovered,
        replacement.commit.commit_id,
        &[replaced_entity],
        &[],
        &[start_lineage, replacement_lineage],
    );
}
