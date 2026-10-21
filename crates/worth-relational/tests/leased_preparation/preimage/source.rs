//! Genuine aspect-backed records and ordinary transaction intents for preimage admission.
use super::super::*;
use worth_foundational::facade::{
    aspects, AspectFieldLocator, AspectIdentity, AspectKey, CanonicalFieldPath, FieldKey,
    LocatorAuthority, ScalarAspectType,
};
use worth_relational::facade::{
    identity::{EntityId, RelationId},
    mvcc::BranchBoundRelationalTransaction,
    runtime::RelationalRuntime,
    schema::{
        AspectBinding, DeclaredAspectContractBinding, EntityKindRegistration,
        KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
    },
    transactions::{
        CommitResult, EntityMutationIntent, EntityReference, RecordRef, RelationSpec,
        UpdateEntityFieldsIntent,
    },
};

#[derive(Clone, Copy, Debug)]
pub(super) enum Dependencies {
    Reads,
    Endpoints,
}

pub(super) struct Source {
    pub runtime: RelationalRuntime,
    pub entities: Vec<EntityId>,
    pub unrelated: EntityId,
    pub isolated: EntityId,
    pub relation: RelationId,
    pub original: CommitResult,
}

fn schema() -> RelationalSchemaRegistry {
    let shape = aspects()
        .struct_fields()
        .required("title", ScalarAspectType::String)
        .required("status", ScalarAspectType::String)
        .finish()
        .unwrap();
    support::demo_schema_registry()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(3),
            kind_name: "source.record".into(),
            schema_id: SchemaId("preimage".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                DeclaredAspectContractBinding {
                    binding: AspectBinding::EntityField {
                        field: FieldKey::new("facts").unwrap(),
                    },
                    contract: aspects()
                        .contract()
                        .for_key(AspectKey::new("facts").unwrap())
                        .identified_by(AspectIdentity(0x434f_0017))
                        .at_revision(aspects().vocabulary().revision(1))
                        .struct_aspect(shape),
                },
            ]),
        })
        .unwrap()
}

pub(super) fn facts(title: &str, initial: bool) -> AspectFieldPatch {
    let locator = |field| {
        AspectFieldLocator::new(
            LocatorAuthority::Planned,
            AspectKey::new("facts").unwrap(),
            CanonicalFieldPath::single(FieldKey::new(field).unwrap()),
        )
    };
    let mut fields =
        std::collections::BTreeMap::from([(locator("title"), AspectValue::String(title.into()))]);
    if initial {
        fields.insert(locator("status"), AspectValue::String("retained".into()));
    }
    AspectFieldPatch::from(fields)
}

impl Source {
    pub fn seed(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: u32,
        slots: usize,
    ) -> Self {
        let runtime = RelationalRuntimeApi::builder()
            .schema_registry(schema())
            .build();
        let mut entities = Vec::new();
        for partition in 1..=partitions {
            let (_, basis) = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap();
            let mut tx = runtime
                .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
                .unwrap();
            tx.push_batch(
                WorkerIntentBatch::new("seed").push(MutationIntent::Create(
                    CreateIntent::BulkEntities(BulkEntityCreateIntent {
                        partition_id: PartitionId(partition),
                        kind_id: KindId(3),
                        client_keys: (0..slots)
                            .map(|slot| ClientKey::raw(format!("source-{partition}-{slot}")))
                            .collect(),
                        field_patches: (0..slots).map(|_| facts("original", true)).collect(),
                    }),
                )),
                execution.allocation_policy(),
            )
            .unwrap();
            let seeded = tx.commit(execution, &runtime).unwrap();
            entities.extend(
                seeded
                    .changed_records
                    .iter()
                    .filter_map(|record| match record {
                        RecordRef::Entity(id) => Some(*id),
                        RecordRef::Relation(_) => None,
                    }),
            );
            runtime
                .snapshots()
                .release_snapshot(&seeded.snapshot)
                .unwrap();
        }
        let unrelated = seed_record(&runtime, execution, PartitionId(1), "unrelated");
        let isolated = seed_record(&runtime, execution, PartitionId(77), "isolated");
        let (original, relation) = support::create_relation(
            execution,
            &runtime,
            entities[0],
            unrelated,
            "original-relation",
        );
        Self {
            runtime,
            entities,
            unrelated,
            isolated,
            relation,
            original,
        }
    }

    pub fn transaction(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        dependencies: Dependencies,
    ) -> BranchBoundRelationalTransaction {
        let (_, basis) = self
            .runtime
            .observe_branch(&self.runtime.main_branch_identity())
            .unwrap();
        let mut tx = self
            .runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        if matches!(dependencies, Dependencies::Reads) {
            for entity in &self.entities {
                assert!(tx.read_entity(*entity).unwrap().base().is_some());
            }
        }
        let mut batch = WorkerIntentBatch::new("publish").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: self.entities[0],
                fields: facts("published", false),
            }),
        ));
        let targets = match dependencies {
            Dependencies::Reads => 1,
            Dependencies::Endpoints => self.entities.len() - 1,
        };
        for (index, target) in self.entities.iter().skip(1).take(targets).enumerate() {
            batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: ClientKey::raw(format!("published-{index}")),
                    source: EntityReference::Existing(self.entities[0]),
                    target: EntityReference::Existing(*target),
                    fields: AspectFieldPatch::from_locator(
                        support::aspect_field_locator("label"),
                        AspectValue::String("published".into()),
                    ),
                },
            )));
        }
        tx.push_batch(batch, execution.allocation_policy()).unwrap();
        tx
    }
}

fn seed_record(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    partition_id: PartitionId,
    title: &str,
) -> EntityId {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut tx = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new(title).push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id,
                kind_id: KindId(3),
                client_key: ClientKey::raw(title),
                fields: facts(title, true),
            },
        ))),
        execution.allocation_policy(),
    )
    .unwrap();
    let committed = tx.commit(execution, runtime).unwrap();
    let entity = support::changed_entity(&committed).unwrap();
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    entity
}
