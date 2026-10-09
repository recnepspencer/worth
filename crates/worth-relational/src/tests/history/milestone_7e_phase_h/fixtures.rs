use crate::facade::history::BranchId;
use crate::facade::identity::KindId;
use crate::facade::merge::{
    AspectMergePolicyDeclaration, AspectMergePolicyKind, IdentityBasisDeclaration,
    IdentityBasisKind, IdentityBasisScope, MergeExecutionRequest, MergeIntent,
    MergeManualResolutionClass, MergePlanningRequest, MergePolicyDecisionBoundary,
    MergePolicyRejectClass, RelationalSchemaReconciliationBasisRow,
    RelationalSchemaReconciliationCorrespondenceLinkRow, RelationalSchemaReconciliationWitness,
    RelationalSchemaReconciliationWitnessRow,
};
use crate::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::facade::transactions::{CreateIntent, MutationIntent, WorkerIntentBatch};
use crate::merge::data::RelationalSchemaReconciliationWitnessRowInput;
use crate::tests::support::{
    aspect_key, entity_field_aspect, field_key, relation_field_aspect, relation_source_aspect,
    relation_target_aspect, string_aspect_field_patch, unique_test_store_path, CascadeDeletePolicy,
    CrossContextPolicy, DurabilityMode, DurableStoreLayout,
};
use crate::transactions::data::{PublishedMergeExecutionAuthority, RecordRef};
use std::sync::Arc;

pub(super) fn additive_row() -> RelationalSchemaReconciliationWitnessRow {
    retained_row(
        RecordRef::Entity(crate::facade::identity::EntityId::new(
            crate::facade::identity::PartitionId::main(),
            1,
            1,
        )),
        Some(RecordRef::Entity(crate::facade::identity::EntityId::new(
            crate::facade::identity::PartitionId::main(),
            1,
            1,
        ))),
        1,
        0,
        0,
        0,
        MergePolicyDecisionBoundary::AutoResolved,
        false,
    )
}

pub(super) fn denied_narrowing_row() -> RelationalSchemaReconciliationWitnessRow {
    retained_row(
        RecordRef::Entity(crate::facade::identity::EntityId::new(
            crate::facade::identity::PartitionId::main(),
            2,
            1,
        )),
        Some(RecordRef::Entity(crate::facade::identity::EntityId::new(
            crate::facade::identity::PartitionId::main(),
            2,
            1,
        ))),
        0,
        1,
        0,
        0,
        MergePolicyDecisionBoundary::RequiresManualResolution {
            class: MergeManualResolutionClass::UnvalidatedSchemaCorrespondence,
        },
        false,
    )
}

pub(super) fn type_incompatible_row() -> RelationalSchemaReconciliationWitnessRow {
    retained_row(
        RecordRef::Entity(crate::facade::identity::EntityId::new(
            crate::facade::identity::PartitionId::main(),
            3,
            1,
        )),
        Some(RecordRef::Entity(crate::facade::identity::EntityId::new(
            crate::facade::identity::PartitionId::main(),
            3,
            1,
        ))),
        0,
        0,
        1,
        0,
        MergePolicyDecisionBoundary::Reject {
            class: MergePolicyRejectClass::BuiltInFailOnConflict,
        },
        false,
    )
}

pub(super) fn structural_incompatible_row() -> RelationalSchemaReconciliationWitnessRow {
    retained_row(
        RecordRef::Entity(crate::facade::identity::EntityId::new(
            crate::facade::identity::PartitionId::main(),
            4,
            1,
        )),
        Some(RecordRef::Entity(crate::facade::identity::EntityId::new(
            crate::facade::identity::PartitionId::main(),
            4,
            1,
        ))),
        0,
        0,
        0,
        0,
        MergePolicyDecisionBoundary::AutoResolved,
        true,
    )
}

pub(super) fn synthetic_witness(
    rows: Vec<RelationalSchemaReconciliationWitnessRow>,
) -> RelationalSchemaReconciliationWitness {
    RelationalSchemaReconciliationWitness::retained(
        "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd".to_string(),
        "efefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefef".to_string(),
        Arc::from(rows),
    )
}

pub(super) fn runtime_with_schema_declared_entity_policy(
    policy: AspectMergePolicyKind,
) -> RelationalRuntime {
    runtime_with_schema_declared_entity_policy_builder(policy, None)
}

