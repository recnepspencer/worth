#![allow(dead_code)]

use std::collections::BTreeMap;

use worth_foundational::facade::{
    aspects, AspectFieldLocator, AspectIdentity, AspectKey, AspectValue, CanonicalFieldPath,
    FieldKey, LocatorAuthority, ScalarAspectType,
};
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    config::{CascadeDeletePolicy, CrossContextPolicy},
    history::BranchId,
    identity::{EntityId, KindId, PartitionId, RelationId},
    schema::{
        AspectBinding, DeclaredAspectContractBinding, EntityKindRegistration,
        KindAspectContractDeclarations, RelationIntegrityDeclarations, RelationKindRegistration,
        RelationalSchemaRegistry, SchemaId, SchemaVersionId,
    },
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, CreateIntent, DeleteEntityIntent, EntityMutationIntent, EntityReference,
        EntitySpec, MutationIntent, RelationSpec, UpdateEntityFieldsIntent, WorkerIntentBatch,
    },
};

fn main_basis(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
) -> AdmittedRelationalBranchBasis {
    let identity = runtime.main_branch_identity();
    runtime
        .observe_branch(&identity)
        .map(|(_, basis)| basis)
        .expect("configured main branch must remain owner-observable")
}

pub fn demo_schema_registry() -> RelationalSchemaRegistry {
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "demo.entity".to_string(),
            schema_id: SchemaId("demo".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_string_field_aspect(aspect_key("name"), field_key("name")),
            ]),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "demo.relation".to_string(),
                schema_id: SchemaId("demo".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                    relation_string_field_aspect(aspect_key("label"), field_key("label")),
                ]),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .expect("demo schema registry")
}

fn main() {
    // This example is a shared helper module for other examples.
}

pub fn create_entity(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    name: &str,
) -> (
    worth_relational::facade::transactions::CommitResult,
    EntityId,
) {
    let mut tx = {
        let basis = main_basis(runtime);
        runtime
            .begin_branch_transaction(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("owner-admitted transaction context")
    };
    tx.push_batch(
        WorkerIntentBatch::new(format!("create-{name}")).push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw(name),
                fields: string_field_patch(aspect_key("name"), field_key("name"), name),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("example entity batch must stage");
    let outcome = tx
        .commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("entity commit");
    let entity_id = changed_entity(&outcome).expect("created entity id");
    (outcome, entity_id)
}

pub fn update_entity(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    entity_id: EntityId,
    name: &str,
) -> worth_relational::facade::transactions::CommitResult {
    update_entity_on_branch(runtime, entity_id, name, None)
}

pub fn update_entity_on_branch(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    entity_id: EntityId,
    name: &str,
    target_branch: Option<BranchId>,
) -> worth_relational::facade::transactions::CommitResult {
    let basis = target_branch.map_or_else(
        || main_basis(runtime),
        |branch| {
            let identity = runtime
                .branch_identity(&branch)
                .expect("example branch must be owner-registered");
            runtime
                .admit_branch_basis(&identity)
                .expect("example branch identity must be owner-admitted")
        },
    );
    let mut tx = {
        runtime
            .begin_branch_transaction(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("owner-admitted transaction context")
    };
    tx.push_batch(
        WorkerIntentBatch::new(format!("update-{name}")).push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id,
                fields: string_field_patch(aspect_key("name"), field_key("name"), name),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("example update batch must stage");
    tx.commit(
        runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("update commit")
}

pub fn delete_entity(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    entity_id: EntityId,
) -> worth_relational::facade::transactions::CommitResult {
    let mut tx = {
        let basis = main_basis(runtime);
        runtime
            .begin_branch_transaction(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("owner-admitted transaction context")
    };
    tx.push_batch(
        WorkerIntentBatch::new("delete-entity").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("example delete batch must stage");
    tx.commit(
        runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("delete entity commit")
}

pub fn create_relation(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    source: EntityId,
    target: EntityId,
    label: &str,
) -> (
    worth_relational::facade::transactions::CommitResult,
    RelationId,
) {
    let mut tx = {
        let basis = main_basis(runtime);
        runtime
            .begin_branch_transaction(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("owner-admitted transaction context")
    };
    tx.push_batch(
        WorkerIntentBatch::new(format!("rel-{label}")).push(MutationIntent::Create(
            CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw(label),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
                fields: string_field_patch(aspect_key("label"), field_key("label"), label),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("example relation batch must stage");
    let outcome = tx
        .commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("relation commit");
    let relation_id = changed_relation(&outcome).expect("created relation id");
    (outcome, relation_id)
}

pub fn changed_entity(
    outcome: &worth_relational::facade::transactions::CommitResult,
) -> Option<EntityId> {
    outcome
        .changed_records
        .iter()
        .find_map(|record| match record {
            worth_relational::facade::transactions::RecordRef::Entity(entity_id) => {
                Some(*entity_id)
            }
            worth_relational::facade::transactions::RecordRef::Relation(_) => None,
        })
}

pub fn changed_relation(
    outcome: &worth_relational::facade::transactions::CommitResult,
) -> Option<RelationId> {
    outcome
        .changed_records
        .iter()
        .find_map(|record| match record {
            worth_relational::facade::transactions::RecordRef::Relation(relation_id) => {
                Some(*relation_id)
            }
            worth_relational::facade::transactions::RecordRef::Entity(_) => None,
        })
}

pub fn field_key(label: &str) -> FieldKey {
    FieldKey::new(label).expect("example field key must be foundational")
}

pub fn aspect_field_locator(label: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        aspect_key(label),
        CanonicalFieldPath::single(field_key(label)),
    )
}

fn string_field_patch(aspect_key: AspectKey, field_key: FieldKey, value: &str) -> AspectFieldPatch {
    let mut fields = BTreeMap::new();
    fields.insert(
        AspectFieldLocator::new(
            LocatorAuthority::Planned,
            aspect_key,
            CanonicalFieldPath::single(field_key),
        ),
        AspectValue::String(value.to_string().into()),
    );
    AspectFieldPatch::from(fields)
}

fn aspect_key(label: &str) -> AspectKey {
    AspectKey::new(label).expect("example aspect key must be foundational")
}

fn entity_string_field_aspect(
    aspect_key: AspectKey,
    field_key: FieldKey,
) -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding {
        binding: AspectBinding::EntityField { field: field_key },
        contract: scalar_string_contract(aspect_key),
    }
}

fn relation_string_field_aspect(
    aspect_key: AspectKey,
    field_key: FieldKey,
) -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding {
        binding: AspectBinding::RelationField { field: field_key },
        contract: scalar_string_contract(aspect_key),
    }
}

fn scalar_string_contract(aspect_key: AspectKey) -> worth_foundational::AspectContract {
    let identity = stable_contract_identity(&aspect_key);
    aspects()
        .contract()
        .for_key(aspect_key)
        .identified_by(AspectIdentity(identity))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(ScalarAspectType::String)
}

fn stable_contract_identity(aspect_key: &AspectKey) -> u64 {
    let mut hash = 14695981039346656037_u64;
    for byte in aspect_key.as_str().as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211_u64);
    }
    hash
}
