//! The oracle's reference: a memory-only Relational runtime executes a seeded
//! sequence of ordinary transactions, and its retained canonical commits,
//! exported as Relational canonical commit artifacts, are the journal the
//! durable Store must hold.

use std::collections::BTreeMap;

use worth_execution::{
    CancellationSource, ExecutionAllocationPolicy, ExecutionRequest, SerialMemoryBudget,
    SerialRequest,
};
use worth_foundational::facade::{
    aspects, AspectFieldLocator, AspectIdentity, AspectKey, AspectValue, CanonicalFieldPath,
    FieldKey, LocatorAuthority, ScalarAspectType,
};
use worth_relational::facade::{
    config::{CascadeDeletePolicy, CrossContextPolicy},
    durability::RelationalCanonicalCommitArtifact,
    identity::{EntityId, KindId, PartitionId},
    mvcc::RelationalTransactionIntent,
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    schema::{
        AspectBinding, DeclaredAspectContractBinding, EntityKindRegistration,
        KindAspectContractDeclarations, RelationIntegrityDeclarations, RelationKindRegistration,
        RelationalSchemaRegistry, SchemaId, SchemaVersionId,
    },
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, CreateIntent, DeleteEntityIntent, EntityMutationIntent, EntityReference,
        EntitySpec, MutationIntent, RecordRef, RelationSpec, UpdateEntityFieldsIntent,
        WorkerIntentBatch,
    },
};

/// The reference run: the runtime that produced the commits stays alive so
/// readmission is checked against real Relational authority.
pub(super) struct RelationalReference {
    pub(super) artifacts: Vec<Vec<u8>>,
    request: SerialRequest,
    _runtime: RelationalRuntime,
}

impl RelationalReference {
    /// Executes `commits` seeded transactions and exports their artifacts.
    pub(super) fn generate(seed: u64, commits: usize) -> Self {
        let request = serial_request();
        let runtime = RelationalRuntimeApi::builder()
            .runtime_setup(|runtime| {
                runtime.runtime_name("journal-oracle");
            })
            .schema_setup(|schema| {
                schema.schema_registry(schema_registry());
            })
            .build();
        {
            let execution = ExecutionRequest::serial(&request);
            let mut generator = Xorshift(seed | 1);
            let mut live: Vec<EntityId> = Vec::new();
            for step in 0..commits {
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
                let outcome = commit(&runtime, &execution, batch);
                live.extend(
                    outcome
                        .changed_records
                        .iter()
                        .filter_map(|record| match record {
                            RecordRef::Entity(entity) if choice <= 2 => Some(*entity),
                            _ => None,
                        }),
                );
            }
        }
        let artifacts = runtime
            .durability()
            .canonical_commit_artifacts(ExecutionAllocationPolicy::SystemAllocation)
            .map(|artifact| {
                artifact
                    .expect("retained commits encode")
                    .as_bytes()
                    .to_vec()
            })
            .collect::<Vec<_>>();
        assert_eq!(artifacts.len(), commits, "every commit is retained");
        Self {
            artifacts,
            request,
            _runtime: runtime,
        }
    }

    /// Requires `bytes` to readmit as the Relational commit at `position`.
    pub(super) fn assert_readmits(&self, position: u64, bytes: &[u8]) {
        let execution = ExecutionRequest::serial(&self.request);
        let readmitted = RelationalCanonicalCommitArtifact::readmit(bytes, &execution)
            .unwrap_or_else(|refusal| panic!("journal entry {position} refused: {refusal:?}"));
        assert_eq!(
            readmitted.position().0,
            position,
            "journal position {position}"
        );
    }
}

fn commit(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    batch: WorkerIntentBatch,
) -> worth_relational::facade::transactions::CommitResult {
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("the main branch is owner-observable");
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("owner-admitted transaction");
    transaction
        .push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
        .expect("the batch stages");
    transaction
        .commit(execution, runtime)
        .expect("the reference commit")
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

fn schema_registry() -> RelationalSchemaRegistry {
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

fn serial_request() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    )
}

/// A small deterministic generator, so a failing seed reproduces exactly.
struct Xorshift(u64);

impl Xorshift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn index(&mut self, length: usize) -> usize {
        (self.next() % length as u64) as usize
    }
}
