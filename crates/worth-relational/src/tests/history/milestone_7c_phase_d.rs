use std::sync::Arc;

use crate::facade::history::BranchId;
use crate::facade::identity::KindId;
use crate::facade::merge::{
    AspectMergePolicyDeclaration, AspectMergePolicyKind, IdentityBasisDeclaration,
    IdentityBasisKind, IdentityBasisScope, MergeExecutionRequest, MergeIntent,
};
use crate::facade::runtime::RelationalRuntimeApi;
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::facade::transactions::{
    AspectFieldLocator, CreateIntent, EntityMutationIntent, MutationIntent, TransactionId,
};
use crate::tests::support::{
    create_branch_from_main, create_entity, create_entity_outcome_on_branch, entity_field_aspect,
    persisted_runtime_with_test_schema, runtime_with_test_schema, update_entity,
    CascadeDeletePolicy, CrossContextPolicy,
};
use worth_foundational::facade::{AspectKey, AspectValue, InternedString};

fn test_patch_target(aspect: &str, field: &str) -> AspectFieldLocator {
    crate::transactions::data::planned_single_field_locator(
        crate::tests::support::aspect_key(aspect),
        crate::tests::support::field_key(field),
    )
}

#[test]
fn derive_merge_commit_mutation_plan_emits_source_authorized_create_intent() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity(&runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(&runtime, "feature-only", BranchId("feature".to_string()));

    let prepared = runtime
        .prepare_merge_execution(MergeExecutionRequest {
            target_branch: BranchId("main".to_string()),
            source_branch: BranchId("feature".to_string()),
            merge_intent: MergeIntent::ReconcileIntoTarget,
        })
        .expect("prepared merge");

    let plan = prepared.bind_mutation_plan_for_test(TransactionId(77));

    assert_eq!(plan.transaction_id, TransactionId(77));
    assert_eq!(plan.structural_summary.executed_record_count, 1);
    assert_eq!(plan.structural_summary.adopted_source_record_count, 1);
    assert_eq!(plan.structural_summary.emitted_entity_create_count, 1);
    assert_eq!(plan.merged_plan.merged_intents.len(), 1);
    match &plan.merged_plan.merged_intents[0] {
        MutationIntent::Create(CreateIntent::Entity(spec)) => {
            assert_eq!(spec.kind_id, KindId(1));
            assert_eq!(
                spec.fields.get(&test_patch_target("name", "name")),
                Some(&AspectValue::String(InternedString::Raw(
                    "feature-only".to_string()
                )))
            );
        }
        other => panic!("expected entity create intent, got {other:?}"),
    }
}

#[test]
fn derive_merge_commit_mutation_plan_preserves_exact_shared_truth_without_mutation() {
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                ),
            ]),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build();
    let shared = create_entity(&runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity(&runtime, shared, "same");
    crate::tests::support::update_entity_on_branch(
        &runtime,
        shared,
        "same",
        BranchId("feature".to_string()),
    );

    let prepared = runtime
        .prepare_merge_execution(MergeExecutionRequest {
            target_branch: BranchId("main".to_string()),
            source_branch: BranchId("feature".to_string()),
            merge_intent: MergeIntent::ReconcileIntoTarget,
        })
        .expect("prepared merge");

    let plan = prepared.bind_mutation_plan_for_test(TransactionId(88));

    assert_eq!(plan.structural_summary.preserved_shared_record_count, 1);
    assert_eq!(plan.structural_summary.emitted_mutation_intent_count, 0);
    assert!(plan.merged_plan.merged_intents.is_empty());
}

#[test]
fn derive_merge_commit_mutation_plan_reconciles_target_with_source_authorized_aspects() {
    let name_key = AspectKey::new("name").unwrap();
    let status_key = AspectKey::new("status").unwrap();
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                ),
                entity_field_aspect(
                    crate::tests::support::aspect_key("status"),
                    crate::tests::support::field_key("status"),
                ),
            ])
            .with_identity_declarations(vec![IdentityBasisDeclaration {
                scope: IdentityBasisScope::AspectKey(name_key.clone()),
                basis: IdentityBasisKind::DeclaredKeySet(Arc::from([name_key.clone()])),
            }])
            .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                aspect_key: status_key.clone(),
                policy: AspectMergePolicyKind::PreferRicher,
            }]),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build();

    let mut main_txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime);
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    main_txn
        .push_batch(
            crate::facade::transactions::WorkerIntentBatch::new("main-seed").push(
                MutationIntent::Create(CreateIntent::Entity(
                    crate::transactions::data::EntitySpec {
                        partition_id: crate::facade::identity::PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: crate::symbols::data::ClientKey::raw("main-shared"),
                        fields: crate::tests::support::single_string_aspect_field_patch(
                            crate::tests::support::aspect_key("name"),
                            crate::tests::support::field_key("name"),
                            "shared-name",
                        ),
                    },
                )),
            ),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    main_txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    create_branch_from_main(&runtime, "feature");
    let mut feature_txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_branch(
                &runtime,
                BranchId("feature".to_string()),
            );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    feature_txn
        .push_batch(
            crate::facade::transactions::WorkerIntentBatch::new("feature-seed").push(
                MutationIntent::Create(CreateIntent::Entity(
                    crate::transactions::data::EntitySpec {
                        partition_id: crate::facade::identity::PartitionId::main(),
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
                    },
                )),
            ),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    feature_txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    let prepared = runtime
        .prepare_merge_execution(MergeExecutionRequest {
            target_branch: BranchId("main".to_string()),
            source_branch: BranchId("feature".to_string()),
            merge_intent: MergeIntent::ReconcileIntoTarget,
        })
        .expect("prepared merge");

    let plan = prepared.bind_mutation_plan_for_test(TransactionId(99));

    assert_eq!(plan.structural_summary.reconciled_record_count, 1);
    assert_eq!(plan.structural_summary.emitted_entity_update_count, 1);
    assert_eq!(plan.merged_plan.merged_intents.len(), 1);
    match &plan.merged_plan.merged_intents[0] {
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(intent)) => {
            assert_eq!(
                intent.fields.get(&test_patch_target("status", "status")),
                Some(&AspectValue::String("active".into()))
            );
            assert_eq!(intent.fields.get(&test_patch_target("name", "name")), None);
        }
        other => panic!("expected entity field update intent, got {other:?}"),
    }
}

#[test]
fn derive_merge_commit_mutation_plan_does_not_rely_on_raw_lowered_record_arrays() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(&runtime, "feature-only", BranchId("feature".to_string()));

    let mut prepared = runtime
        .prepare_merge_execution(MergeExecutionRequest {
            target_branch: BranchId("main".to_string()),
            source_branch: BranchId("feature".to_string()),
            merge_intent: MergeIntent::ReconcileIntoTarget,
        })
        .expect("prepared merge");
    prepared.execution_ready_plan_mut_for_test().lowered_records = Arc::from([]);

    let plan = prepared.bind_mutation_plan_for_test(TransactionId(123));

    assert_eq!(plan.structural_summary.executed_record_count, 1);
    assert_eq!(plan.merged_plan.merged_intents.len(), 1);
}
