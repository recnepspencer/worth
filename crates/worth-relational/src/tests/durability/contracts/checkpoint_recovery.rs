use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn memory_checkpoint_preserves_lineage_artifact_and_original_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "first");
    let second = create_entity_outcome(execution, &runtime, "second");
    let first_entity = changed_entities(&first)[0];
    let second_entity = changed_entities(&second)[0];
    let first_lineage = runtime
        .lineage_access()
        .for_record(execution, first_entity)
        .unwrap()
        .unwrap()
        .lineage_id;
    let second_lineage = runtime
        .lineage_access()
        .for_record(execution, second_entity)
        .unwrap()
        .unwrap()
        .lineage_id;
    let checkpoint = runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    assert_eq!(
        checkpoint
            .lineage
            .digest_basis()
            .published_lineage_commit_count,
        runtime
            .history()
            .commit_envelopes_snapshot()
            .iter()
            .filter(|envelope| envelope.has_lineage_authority())
            .count()
    );
    assert_eq!(
        checkpoint
            .lineage
            .digest_basis()
            .published_lineage_event_count,
        runtime
            .history()
            .commit_envelopes_snapshot()
            .iter()
            .map(|envelope| envelope.lineage_digest_basis().lineage_event_count())
            .sum::<usize>()
    );
    assert_eq!(
        checkpoint
            .lineage
            .digest_basis()
            .published_lineage_decision_count,
        runtime
            .history()
            .commit_envelopes_snapshot()
            .iter()
            .map(|envelope| envelope.lineage_digest_basis().lineage_decision_count())
            .sum::<usize>()
    );
    assert_eq!(
        checkpoint.lineage.counters().node_count,
        checkpoint.lineage.nodes().len()
    );
    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let mut recovered = runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let graph = recovered.lineage_access().graph(
        execution,
        crate::facade::lineage::LineageGraphRequest {
            branch_id: BranchId("main".to_string()),
            traversal_basis:
                crate::facade::lineage::LineageGraphTraversalBasis::FullBranchGraphMaterialization,
        },
    ).unwrap();

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.events.len(), 2);
    assert!(graph
        .events
        .iter()
        .all(|event| event.kind == LineageEventKind::Create));
    assert_eq!(
        recovered
            .lineage_access()
            .for_record(execution, first_entity)
            .unwrap()
            .unwrap()
            .lineage_id(),
        first_lineage
    );
    assert_eq!(
        recovered
            .lineage_access()
            .for_record(execution, second_entity)
            .unwrap()
            .unwrap()
            .lineage_id(),
        second_lineage
    );
}

// Fixture adaptation (C2.4): restores e4d3b01c8d's retained envelope census
// and partition-image pointer identity on the Memory runtime (Embedded
// checkpoint plus post-checkpoint tail recovery plan).
#[test]
fn durability_contract_checkpoint_tail_recovery_preserves_post_checkpoint_commits() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let main = create_entity_outcome(execution, &runtime, "main-a");
    let _checkpoint = runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let later = create_entity_outcome(execution, &runtime, "main-b");
    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let checkpoint_images = plan.checkpoint.as_ref().unwrap().partition_images.as_ptr();
    let mut recovered = runtime_with_test_schema();
    let outcome = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    assert_eq!(outcome.recovered_commits, 2);
    assert_eq!(outcome.coverage.checkpoint_commits, 1);
    assert_eq!(outcome.coverage.replayed_tail_commits, 1);
    assert_eq!(outcome.latest_commit, Some(later.commit.clone()));
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_string())),
        Some(later.commit.clone())
    );
    assert_eq!(
        recovered
            .replay()
            .canonical_commit_envelope(execution, main.commit.commit_id)
            .unwrap()
            .unwrap()
            .commit
            .commit_id,
        main.commit.commit_id
    );
    let retained = recovered.durability.latest_checkpoint().unwrap();
    assert_eq!(retained.envelopes.len(), 1);
    assert_eq!(retained.partition_images.as_ptr(), checkpoint_images);
}

// Fixture adaptation (C2.4): restores f9f15bf8c8's tail-replay snapshot-handle
// release bound beyond live capacity on the Memory runtime.
#[test]
fn durability_tail_replay_releases_each_commit_snapshot_beyond_live_capacity() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(
        crate::facade::config::RelationalRuntimeProfile::AiWorkflow,
    );
    create_entity(execution, &runtime, "replay-capacity-checkpoint");
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    for index in 0..70 {
        create_entity(
            execution,
            &runtime,
            &format!("replay-capacity-tail-{index}"),
        );
    }
    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let mut recovered = runtime_with_test_schema_profile(
        crate::facade::config::RelationalRuntimeProfile::AiWorkflow,
    );

    let outcome = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    assert_eq!(outcome.recovered_commits, 71);
    assert_eq!(
        recovered
            .storage_access()
            .storage_stats(execution)
            .unwrap()
            .published_snapshot_handle_count,
        0,
        "replay closeout must not consume the live published-snapshot budget"
    );
}
