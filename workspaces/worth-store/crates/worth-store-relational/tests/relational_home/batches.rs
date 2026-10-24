//! The ordinary transactions a script is made of: creates, field updates,
//! relations between live entities, and deletes, over a two-kind schema.

use std::collections::BTreeMap;

use worth_foundational::facade::{
    aspects, AspectFieldLocator, AspectIdentity, AspectKey, AspectValue, CanonicalFieldPath,
    FieldKey, LocatorAuthority, ScalarAspectType,
};
use worth_relational::facade::{
    config::{CascadeDeletePolicy, CrossContextPolicy},
    identity::{EntityId, KindId, PartitionId},
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

use super::workload::Xorshift;

/// The next batch for a branch whose live entities are `live`, and whether it
/// creates an entity. A delete removes its entity from `live`.
pub(super) fn next(
    generator: &mut Xorshift,
    live: &mut Vec<EntityId>,
    step: usize,
) -> (WorkerIntentBatch, bool) {
    let choice = if live.len() < 2 {
        0
    } else {
        generator.next() % 8
    };
    let batch = match choice {
        0..=2 => create(step),
        3..=4 => update(live[generator.index(live.len())], step),
        5..=6 => relate(
            live[generator.index(live.len())],
            live[generator.index(live.len())],
            step,
        ),
        _ => delete(live.swap_remove(generator.index(live.len()))),
    };
    (batch, choice <= 2)
}

fn create(step: usize) -> WorkerIntentBatch {
    let name = format!("entity-{step}");
    WorkerIntentBatch::new(format!("create-{step}")).push(MutationIntent::Create(
        CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw(&name),
            fields: field_patch("name", &name),
        }),
    ))
}

fn update(entity_id: EntityId, step: usize) -> WorkerIntentBatch {
    WorkerIntentBatch::new(format!("update-{step}")).push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id,
            fields: field_patch("name", &format!("renamed-{step}")),
        }),
    ))
}

fn relate(source: EntityId, target: EntityId, step: usize) -> WorkerIntentBatch {
    let label = format!("link-{step}");
    WorkerIntentBatch::new(format!("relate-{step}")).push(MutationIntent::Create(
        CreateIntent::Relation(RelationSpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: ClientKey::raw(&label),
            source: EntityReference::Existing(source),
            target: EntityReference::Existing(target),
            fields: field_patch("label", &label),
        }),
    ))
}

fn delete(entity_id: EntityId) -> WorkerIntentBatch {
    WorkerIntentBatch::new("delete").push(MutationIntent::Entity(EntityMutationIntent::Delete(
        DeleteEntityIntent { entity_id },
    )))
}

pub(super) fn schema_registry() -> RelationalSchemaRegistry {
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "oracle.entity".to_string(),
            schema_id: SchemaId("oracle".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                DeclaredAspectContractBinding {
                    binding: AspectBinding::EntityField {
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
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                    DeclaredAspectContractBinding {
                        binding: AspectBinding::RelationField {
                            field: field_key("label"),
                        },
                        contract: string_contract("label"),
                    },
                ]),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .expect("oracle schema registry")
}

fn field_patch(label: &str, value: &str) -> AspectFieldPatch {
    let mut fields = BTreeMap::new();
    fields.insert(
        AspectFieldLocator::new(
            LocatorAuthority::Planned,
            aspect_key(label),
            CanonicalFieldPath::single(field_key(label)),
        ),
        AspectValue::String(value.to_string().into()),
    );
    AspectFieldPatch::from(fields)
}

fn field_key(label: &str) -> FieldKey {
    FieldKey::new(label).expect("oracle field key")
}

fn aspect_key(label: &str) -> AspectKey {
    AspectKey::new(label).expect("oracle aspect key")
}

fn string_contract(label: &str) -> worth_foundational::AspectContract {
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
        .scalar(ScalarAspectType::String)
}
