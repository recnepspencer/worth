use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

fn name_index(runtime: &crate::runtime::RelationalRuntime) -> DerivedIndexDefinition {
    runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "checkpoint-index-tail".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped: false,
        })
        .expect("index fixture has identity capacity")
}

fn scoped_name_index(runtime: &crate::runtime::RelationalRuntime) -> DerivedIndexDefinition {
    runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "scoped-checkpoint-index".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped: true,
        })
        .expect("index fixture has identity capacity")
}

fn build(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    index: DerivedIndexId,
    outcome: &crate::transactions::data::CommitResult,
) -> crate::indexes::data::DerivedIndexGenerationId {
    build_on_branch(execution, runtime, index, outcome, BranchId("main".into()))
}

fn build_on_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    index: DerivedIndexId,
    outcome: &crate::transactions::data::CommitResult,
    branch_id: BranchId,
) -> crate::indexes::data::DerivedIndexGenerationId {
    let built = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: outcome.commit.commit_id,
            branch_id,
            index_ids: vec![index],
        },
    );
    assert!(built.failed_indexes.is_empty());
    built.generations[0].generation_id
}

#[test]
fn live_fork_scoped_generation_readmits_from_shared_basis() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let anchor = create_entity_outcome(execution, &runtime, "scoped-fork-checkpoint-anchor");
    let branch = create_branch_from_main(&runtime, "scoped-fork-checkpoint");
    let identity = runtime.branch_identity(&branch).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let index = scoped_name_index(&runtime);
    let built = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: anchor.commit.commit_id,
            branch_id: branch.clone(),
            index_ids: vec![index.index_id],
        },
        &basis,
    );
    assert!(built.failed_indexes.is_empty());
    let generation = built.generations[0].generation_id;
    drop(basis);
    release_test_commit_snapshot(&runtime, &anchor);
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
    assert!(recovered.indexes.generation(generation).is_some());
}

#[test]
fn checkpoint_index_artifact_survives_tail_replay_without_envelope_caches() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "checkpoint-index-first");
    let index = name_index(&runtime);
    let first_generation = build(execution, &runtime, index.index_id, &first);
    let checkpoint = crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    assert!(checkpoint.derived_index_artifacts.is_empty());
    assert_eq!(checkpoint.derived_index_checkpoint_format, 3);
    let second = create_entity_outcome(execution, &runtime, "checkpoint-index-tail");
    let unjournaled_cache = build(execution, &runtime, index.index_id, &second);
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
    assert!(recovered.indexes.generation(first_generation).is_some());
    assert!(recovered.indexes.generation(unjournaled_cache).is_none());
    let rebuilt = build(execution, &recovered, index.index_id, &second);
    assert!(recovered.indexes.generation(rebuilt).is_some());
}

#[path = "derived_index_checkpoint/scoped_affinity.rs"]
mod scoped_affinity;

#[path = "derived_index_checkpoint/lagging_scoped.rs"]
mod lagging_scoped;

#[path = "derived_index_checkpoint/artifact_affinity.rs"]
mod artifact_affinity;
