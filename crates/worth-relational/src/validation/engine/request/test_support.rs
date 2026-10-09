use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::facade::{
    runtime::RelationalRuntimeApi,
    schema::{
        EntityKindRegistration, KindAspectContractDeclarations, RelationKindRegistration,
        RelationalSchemaRegistry, SchemaId, SchemaVersionId,
    },
};
use crate::identity::data::{KindId, PartitionId};
use crate::schema::data::{EndpointKindContractDeclaration, RelationIntegrityDeclarations};
use crate::transactions::data::EntityReference;
use crate::transactions::data::{
    CreateIntent, EntitySpec, MergedCommitPlan, MutationIntent, RelationSpec, WorkerIntentBatch,
};
use crate::validation::data::InvariantPlanContract;
use crate::validation::engine::{
    InvariantExecutionRequest, InvariantObservation, InvariantRequestProfile,
};

pub(super) fn relation_integrity_runtime() -> crate::runtime::RelationalRuntime {
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.edge.a".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::new(
                    vec![EndpointKindContractDeclaration {
                        contract_id: "kind2".into(),
                        allowed_source_kinds: vec![KindId(1)],
                        allowed_target_kinds: vec![KindId(1)],
                        self_edges_allowed: false,
                        cross_context_policy: CrossContextPolicy::AllowExplicit,
                    }],
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
            })
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(3),
                kind_name: "test.edge.b".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::new(
                    vec![EndpointKindContractDeclaration {
                        contract_id: "kind3".into(),
                        allowed_source_kinds: vec![KindId(1)],
                        allowed_target_kinds: vec![KindId(1)],
                        self_edges_allowed: false,
                        cross_context_policy: CrossContextPolicy::AllowExplicit,
                    }],
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
            })
        })
        .unwrap();
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
}

pub(super) fn create_relation_of_kind(
    runtime: &crate::runtime::RelationalRuntime,
    kind_id: KindId,
    source: crate::identity::data::EntityId,
    target: crate::identity::data::EntityId,
    client_key: &str,
) -> crate::identity::data::RelationId {
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    txn.push_batch(
        WorkerIntentBatch::new(format!("relation-{client_key}")).push(MutationIntent::Create(
            CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id,
                client_key: crate::symbols::data::ClientKey::raw(client_key),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    outcome
        .changed_records
        .iter()
        .find_map(|record| match record {
            crate::facade::transactions::RecordRef::Relation(relation_id) => Some(*relation_id),
            crate::facade::transactions::RecordRef::Entity(_) => None,
        })
        .expect("created relation")
}

pub(super) fn create_entity(
    runtime: &crate::runtime::RelationalRuntime,
    name: &str,
) -> crate::identity::data::EntityId {
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    txn.push_batch(
        WorkerIntentBatch::new(format!("entity-{name}")).push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw(name),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    outcome
        .changed_records
        .iter()
        .find_map(|record| match record {
            crate::facade::transactions::RecordRef::Entity(entity_id) => Some(*entity_id),
            crate::facade::transactions::RecordRef::Relation(_) => None,
        })
        .expect("created entity")
}

pub(super) fn request_for_plan<'runtime>(
    runtime: &'runtime crate::runtime::RelationalRuntime,
    plan: &'runtime MergedCommitPlan,
) -> InvariantExecutionRequest<'runtime> {
    InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        Some(plan),
        Some(InvariantPlanContract::from_merged_plan(plan)),
    )
}
