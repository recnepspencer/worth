//! The ordinary transactions a script is made of: creates, field updates,
//! relations between live entities, and deletes, over a two-kind schema.

pub(crate) use {std::collections::BTreeMap, std::collections::BTreeSet};

pub(crate) use {worth_foundational::facade::aspects, worth_foundational::facade::AspectFieldLocator, worth_foundational::facade::AspectIdentity, worth_foundational::facade::AspectKey, worth_foundational::facade::AspectValue, worth_foundational::facade::CanonicalFieldPath, worth_foundational::facade::FieldKey, worth_foundational::facade::LocatorAuthority, worth_foundational::facade::ScalarAspectType};
pub(crate) use {worth_relational::facade::config::CascadeDeletePolicy, worth_relational::facade::config::CrossContextPolicy, worth_relational::facade::identity::EntityId, worth_relational::facade::identity::KindId, worth_relational::facade::identity::PartitionId, worth_relational::facade::schema::AspectBinding, worth_relational::facade::schema::DeclaredAspectContractBinding, worth_relational::facade::schema::EntityKindRegistration, worth_relational::facade::schema::KindAspectContractDeclarations, worth_relational::facade::schema::RelationIntegrityDeclarations, worth_relational::facade::schema::RelationKindRegistration, worth_relational::facade::schema::RelationalSchemaRegistry, worth_relational::facade::schema::SchemaId, worth_relational::facade::schema::SchemaVersionId, worth_relational::facade::symbols::ClientKey, worth_relational::facade::transactions::AspectFieldPatch, worth_relational::facade::transactions::CreateIntent, worth_relational::facade::transactions::DeleteEntityIntent, worth_relational::facade::transactions::EntityMutationIntent, worth_relational::facade::transactions::EntityReference, worth_relational::facade::transactions::EntitySpec, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::RelationSpec, worth_relational::facade::transactions::UpdateEntityFieldsIntent, worth_relational::facade::transactions::WorkerIntentBatch};

pub(crate) use {crate::workload::Xorshift};

/// What one branch holds that the generator must respect: its live entities,
/// and the entity pairs it already relates (a pair relates at most once).
#[derive(Clone, Debug, Default)]
pub(crate) struct BranchLive {
    pub(crate) entities: Vec<EntityId>,
    pub(crate) related: BTreeSet<(EntityId, EntityId)>,
}

/// The next batch for a branch, and whether it creates an entity. A delete
/// removes its entity from the branch; a relation records its pair, and a
/// pair already related becomes an update instead.
pub(crate) fn next(
    generator: &mut Xorshift,
    live: &mut BranchLive,
    step: usize,
) -> (WorkerIntentBatch, bool) {
    let entities = &mut live.entities;
    let choice = if entities.len() < 2 {
        0
    } else {
        generator.next() % 8
    };
    let batch = match choice {
        0..=2 => create(step),
        3..=4 => update(entities[generator.index(entities.len())], step),
        5..=6 => {
            let source = entities[generator.index(entities.len())];
            let target = entities[generator.index(entities.len())];
            if live.related.insert((source, target)) {
                relate(source, target, step)
            } else {
                update(source, step)
            }
        }
        _ => delete(entities.swap_remove(generator.index(entities.len()))),
    };
    (batch, choice <= 2)
}

pub(crate) fn create(step: usize) -> WorkerIntentBatch {
    let name = format!("entity-{step}");
    worth_relational::facade::transactions::WorkerIntentBatch::new(format!("create-{step}")).push(worth_relational::facade::transactions::MutationIntent::Create(
        worth_relational::facade::transactions::CreateIntent::Entity(EntitySpec {
            partition_id: worth_relational::facade::identity::PartitionId::main(),
            kind_id: KindId(1),
            client_key: worth_relational::facade::symbols::ClientKey::raw(&name),
            fields: field_patch("name", &name),
        }),
    ))
}

