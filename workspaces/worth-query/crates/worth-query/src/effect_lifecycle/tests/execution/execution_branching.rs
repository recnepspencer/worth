use worth_relational::facade::commit_strategies::{
    CommitStrategyId, CommitStrategyRegistration, IntentReconciliationStrategy,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};
use worth_relational::facade::transactions::{
    CreateIntent, EntityMutationIntent, EntitySpec, MutationIntent, RecordRef,
    UpdateEntityFieldsIntent, WorkerIntentBatch,
};
use worth_relational::facade::{identity::KindId, identity::PartitionId, symbols::ClientKey};

use crate::aspect_field_authoring::{
    aspect_key, entity_string_field_aspect, lifecycle_string_aspect,
    single_native_string_aspect_field_patch,
};
use crate::effect_lifecycle::{
    scope_admitted_effect_plan, EffectExecutionAuthority, EffectExecutionDenialKind,
};

use super::execution_support::{
    branch_snapshot_identity, exact_branch_head_commit_id, exact_branch_snapshot,
};
use super::support::{
    admitted_mutation_effect_for_entity_with_binding, native_name_patch,
    runtime_workflow_binding_for_branch,
};

#[test]
fn lowered_mutation_execution_preserves_branch_scoped_authority_target() {
    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(&mut runtime, "before", BranchId("main".to_string()));
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId("branch-a".to_string()),
        &BranchId("main".to_string()),
    )
    .expect("branch-a should be created");
    let main_head_before = update_entity_name(
        &mut runtime,
        entity_id,
        "main-diverged",
        BranchId("main".to_string()),
    );
    let branch_head_before = exact_branch_head_commit_id(&runtime, "branch-a");
    let lowered = scope_admitted_effect_plan(admitted_mutation_effect_for_entity_with_binding(
        runtime_workflow_binding_for_branch(
            branch_snapshot_identity(&runtime, "branch-a"),
            "branch-a",
        ),
        entity_id,
        native_name_patch("authority-plan"),
    ))
    .lower()
    .expect("mutation should lower");

    let executed = lowered
        .execute_with(EffectExecutionAuthority::relational(&mut runtime))
        .expect("lowered mutation should execute");

    let commit = executed
        .as_mutation()
        .expect("mutation artifact should be present");
    assert_eq!(commit.outcome().commit.parents, vec![branch_head_before]);
    assert_ne!(commit.outcome().commit.parents, vec![main_head_before]);
    assert_eq!(
        exact_branch_head_commit_id(&runtime, "main"),
        main_head_before
    );
    assert_eq!(
        exact_branch_head_commit_id(&runtime, "branch-a"),
        commit.outcome().commit.commit_id
    );
    let snapshot = exact_branch_snapshot(&mut runtime, "branch-a");
    let read_view = runtime
        .read_truth()
        .read_snapshot(&snapshot)
        .expect("snapshot should read");
    let updated = read_view
        .entities()
        .iter()
        .find(|record| record.entity_id == entity_id)
        .expect("entity should still exist after execution");
    assert_entity_name(updated, "authority-plan");
    assert!(runtime.snapshots().release_snapshot(&snapshot).is_ok());
}

#[test]
fn retained_lowered_mutation_denies_after_intervening_truth_change() {
    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(&mut runtime, "before", BranchId("main".to_string()));
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId("branch-a".to_string()),
        &BranchId("main".to_string()),
    )
    .expect("branch-a should be created");
    let lowered = scope_admitted_effect_plan(admitted_mutation_effect_for_entity_with_binding(
        runtime_workflow_binding_for_branch(
            branch_snapshot_identity(&runtime, "branch-a"),
            "branch-a",
        ),
        entity_id,
        native_name_patch("authority-plan"),
    ))
    .lower()
    .expect("mutation should lower");

    let branch_head_before = exact_branch_head_commit_id(&runtime, "branch-a");
    update_entity_name(
        &mut runtime,
        entity_id,
        "intervening",
        BranchId("branch-a".to_string()),
    );

    let denial = lowered
        .execute_with(EffectExecutionAuthority::relational(&mut runtime))
        .expect_err("retained lowered mutation should deny stale exact-basis replay");
    let denial = denial.denial().expect("stale replay is a denial");

    assert_eq!(
        denial.denial_kind(),
        EffectExecutionDenialKind::RelationalExactBasisStale
    );
    assert_eq!(denial.counters().execution_denied_count(), 1);
    assert_eq!(
        exact_branch_head_commit_id(&runtime, "branch-a"),
        runtime
            .history()
            .historical_latest_commit()
            .unwrap()
            .commit_id
    );
    assert_ne!(
        exact_branch_head_commit_id(&runtime, "branch-a"),
        branch_head_before
    );
    let snapshot = exact_branch_snapshot(&mut runtime, "branch-a");
    let read_view = runtime
        .read_truth()
        .read_snapshot(&snapshot)
        .expect("snapshot should read");
    let updated = read_view
        .entities()
        .iter()
        .find(|record| record.entity_id == entity_id)
        .expect("entity should still exist after stale denial");
    assert_entity_name(updated, "intervening");
    assert!(runtime.snapshots().release_snapshot(&snapshot).is_ok());
}

