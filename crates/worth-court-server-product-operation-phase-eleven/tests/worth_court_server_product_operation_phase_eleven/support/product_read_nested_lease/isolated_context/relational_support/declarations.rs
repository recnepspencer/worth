#![allow(dead_code)]

pub(crate) use {std::collections::BTreeMap};

pub(crate) use {worth_foundational::facade::aspects, worth_foundational::facade::AspectFieldLocator, worth_foundational::facade::AspectIdentity, worth_foundational::facade::AspectKey, worth_foundational::facade::AspectValue, worth_foundational::facade::CanonicalFieldPath, worth_foundational::facade::FieldKey, worth_foundational::facade::LocatorAuthority, worth_foundational::facade::ScalarAspectType};
pub(crate) use {worth_relational::facade::branch::AdmittedRelationalBranchBasis, worth_relational::facade::config::CascadeDeletePolicy, worth_relational::facade::config::CrossContextPolicy, worth_relational::facade::history::BranchId, worth_relational::facade::identity::EntityId, worth_relational::facade::identity::KindId, worth_relational::facade::identity::PartitionId, worth_relational::facade::identity::RelationId, worth_relational::facade::schema::AspectBinding, worth_relational::facade::schema::DeclaredAspectContractBinding, worth_relational::facade::schema::EntityKindRegistration, worth_relational::facade::schema::KindAspectContractDeclarations, worth_relational::facade::schema::RelationIntegrityDeclarations, worth_relational::facade::schema::RelationKindRegistration, worth_relational::facade::schema::RelationalSchemaRegistry, worth_relational::facade::schema::SchemaId, worth_relational::facade::schema::SchemaVersionId, worth_relational::facade::symbols::ClientKey, worth_relational::facade::transactions::AspectFieldPatch, worth_relational::facade::transactions::CreateIntent, worth_relational::facade::transactions::DeleteEntityIntent, worth_relational::facade::transactions::EntityMutationIntent, worth_relational::facade::transactions::EntityReference, worth_relational::facade::transactions::EntitySpec, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::RelationSpec, worth_relational::facade::transactions::UpdateEntityFieldsIntent, worth_relational::facade::transactions::WorkerIntentBatch};

pub(crate) fn main_basis(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
) -> AdmittedRelationalBranchBasis {
    let identity = runtime.main_branch_identity();
    runtime
        .observe_branch(&identity)
        .map(|(_, basis)| basis)
        .expect("configured main branch must remain owner-observable")
}

pub(crate) fn demo_schema_registry() -> RelationalSchemaRegistry {
    worth_relational::facade::schema::RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "demo.entity".to_string(),
            schema_id: SchemaId("demo".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: worth_relational::facade::schema::KindAspectContractDeclarations::new(vec![
                entity_string_field_aspect(aspect_key("name"), field_key("name")),
            ]),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "demo.relation".to_string(),
                schema_id: SchemaId("demo".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: worth_relational::facade::config::CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: worth_relational::facade::config::CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: worth_relational::facade::schema::KindAspectContractDeclarations::new(vec![
                    relation_string_field_aspect(aspect_key("label"), field_key("label")),
                ]),
                relation_integrity: worth_relational::facade::schema::RelationIntegrityDeclarations::default(),
            })
        })
        .expect("demo schema registry")
}

pub(crate) fn main() {
    // This example is a shared helper module for other examples.
}