pub(super) fn persisted_runtime_with_schema_declared_entity_policy(
    policy: AspectMergePolicyKind,
) -> RelationalRuntime {
    runtime_with_schema_declared_entity_policy_builder(
        policy,
        Some(unique_test_store_path("worth-relational-7e-phase-h-schema")),
    )
}

pub(super) fn runtime_with_relation_identity_registry(
    root_path: std::path::PathBuf,
) -> RelationalRuntime {
    let label_key = aspect_key("label");
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
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                    relation_field_aspect(label_key.clone(), field_key("label")),
                    relation_source_aspect(),
                    relation_target_aspect(),
                ])
                .with_identity_declarations(vec![IdentityBasisDeclaration {
                    scope: IdentityBasisScope::AspectKey(label_key.clone()),
                    basis: IdentityBasisKind::DeclaredKeySet(Arc::from([label_key])),
                }]),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .expect("topology identity registry");

    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(DurableStoreLayout {
            root_path,
            segment_commit_capacity: 2,
        })
        .build()
}

pub(super) fn create_named_entity_on_branch(
    runtime: &RelationalRuntime,
    client_key: &str,
    name: &str,
    status: Option<&str>,
    branch: &str,
) {
    let mut fields = vec![(aspect_key("name"), field_key("name"), name)];
    if let Some(status) = status {
        fields.push((aspect_key("status"), field_key("status"), status));
    }
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_branch(
        runtime,
        BranchId(branch.to_string()),
    );
    txn.push_batch(
        WorkerIntentBatch::new(format!("seed-{client_key}")).push(MutationIntent::Create(
            CreateIntent::Entity(crate::transactions::data::EntitySpec {
                partition_id: crate::facade::identity::PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw(client_key),
                fields: string_aspect_field_patch(fields),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(
        runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("seed entity");
}

pub(super) fn merge_request() -> MergeExecutionRequest {
    MergeExecutionRequest::new(
        BranchId("main".to_string()),
        BranchId("feature".to_string()),
        MergeIntent::ReconcileIntoTarget,
    )
}

pub(super) fn merge_planning_request() -> MergePlanningRequest {
    merge_request().into()
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

fn retained_row(
    record: RecordRef,
    target_record: Option<RecordRef>,
    source_only_aspect_count: usize,
    target_only_aspect_count: usize,
    divergent_aspect_count: usize,
    unavailable_aspect_count: usize,
    decision_boundary: MergePolicyDecisionBoundary,
    relation_endpoint_divergence: bool,
) -> RelationalSchemaReconciliationWitnessRow {
    RelationalSchemaReconciliationWitnessRow::retained(
        RelationalSchemaReconciliationWitnessRowInput {
            record: record.clone(),
            target_record: target_record.clone(),
            basis: RelationalSchemaReconciliationBasisRow {
                source_kind_id: Some(KindId(1)),
                target_kind_id: Some(KindId(1)),
                source_schema_id: Some(SchemaId("test".to_string())),
                source_schema_version_id: Some(SchemaVersionId(1)),
                target_schema_id: Some(SchemaId("test".to_string())),
                target_schema_version_id: Some(SchemaVersionId(1)),
                registry_digest: "abababababababababababababababababababababababababababababababab"
                    .to_string(),
            },
            source_only_aspect_count,
            target_only_aspect_count,
            divergent_aspect_count,
            unavailable_aspect_count,
            decision_boundary,
            relation_endpoint_divergence,
            correspondence_linkage: target_record.map(|target| {
                RelationalSchemaReconciliationCorrespondenceLinkRow {
                    scope: IdentityBasisScope::EntityKind(KindId(1)),
                    basis: IdentityBasisKind::StorageIdentity,
                    source_record: record,
                    target_record: target,
                }
            }),
        },
    )
}

fn runtime_with_schema_declared_entity_policy_builder(
    policy: AspectMergePolicyKind,
    root_path: Option<std::path::PathBuf>,
) -> RelationalRuntime {
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
                basis: IdentityBasisKind::DeclaredKeySet(Arc::from([name_key])),
            }])
            .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                aspect_key: status_key,
                policy,
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
        .expect("schema-declared merge policy registry");
    let mut builder = RelationalRuntimeApi::builder().schema_registry(registry);
    if let Some(root_path) = root_path {
        builder = builder
            .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
            .durable_store_layout(DurableStoreLayout {
                root_path,
                segment_commit_capacity: 2,
            });
    }
    builder.build()
}
