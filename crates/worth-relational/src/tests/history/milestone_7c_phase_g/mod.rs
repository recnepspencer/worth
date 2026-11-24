//! Owner rebuild of executed merge history from its retained Audit recovery plan.
//!
//! The public replay, durable recovery, envelope, truth and head parity of these
//! three merge shapes is certified by the relational certification court
//! (`durable_publication/merge_execution_certification`). This owner unit keeps
//! only the crate-private direct rebuild entry over actual Memory histories.
mod schema_fixtures;

use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use crate::facade::history::BranchId;
use crate::facade::identity::{KindId, PartitionId};
use crate::facade::merge::{
    MergeExecutionError, MergeExecutionOutcome, MergeExecutionRequest, MergeIntent,
};
use crate::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use crate::facade::transactions::{CreateIntent, MutationIntent, WorkerIntentBatch};
use crate::tests::support::{
    create_branch_from_main, create_entity, create_entity_outcome_on_branch,
    runtime_with_test_schema, update_entity, update_entity_on_branch, RelationalRuntimeProfile,
};
use schema_fixtures::{drifted_schema_registry, prefer_richer_registry};

#[test]
fn executed_merge_history_rebuilds_directly_from_its_audit_recovery_plan() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, merge) = exact_shared_merge(execution);
    assert_eq!(merge.structural_summary.preserved_shared_record_count, 1);
    assert_eq!(merge.structural_summary.emitted_mutation_intent_count, 0);
    assert_direct_rebuild(execution, &runtime);

    let (runtime, merge) = source_only_addition_merge(execution);
    assert_eq!(merge.structural_summary.adopted_source_record_count, 1);
    assert_eq!(merge.structural_summary.emitted_entity_create_count, 1);
    assert_direct_rebuild(execution, &runtime);

    let (runtime, merge) = prefer_richer_merge(execution);
    assert_eq!(merge.structural_summary.reconciled_record_count, 1);
    assert_eq!(merge.structural_summary.emitted_entity_update_count, 1);
    assert_direct_rebuild(execution, &runtime);
}

fn assert_direct_rebuild(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) {
    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::AuditRecoveryVerification)
        .unwrap();
    runtime
        .rebuild_runtime_from_plan(execution, plan)
        .unwrap_or_else(|error| panic!("direct replay rebuild failed: {error:?}"));
}

fn execute_feature_into_main(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> MergeExecutionOutcome {
    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest::new(
                BranchId("main".to_string()),
                BranchId("feature".to_string()),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("prepared merge");
    runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed merge")
}

fn exact_shared_merge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (RelationalRuntime, MergeExecutionOutcome) {
    let runtime = runtime_with_test_schema();
    let shared = create_entity(execution, &runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity(execution, &runtime, shared, "same");
    update_entity_on_branch(
        execution,
        &runtime,
        shared,
        "same",
        BranchId("feature".to_string()),
    );
    let merge = execute_feature_into_main(execution, &runtime);
    (runtime, merge)
}

fn source_only_addition_merge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (RelationalRuntime, MergeExecutionOutcome) {
    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );
    let merge = execute_feature_into_main(execution, &runtime);
    (runtime, merge)
}

fn prefer_richer_merge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (RelationalRuntime, MergeExecutionOutcome) {
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(prefer_richer_registry())
        .build()
        .expect("valid native runtime configuration");
    create_entity(execution, &runtime, "shared-name");
    create_branch_from_main(&runtime, "feature");

    let mut feature_txn = crate::tests::support::test_owner_begin_transaction_for_branch(
        &runtime,
        BranchId("feature".to_string()),
    );
    feature_txn
        .push_batch(
            WorkerIntentBatch::new("feature-seed").push(MutationIntent::Create(
                CreateIntent::Entity(crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("feature-shared"),
                    fields: crate::tests::support::string_aspect_field_patch([
                        (
                            crate::tests::support::aspect_key("name"),
                            crate::tests::support::field_key("name"),
                            "shared-name",
                        ),
                        (
                            crate::tests::support::aspect_key("status"),
                            crate::tests::support::field_key("status"),
                            "active",
                        ),
                    ]),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    feature_txn
        .commit(execution, &runtime)
        .expect("feature branch seed");
    let merge = execute_feature_into_main(execution, &runtime);
    (runtime, merge)
}

#[test]
fn authoritative_merge_execution_certification_rejects_metadata_reference_movement() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest::new(
                BranchId("main".to_string()),
                BranchId("feature".to_string()),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("prepared merge execution");
    let target_branch = BranchId("main".to_string());
    let target_head = runtime
        .history()
        .branch_head(&target_branch)
        .expect("main has a committed head");
    let target_generation = runtime
        .history
        .branch_cell(&target_branch)
        .expect("main has a reference cell")
        .observation()
        .generation()
        .get();
    runtime
        .history
        .branch_cell_mut(&target_branch)
        .expect("main has a mutable reference cell")
        .advance_metadata()
        .expect("metadata movement should advance only the reference generation");
    assert_eq!(
        runtime
            .history()
            .branch_head(&target_branch)
            .expect("metadata movement does not change the immutable head")
            .commit_id,
        target_head.commit_id
    );
    assert_eq!(
        runtime
            .history
            .branch_cell(&target_branch)
            .expect("main reference remains live")
            .observation()
            .generation()
            .get(),
        target_generation + 1
    );

    match runtime
        .merge()
        .verify_prepared_merge_execution(execution, &prepared)
    {
        Err(MergeExecutionError::StaleBranchReference { branch, .. }) => {
            assert_eq!(branch, target_branch);
        }
        other => panic!("expected metadata-only reference movement rejection, got {other:?}"),
    }
}

#[test]
fn authoritative_merge_execution_certification_rejects_schema_semantic_drift_after_planning() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest::new(
                BranchId("main".to_string()),
                BranchId("feature".to_string()),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("prepared merge execution");
    runtime.set_schema_registry_for_test(drifted_schema_registry());

    match runtime.execute_prepared_merge(execution, prepared) {
        Err(MergeExecutionError::SchemaSemanticDrift { .. }) => {}
        other => panic!("expected schema semantic drift rejection, got {other:?}"),
    }
}