pub(crate) fn create_entity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        worth_relational::facade::transactions::WorkerIntentBatch::new(format!("create-{name}")).push(worth_relational::facade::transactions::MutationIntent::Create(
            worth_relational::facade::transactions::CreateIntent::Entity(EntitySpec {
                partition_id: worth_relational::facade::identity::PartitionId::main(),
                kind_id: KindId(1),
                client_key: worth_relational::facade::symbols::ClientKey::raw(name),
                fields: string_field_patch(aspect_key("name"), field_key("name"), name),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("example entity batch must stage");
    let outcome = tx.commit(execution, runtime).expect("entity commit");
    let entity_id = changed_entity(&outcome).expect("created entity id");
    (outcome, entity_id)
}

pub(crate) fn update_entity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    entity_id: EntityId,
    name: &str,
) -> worth_relational::facade::transactions::CommitResult {
    update_entity_on_branch(execution, runtime, entity_id, name, None)
}

pub(crate) fn update_entity_on_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        worth_relational::facade::transactions::WorkerIntentBatch::new(format!("update-{name}")).push(worth_relational::facade::transactions::MutationIntent::Entity(
            worth_relational::facade::transactions::EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id,
                fields: string_field_patch(aspect_key("name"), field_key("name"), name),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("example update batch must stage");
    tx.commit(execution, runtime).expect("update commit")
}

pub(crate) fn delete_entity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        worth_relational::facade::transactions::WorkerIntentBatch::new("delete-entity").push(worth_relational::facade::transactions::MutationIntent::Entity(
            worth_relational::facade::transactions::EntityMutationIntent::Delete(DeleteEntityIntent { entity_id }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("example delete batch must stage");
    tx.commit(execution, runtime).expect("delete entity commit")
}

pub(crate) fn create_relation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        worth_relational::facade::transactions::WorkerIntentBatch::new(format!("rel-{label}")).push(worth_relational::facade::transactions::MutationIntent::Create(
            worth_relational::facade::transactions::CreateIntent::Relation(RelationSpec {
                partition_id: worth_relational::facade::identity::PartitionId::main(),
                kind_id: KindId(2),
                client_key: worth_relational::facade::symbols::ClientKey::raw(label),
                source: worth_relational::facade::transactions::EntityReference::Existing(source),
                target: worth_relational::facade::transactions::EntityReference::Existing(target),
                fields: string_field_patch(aspect_key("label"), field_key("label"), label),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("example relation batch must stage");
    let outcome = tx.commit(execution, runtime).expect("relation commit");
    let relation_id = changed_relation(&outcome).expect("created relation id");
    (outcome, relation_id)
}

pub(crate) fn changed_entity(
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

pub(crate) fn changed_relation(
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

pub(crate) fn field_key(label: &str) -> FieldKey {
    worth_foundational::facade::FieldKey::new(label).expect("example field key must be foundational")
}

pub(crate) fn aspect_field_locator(label: &str) -> AspectFieldLocator {
    worth_foundational::facade::AspectFieldLocator::new(
        worth_foundational::facade::LocatorAuthority::Planned,
        aspect_key(label),
        worth_foundational::facade::CanonicalFieldPath::single(field_key(label)),
    )
}

pub(crate) fn string_field_patch(aspect_key: AspectKey, field_key: FieldKey, value: &str) -> AspectFieldPatch {
    let mut fields = std::collections::BTreeMap::new();
    fields.insert(
        worth_foundational::facade::AspectFieldLocator::new(
            worth_foundational::facade::LocatorAuthority::Planned,
            aspect_key,
            worth_foundational::facade::CanonicalFieldPath::single(field_key),
        ),
        worth_foundational::facade::AspectValue::String(value.to_string().into()),
    );
    worth_relational::facade::transactions::AspectFieldPatch::from(fields)
}

pub(crate) fn aspect_key(label: &str) -> AspectKey {
    worth_foundational::facade::AspectKey::new(label).expect("example aspect key must be foundational")
}

pub(crate) fn entity_string_field_aspect(
    aspect_key: AspectKey,
    field_key: FieldKey,
) -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding {
        binding: worth_relational::facade::schema::AspectBinding::EntityField { field: field_key },
        contract: scalar_string_contract(aspect_key),
    }
}

pub(crate) fn relation_string_field_aspect(
    aspect_key: AspectKey,
    field_key: FieldKey,
) -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding {
        binding: worth_relational::facade::schema::AspectBinding::RelationField { field: field_key },
        contract: scalar_string_contract(aspect_key),
    }
}

pub(crate) fn scalar_string_contract(aspect_key: AspectKey) -> worth_foundational::AspectContract {
    let identity = stable_contract_identity(&aspect_key);
    aspects()
        .contract()
        .for_key(aspect_key)
        .identified_by(AspectIdentity(identity))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(worth_foundational::facade::ScalarAspectType::String)
}

pub(crate) fn stable_contract_identity(aspect_key: &AspectKey) -> u64 {
    let mut hash = 14695981039346656037_u64;
    for byte in aspect_key.as_str().as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211_u64);
    }
    hash
}

pub(crate) fn serial_request() -> worth_execution::SerialRequest {
    worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        worth_execution::CancellationSource::new().token(),
        None,
    )
}