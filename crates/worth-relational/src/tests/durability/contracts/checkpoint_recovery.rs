use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
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

#[test]
fn durability_contract_checkpoint_recovers_index_metadata() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let commit = create_entity_outcome(execution, &runtime, "indexed");
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "entity-name".to_string(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped: false,
        })
        .expect("index fixture has identity capacity");
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
        .read_latest_generation(execution, index.index_id, &BranchId("main".to_string()))
        .unwrap()
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

#[path = "checkpoint_recovery/empty_head.rs"]
mod empty_head;
