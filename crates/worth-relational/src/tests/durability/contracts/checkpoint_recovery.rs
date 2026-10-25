use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use worth_foundational::{FoundationalBranchReferenceObservation, FoundationalBranchTarget};

#[test]
fn durability_contract_checkpoint_tail_recovery_preserves_post_checkpoint_commits() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let main = create_entity_outcome(execution, &runtime, "main-a");
    let _checkpoint = crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let later = create_entity_outcome(execution, &runtime, "main-b");
    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let checkpoint_images = plan.checkpoint.as_ref().unwrap().partition_images.as_ptr();
    let mut recovered = durable_runtime_with_test_schema();
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
            .canonical_commit_envelope(main.commit.commit_id)
            .unwrap()
            .commit
            .commit_id,
        main.commit.commit_id
    );
    let retained = recovered.durability.latest_checkpoint().unwrap();
    assert_eq!(retained.envelopes.len(), 1);
    assert_eq!(retained.partition_images.as_ptr(), checkpoint_images);
}

#[test]
fn durability_tail_replay_releases_each_commit_snapshot_beyond_live_capacity() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema_profile(
        crate::facade::config::RelationalRuntimeProfile::AiWorkflow,
    );
    create_entity(execution, &runtime, "replay-capacity-checkpoint");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    for index in 0..70 {
        create_entity(
            execution,
            &runtime,
            &format!("replay-capacity-tail-{index}"),
        );
    }
    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema_profile(
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

#[test]
fn durability_contract_recovery_fails_closed_without_exact_branch_cells() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "branch-cell-required");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    plan.checkpoint
        .as_mut()
        .expect("checkpoint is selected")
        .branch_cells
        .clear();

    let mut recovered = durable_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("recovery must not synthesize branch cells from legacy heads");
    let error = crate::tests::support::expect_durability_failure(error);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error
        .detail
        .contains("durable checkpoint omitted exact branch-cell state"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
    assert!(recovered
        .history()
        .branch_head(&BranchId("main".to_owned()))
        .is_none());
}

#[test]
fn durability_contract_recovery_rejects_branch_cell_target_without_catalog_artifact() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "branch-cell-artifact-required");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let checkpoint = plan.checkpoint.as_mut().expect("checkpoint is selected");
    let main_cell = checkpoint
        .branch_cells
        .iter_mut()
        .find(|cell| cell.branch_id == BranchId("main".to_owned()))
        .expect("main branch cell is checkpointed");
    let target = crate::branch::RelationalBranchTarget::new(
        main_cell.runtime_instance_id,
        u64::MAX,
        u64::MAX,
        Vec::new(),
        crate::branch::RelationalBranchRootDescriptor::new([9; 32], [8; 32]),
    );
    main_cell.observation = FoundationalBranchReferenceObservation::new(
        main_cell.observation.branch_id().clone(),
        FoundationalBranchTarget::basis(target),
        main_cell.observation.generation(),
    );

    let mut recovered = durable_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("recovery must reject a branch target with no immutable artifact");
    let error = crate::tests::support::expect_durability_failure(error);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(
        error.detail.contains("references missing commit artifact"),
        "{error:?}"
    );
    assert_eq!(recovered.history().immutable_commit_count(), 0);
    assert!(recovered
        .history()
        .branch_head(&BranchId("main".to_owned()))
        .is_none());
}

#[test]
fn durability_contract_recovery_rejects_tail_checkpoint_drift_before_mutation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let _first = create_entity_outcome(execution, &runtime, "checkpoint-basis");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let second = create_entity_outcome(execution, &runtime, "tail-commit");
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let tail = plan
        .tail_log
        .iter_mut()
        .map(|commit| commit.envelope_mut_for_test())
        .find(|envelope| envelope.commit.commit_id == second.commit.commit_id)
        .expect("tail contains the post-checkpoint commit");
    let checkpoint = tail
        .branch_cell_checkpoint
        .as_mut()
        .expect("tail envelope carries its exact pre-commit branch cell");
    checkpoint.truth_version = crate::branch::RelationalBranchVersion::new(
        checkpoint.truth_version.as_u64().saturating_add(1),
    );

    let mut recovered = durable_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("recovery must reject a mismatching existing branch checkpoint");
    let error = crate::tests::support::expect_durability_failure(error);
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error
        .detail
        .contains("recovery branch-cell state conflicts"));
    assert!(recovered.history().immutable_commit_count() <= 1);
    assert_ne!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_owned()))
            .map(|head| head.commit_id),
        Some(second.commit.commit_id)
    );
}

#[test]
fn durability_contract_checkpoint_recovers_index_metadata() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let commit = create_entity_outcome(execution, &runtime, "indexed");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity-name".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: commit.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
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

    let index_access = recovered.index_access();
    let generation = index_access
        .latest_generation(index.index_id, &BranchId("main".to_string()))
        .unwrap();
    assert_eq!(generation.generation_id, build.generations[0].generation_id);
    assert_eq!(generation.source_commit_id, commit.commit.commit_id);
}

#[test]
fn durability_contract_checkpoint_recovers_lineage_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "first");
    let second = create_entity_outcome(execution, &runtime, "second");
    let first_entity = changed_entities(&first)[0];
    let second_entity = changed_entities(&second)[0];
    let first_lineage = runtime
        .lineage_access()
        .for_record(first_entity)
        .unwrap()
        .lineage_id;
    let second_lineage = runtime
        .lineage_access()
        .for_record(second_entity)
        .unwrap()
        .lineage_id;
    let checkpoint = crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
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
    let graph = recovered.lineage_access().graph(
        execution,
        crate::facade::lineage::LineageGraphRequest {
            branch_id: BranchId("main".to_string()),
            traversal_basis:
                crate::facade::lineage::LineageGraphTraversalBasis::FullBranchGraphMaterialization,
        },
    );

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.events.len(), 2);
    assert!(graph
        .events
        .iter()
        .all(|event| event.kind == LineageEventKind::Create));
    assert_eq!(
        recovered
            .lineage_access()
            .for_record(first_entity)
            .unwrap()
            .lineage_id(),
        first_lineage
    );
    assert_eq!(
        recovered
            .lineage_access()
            .for_record(second_entity)
            .unwrap()
            .lineage_id(),
        second_lineage
    );
}

#[path = "checkpoint_recovery/empty_head.rs"]
mod empty_head;

#[path = "checkpoint_recovery/checkpoint_fallback.rs"]
mod checkpoint_fallback;