#[test]
fn lowered_branch_mutation_does_not_deny_when_only_another_branch_moves() {
    let mut runtime = relational_runtime_with_intent_strategy();
    let entity_id = create_entity(&mut runtime, "before", BranchId("main".to_string()));
    crate::runtime::fork_branch_from_exact_source(
        &mut runtime,
        BranchId("branch-a".to_string()),
        &BranchId("main".to_string()),
    )
    .expect("branch-a should be created");
    let lowered = scope_admitted_effect_plan(admitted_mutation_effect_for_entity_with_binding(
        runtime_workflow_binding_for_branch(
            branch_snapshot_identity(&runtime, "branch-a"),
            "branch-a",
        ),
        entity_id,
        native_name_patch("authority-plan"),
    ))
    .lower()
    .expect("mutation should lower");

    let main_head_before = update_entity_name(
        &mut runtime,
        entity_id,
        "main-only-intervening",
        BranchId("main".to_string()),
    );

    let executed = lowered
        .execute_with(EffectExecutionAuthority::relational(&mut runtime))
        .expect("unrelated main-branch movement should not stale-deny branch-a mutation");

    let commit = executed
        .as_mutation()
        .expect("mutation artifact should be present");
    assert_ne!(commit.outcome().commit.commit_id, main_head_before);
    assert_eq!(commit.outcome().commit.parents.len(), 1);
    assert_eq!(
        exact_branch_head_commit_id(&runtime, "main"),
        main_head_before
    );
}

fn relational_runtime_with_intent_strategy() -> RelationalRuntime {
    let descriptor = IntentReconciliationStrategy::descriptor(CommitStrategyId(311));
    RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .commit_strategy(
            CommitStrategyRegistration::new(descriptor.clone()).expect("strategy registration"),
        )
        .commit_strategy_executor(IntentReconciliationStrategy::execution_registration(
            &descriptor,
        ))
        .build()
}

fn create_entity(
    runtime: &mut RelationalRuntime,
    name: &str,
    branch: BranchId,
) -> worth_relational::facade::identity::EntityId {
    let mut txn = {
        let identity = runtime.branch_identity(&branch).expect("branch identity");
        let transaction_validation_input = runtime
            .admit_branch_basis(&identity)
            .expect("branch binding");
        runtime
            .begin_branch_transaction(
                &transaction_validation_input,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(
        WorkerIntentBatch::new(format!("create-{name}")).push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw(name),
                fields: single_native_string_aspect_field_patch("name", "name", name)
                    .expect("entity name aspect patch"),
            }),
        )),
        worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("execution fixture staging fits its transaction budget");
    let outcome = txn
        .commit(
            runtime,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("seed commit should succeed");
    let entity = outcome
        .changed_records
        .iter()
        .find_map(|record| match record {
            RecordRef::Entity(entity_id) => Some(*entity_id),
            RecordRef::Relation(_) => None,
        })
        .expect("seed commit should touch one entity");
    runtime
        .snapshots()
        .release_snapshot(&outcome.snapshot)
        .expect("execution fixture releases its exact published snapshot");
    entity
}

fn update_entity_name(
    runtime: &mut RelationalRuntime,
    entity_id: worth_relational::facade::identity::EntityId,
    name: &str,
    branch: BranchId,
) -> worth_relational::facade::history::CommitId {
    let mut txn = {
        let identity = runtime.branch_identity(&branch).expect("branch identity");
        let transaction_validation_input = runtime
            .admit_branch_basis(&identity)
            .expect("branch binding");
        runtime
            .begin_branch_transaction(
                &transaction_validation_input,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(
        WorkerIntentBatch::new(format!("update-{name}")).push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id,
                fields: single_native_string_aspect_field_patch("name", "name", name)
                    .expect("entity name aspect patch"),
            }),
        )),
        worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("execution fixture staging fits its transaction budget");
    let outcome = txn
        .commit(
            runtime,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("intervening update should succeed");
    let commit_id = outcome.outcome().commit.commit_id;
    runtime
        .snapshots()
        .release_snapshot(&outcome.snapshot)
        .expect("execution fixture releases its exact published snapshot");
    commit_id
}

fn test_schema_registry() -> RelationalSchemaRegistry {
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_string_field_aspect("name", "name").expect("name aspect"),
                lifecycle_string_aspect("lifecycle").expect("lifecycle aspect"),
            ]),
        })
        .expect("test entity kind should register")
}

fn assert_entity_name(
    record: &worth_relational::facade::runtime::EntityReadRecord,
    expected_name: &str,
) {
    let state = record
        .authoritative_aspect_state
        .as_ref()
        .expect("entity should carry authoritative aspect state");
    let value = state
        .get(&aspect_key("name").expect("valid aspect key"))
        .expect("name aspect should be present");
    match value.view() {
        worth_foundational::facade::ContractValidatedAspectValueView::Scalar(
            worth_foundational::facade::AspectValue::String(
                worth_foundational::facade::InternedString::Raw(actual),
            ),
        ) => assert_eq!(actual, expected_name),
        other => panic!("expected scalar string name aspect, got {other:?}"),
    }
}
