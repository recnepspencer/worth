//! The oracle's reference: a memory-only Relational runtime executes a seeded
//! sequence of ordinary transactions, and its retained canonical commits,
//! exported as Relational canonical commit artifacts, are the journal the
//! durable Store must hold.

pub use {std::collections::BTreeMap};

pub use {worth_execution::CancellationSource, worth_execution::ExecutionAllocationPolicy, worth_execution::ExecutionRequest, worth_execution::SerialMemoryBudget, worth_execution::SerialRequest};
pub use {worth_foundational::facade::aspects, worth_foundational::facade::AspectFieldLocator, worth_foundational::facade::AspectIdentity, worth_foundational::facade::AspectKey, worth_foundational::facade::AspectValue, worth_foundational::facade::CanonicalFieldPath, worth_foundational::facade::FieldKey, worth_foundational::facade::LocatorAuthority, worth_foundational::facade::ScalarAspectType};
pub use {worth_relational::facade::config::CascadeDeletePolicy, worth_relational::facade::config::CrossContextPolicy, worth_relational::facade::durability::RelationalCanonicalCommitArtifact, worth_relational::facade::identity::EntityId, worth_relational::facade::identity::KindId, worth_relational::facade::identity::PartitionId, worth_relational::facade::mvcc::RelationalTransactionIntent, worth_relational::facade::runtime::RelationalRuntime, worth_relational::facade::runtime::RelationalRuntimeApi, worth_relational::facade::schema::AspectBinding, worth_relational::facade::schema::DeclaredAspectContractBinding, worth_relational::facade::schema::EntityKindRegistration, worth_relational::facade::schema::KindAspectContractDeclarations, worth_relational::facade::schema::RelationIntegrityDeclarations, worth_relational::facade::schema::RelationKindRegistration, worth_relational::facade::schema::RelationalSchemaRegistry, worth_relational::facade::schema::SchemaId, worth_relational::facade::schema::SchemaVersionId, worth_relational::facade::symbols::ClientKey, worth_relational::facade::transactions::AspectFieldPatch, worth_relational::facade::transactions::CreateIntent, worth_relational::facade::transactions::DeleteEntityIntent, worth_relational::facade::transactions::EntityMutationIntent, worth_relational::facade::transactions::EntityReference, worth_relational::facade::transactions::EntitySpec, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::RecordRef, worth_relational::facade::transactions::RelationSpec, worth_relational::facade::transactions::UpdateEntityFieldsIntent, worth_relational::facade::transactions::WorkerIntentBatch};

/// The reference run: the runtime that produced the commits stays alive so
/// readmission is checked against real Relational authority.
pub struct RelationalReference {
    pub artifacts: Vec<Vec<u8>>,
    pub request: SerialRequest,
    pub _runtime: RelationalRuntime,
}

impl RelationalReference {
    /// Executes `commits` seeded transactions and exports their artifacts.
    pub fn generate(seed: u64, commits: usize) -> Self {
        let request = serial_request();
        let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
            .runtime_setup(|runtime| {
                runtime.runtime_name("journal-oracle");
            })
            .schema_setup(|schema| {
                schema.schema_registry(schema_registry());
            })
            .build()
            .expect("valid native runtime configuration");
        {
            let execution = worth_execution::ExecutionRequest::serial(&request);
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
                            worth_relational::facade::transactions::RecordRef::Entity(entity) if choice <= 2 => Some(*entity),
                            _ => None,
                        }),
                );
            }
        }
        let artifacts = runtime
            .durability()
            .canonical_commit_artifacts(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
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
    pub fn assert_readmits(&self, position: u64, bytes: &[u8]) {
        let execution = worth_execution::ExecutionRequest::serial(&self.request);
        let readmitted = worth_relational::facade::durability::RelationalCanonicalCommitArtifact::readmit(bytes, &execution)
            .unwrap_or_else(|refusal| panic!("journal entry {position} refused: {refusal:?}"));
        assert_eq!(
            readmitted.position().0,
            position,
            "journal position {position}"
        );
    }
}

pub fn commit(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    batch: WorkerIntentBatch,
) -> worth_relational::facade::transactions::CommitResult {
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("the main branch is owner-observable");
    let mut transaction = runtime
        .begin_branch_transaction(&basis, worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary())
        .expect("owner-admitted transaction");
    transaction
        .push_batch(batch, worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .expect("the batch stages");
    transaction
        .commit(execution, runtime)
        .expect("the reference commit")
}

pub fn create(step: usize) -> WorkerIntentBatch {
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

pub fn update(entity_id: EntityId, step: usize) -> WorkerIntentBatch {
    worth_relational::facade::transactions::WorkerIntentBatch::new(format!("update-{step}")).push(worth_relational::facade::transactions::MutationIntent::Entity(
        worth_relational::facade::transactions::EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id,
            fields: field_patch("name", &format!("renamed-{step}")),
        }),
    ))
}

pub fn relate(source: EntityId, target: EntityId, step: usize) -> WorkerIntentBatch {
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

pub fn delete(entity_id: EntityId) -> WorkerIntentBatch {
    worth_relational::facade::transactions::WorkerIntentBatch::new("delete").push(worth_relational::facade::transactions::MutationIntent::Entity(worth_relational::facade::transactions::EntityMutationIntent::Delete(
        DeleteEntityIntent { entity_id },
    )))
}

pub fn schema_registry() -> RelationalSchemaRegistry {
    worth_relational::facade::schema::RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "oracle.entity".to_string(),
            schema_id: SchemaId("oracle".to_string()),
            schema_version_id: SchemaVersionId(1),
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
                schema_version_id: SchemaVersionId(1),
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

pub fn field_patch(label: &str, value: &str) -> AspectFieldPatch {
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

pub fn field_key(label: &str) -> FieldKey {
    worth_foundational::facade::FieldKey::new(label).expect("oracle field key")
}

pub fn aspect_key(label: &str) -> AspectKey {
    worth_foundational::facade::AspectKey::new(label).expect("oracle aspect key")
}

pub fn string_contract(label: &str) -> worth_foundational::AspectContract {
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

pub fn serial_request() -> SerialRequest {
    worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        worth_execution::CancellationSource::new().token(),
        None,
    )
}

/// A small deterministic generator, so a failing seed reproduces exactly.
pub struct Xorshift(pub u64);

impl Xorshift {
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    pub fn index(&mut self, length: usize) -> usize {
        (self.next() % length as u64) as usize
    }
}