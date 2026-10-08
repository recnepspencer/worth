use crate::facade::history::BranchId;
use crate::facade::identity::KindId;
use crate::facade::merge::{
    AspectMergePolicyDeclaration, AspectMergePolicyKind, IdentityBasisDeclaration,
    IdentityBasisKind, IdentityBasisScope, MergeExecutionRequest, MergeIntent,
};
use crate::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::merge::data::PreparedMergeExecution;
use crate::tests::support::{
    aspect_key, create_branch_from_main, create_entity, entity_field_aspect, field_key,
    string_aspect_field_patch, unique_test_store_path, DurabilityMode, DurableStoreLayout,
};
use crate::transactions::data::PublishedMergeExecutionAuthority;

pub(super) fn merge_request() -> MergeExecutionRequest {
    MergeExecutionRequest::new(
        BranchId("main".to_string()),
        BranchId("feature".to_string()),
        MergeIntent::ReconcileIntoTarget,
    )
}

pub(super) fn runtime_with_schema_declared_entity_policy(persisted: bool) -> RelationalRuntime {
    let name_key = aspect_key("name");
    let status_key = aspect_key("status");
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(name_key.clone(), field_key("name")),
                entity_field_aspect(status_key.clone(), field_key("status")),
            ])
            .with_identity_declarations(vec![IdentityBasisDeclaration {
                scope: IdentityBasisScope::AspectKey(name_key.clone()),
                basis: IdentityBasisKind::DeclaredKeySet(std::sync::Arc::from([name_key])),
            }])
            .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                aspect_key: status_key,
                policy: AspectMergePolicyKind::PreferRicher,
            }]),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: crate::tests::support::CrossContextPolicy::AllowExplicit,
                cascade_delete_policy:
                    crate::tests::support::CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .expect("schema registry");
    let mut builder = RelationalRuntimeApi::builder().schema_registry(registry);
    if persisted {
        builder = builder
            .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
            .durable_store_layout(DurableStoreLayout {
                root_path: unique_test_store_path("worth-relational-7e-phase-l-support"),
                segment_commit_capacity: 2,
            });
    }
    builder.build()
}

pub(super) fn prepared_merge(runtime: &RelationalRuntime) -> PreparedMergeExecution {
    let shared = create_entity(runtime, "shared");
    create_branch_from_main(runtime, "feature");
    update_entity_status_on_branch(runtime, shared, "inactive", "main");
    update_entity_status_on_branch(runtime, shared, "active", "feature");
    runtime
        .merge()
        .prepare_merge_execution(merge_request())
        .expect("prepared merge")
}

fn update_entity_status_on_branch(
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
    status: &str,
    branch: &str,
) {
    let mut txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_branch(
                runtime,
                BranchId(branch.to_string()),
            );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(
        crate::facade::transactions::WorkerIntentBatch::new(format!("status-{branch}")).push(
            crate::facade::transactions::MutationIntent::Entity(
                crate::facade::transactions::EntityMutationIntent::UpdateFields(
                    crate::facade::transactions::UpdateEntityFieldsIntent {
                        entity_id,
                        fields: string_aspect_field_patch([(
                            aspect_key("status"),
                            field_key("status"),
                            status,
                        )]),
                    },
                ),
            ),
        ),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(
        runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("update status");
}

pub(super) fn published_merge_authority(
    runtime: &RelationalRuntime,
    commit_id: crate::facade::history::CommitId,
) -> PublishedMergeExecutionAuthority {
    runtime
        .replay()
        .canonical_commit_envelope(commit_id)
        .and_then(|envelope| envelope.merge_execution_authority.clone())
        .expect("published merge authority")
}