pub(crate) fn update(entity_id: EntityId, step: usize) -> WorkerIntentBatch {
    worth_relational::facade::transactions::WorkerIntentBatch::new(format!("update-{step}")).push(worth_relational::facade::transactions::MutationIntent::Entity(
        worth_relational::facade::transactions::EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id,
            fields: field_patch("name", &format!("renamed-{step}")),
        }),
    ))
}

pub(crate) fn relate(source: EntityId, target: EntityId, step: usize) -> WorkerIntentBatch {
    let label = format!("link-{step}");
    worth_relational::facade::transactions::WorkerIntentBatch::new(format!("relate-{step}")).push(worth_relational::facade::transactions::MutationIntent::Create(
        worth_relational::facade::transactions::CreateIntent::Relation(RelationSpec {
            partition_id: worth_relational::facade::identity::PartitionId::main(),
            kind_id: KindId(2),
            client_key: worth_relational::facade::symbols::ClientKey::raw(&label),
            source: worth_relational::facade::transactions::EntityReference::Existing(source),
            target: worth_relational::facade::transactions::EntityReference::Existing(target),
            fields: field_patch("label", &label),
        }),
    ))
}

pub(crate) fn delete(entity_id: EntityId) -> WorkerIntentBatch {
    worth_relational::facade::transactions::WorkerIntentBatch::new("delete").push(worth_relational::facade::transactions::MutationIntent::Entity(worth_relational::facade::transactions::EntityMutationIntent::Delete(
        DeleteEntityIntent { entity_id },
    )))
}

/// The oracle schema at `version`. Every version has the same executable
/// meaning; a schema transition moves a branch between versions.
pub(crate) fn schema_registry(version: SchemaVersionId) -> RelationalSchemaRegistry {
    worth_relational::facade::schema::RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "oracle.entity".to_string(),
            schema_id: SchemaId("oracle".to_string()),
            schema_version_id: version,
            aspect_contract_declarations: worth_relational::facade::schema::KindAspectContractDeclarations::new(vec![
                DeclaredAspectContractBinding {
                    binding: worth_relational::facade::schema::AspectBinding::EntityField {
                        field: field_key("name"),
                    },
                    contract: string_contract("name"),
                },
            ]),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "oracle.relation".to_string(),
                schema_id: SchemaId("oracle".to_string()),
                schema_version_id: version,
                cross_context_policy: worth_relational::facade::config::CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: worth_relational::facade::config::CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: worth_relational::facade::schema::KindAspectContractDeclarations::new(vec![
                    DeclaredAspectContractBinding {
                        binding: worth_relational::facade::schema::AspectBinding::RelationField {
                            field: field_key("label"),
                        },
                        contract: string_contract("label"),
                    },
                ]),
                relation_integrity: worth_relational::facade::schema::RelationIntegrityDeclarations::default(),
            })
        })
        .expect("oracle schema registry")
}

pub(crate) fn field_patch(label: &str, value: &str) -> AspectFieldPatch {
    let mut fields = std::collections::BTreeMap::new();
    fields.insert(
        worth_foundational::facade::AspectFieldLocator::new(
            worth_foundational::facade::LocatorAuthority::Planned,
            aspect_key(label),
            worth_foundational::facade::CanonicalFieldPath::single(field_key(label)),
        ),
        worth_foundational::facade::AspectValue::String(value.to_string().into()),
    );
    worth_relational::facade::transactions::AspectFieldPatch::from(fields)
}

pub(crate) fn field_key(label: &str) -> FieldKey {
    worth_foundational::facade::FieldKey::new(label).expect("oracle field key")
}

pub(crate) fn aspect_key(label: &str) -> AspectKey {
    worth_foundational::facade::AspectKey::new(label).expect("oracle aspect key")
}

pub(crate) fn string_contract(label: &str) -> worth_foundational::AspectContract {
    let identity = label
        .bytes()
        .fold(14_695_981_039_346_656_037_u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(1_099_511_628_211)
        });
    aspects()
        .contract()
        .for_key(aspect_key(label))
        .identified_by(AspectIdentity(identity))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(worth_foundational::facade::ScalarAspectType::String)
}