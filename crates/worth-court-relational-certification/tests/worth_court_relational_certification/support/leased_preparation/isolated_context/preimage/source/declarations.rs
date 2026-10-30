// Genuine aspect-backed records and ordinary transaction intents for preimage admission.
pub(crate) use {
    crate::leased_preparation::ordinary_context::support, worth_foundational::facade::AspectValue,
    worth_relational::facade::identity::KindId, worth_relational::facade::identity::PartitionId,
    worth_relational::facade::mvcc::RelationalTransactionIntent,
    worth_relational::facade::runtime::RelationalRuntimeApi,
    worth_relational::facade::symbols::ClientKey,
    worth_relational::facade::transactions::AspectFieldPatch,
    worth_relational::facade::transactions::BulkEntityCreateIntent,
    worth_relational::facade::transactions::CreateIntent,
    worth_relational::facade::transactions::EntitySpec,
    worth_relational::facade::transactions::MutationIntent,
    worth_relational::facade::transactions::WorkerIntentBatch,
};
pub(crate) use {
    worth_foundational::facade::aspects, worth_foundational::facade::AspectFieldLocator,
    worth_foundational::facade::AspectIdentity, worth_foundational::facade::AspectKey,
    worth_foundational::facade::CanonicalFieldPath, worth_foundational::facade::FieldKey,
    worth_foundational::facade::LocatorAuthority, worth_foundational::facade::ScalarAspectType,
};
pub(crate) use {
    worth_relational::facade::identity::EntityId, worth_relational::facade::identity::RelationId,
    worth_relational::facade::mvcc::BranchBoundRelationalTransaction,
    worth_relational::facade::runtime::RelationalRuntime,
    worth_relational::facade::schema::AspectBinding,
    worth_relational::facade::schema::DeclaredAspectContractBinding,
    worth_relational::facade::schema::EntityKindRegistration,
    worth_relational::facade::schema::KindAspectContractDeclarations,
    worth_relational::facade::schema::RelationalSchemaRegistry,
    worth_relational::facade::schema::SchemaId, worth_relational::facade::schema::SchemaVersionId,
    worth_relational::facade::transactions::CommitResult,
    worth_relational::facade::transactions::EntityMutationIntent,
    worth_relational::facade::transactions::EntityReference,
    worth_relational::facade::transactions::RecordRef,
    worth_relational::facade::transactions::RelationSpec,
    worth_relational::facade::transactions::UpdateEntityFieldsIntent,
};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Dependencies {
    Reads,
    Endpoints,
}

pub(crate) struct Source {
    pub(crate) runtime: RelationalRuntime,
    pub(crate) entities: Vec<EntityId>,
    pub(crate) unrelated: EntityId,
    pub(crate) isolated: EntityId,
    pub(crate) relation: RelationId,
    pub(crate) original: CommitResult,
}

pub(crate) fn schema() -> RelationalSchemaRegistry {
    let shape = aspects()
        .struct_fields()
        .required(
            "title",
            worth_foundational::facade::ScalarAspectType::String,
        )
        .required(
            "status",
            worth_foundational::facade::ScalarAspectType::String,
        )
        .finish()
        .unwrap();
    crate::leased_preparation::ordinary_context::support::demo_schema_registry()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(3),
            kind_name: "source.record".into(),
            schema_id: SchemaId("preimage".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations:
                worth_relational::facade::schema::KindAspectContractDeclarations::new(vec![
                    DeclaredAspectContractBinding {
                        binding: worth_relational::facade::schema::AspectBinding::EntityField {
                            field: worth_foundational::facade::FieldKey::new("facts").unwrap(),
                        },
                        contract: aspects()
                            .contract()
                            .for_key(worth_foundational::facade::AspectKey::new("facts").unwrap())
                            .identified_by(AspectIdentity(0x434f_0017))
                            .at_revision(aspects().vocabulary().revision(1))
                            .struct_aspect(shape),
                    },
                ]),
        })
        .unwrap()
}

pub(crate) fn facts(title: &str, initial: bool) -> AspectFieldPatch {
    let locator = |field| {
        worth_foundational::facade::AspectFieldLocator::new(
            worth_foundational::facade::LocatorAuthority::Planned,
            worth_foundational::facade::AspectKey::new("facts").unwrap(),
            worth_foundational::facade::CanonicalFieldPath::single(
                worth_foundational::facade::FieldKey::new(field).unwrap(),
            ),
        )
    };
    let mut fields = std::collections::BTreeMap::from([(
        locator("title"),
        worth_foundational::facade::AspectValue::String(title.into()),
    )]);
    if initial {
        fields.insert(
            locator("status"),
            worth_foundational::facade::AspectValue::String("retained".into()),
        );
    }
    worth_relational::facade::transactions::AspectFieldPatch::from(fields)
}

impl Source {
    pub(crate) fn seed(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: u32,
        slots: usize,
    ) -> Self {
        let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
            .schema_registry(schema())
            .build()
            .expect("valid native runtime configuration");
        let mut entities = Vec::new();
        for partition in 1..=partitions {
            let (_, basis) = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap();
            let mut tx = runtime
                .begin_branch_transaction(
                    &basis,
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .unwrap();
            tx.push_batch(
                worth_relational::facade::transactions::WorkerIntentBatch::new("seed").push(
                    worth_relational::facade::transactions::MutationIntent::Create(
                        worth_relational::facade::transactions::CreateIntent::BulkEntities(
                            BulkEntityCreateIntent {
                                partition_id: PartitionId(partition),
                                kind_id: KindId(3),
                                client_keys: (0..slots)
                                    .map(|slot| {
                                        worth_relational::facade::symbols::ClientKey::raw(format!(
                                            "source-{partition}-{slot}"
                                        ))
                                    })
                                    .collect(),
                                field_patches: (0..slots)
                                    .map(|_| facts("original", true))
                                    .collect(),
                            },
                        ),
                    ),
                ),
                execution.allocation_policy(),
            )
            .unwrap();
            let seeded = tx.commit(execution, &runtime).unwrap();
            entities.extend(
                seeded
                    .changed_records
                    .iter()
                    .filter_map(|record| match record {
                        worth_relational::facade::transactions::RecordRef::Entity(id) => Some(*id),
                        worth_relational::facade::transactions::RecordRef::Relation(_) => None,
                    }),
            );
            runtime
                .snapshots()
                .release_snapshot(&seeded.snapshot)
                .unwrap();
        }
        let unrelated = seed_record(&runtime, execution, PartitionId(1), "unrelated");
        let isolated = seed_record(&runtime, execution, PartitionId(77), "isolated");
        let (original, relation) =
            crate::leased_preparation::ordinary_context::support::create_relation(
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

    pub(crate) fn transaction(
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
            .begin_branch_transaction(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .unwrap();
        if matches!(
            dependencies,
            crate::leased_preparation::isolated_context::preimage::source::Dependencies::Reads
        ) {
            for entity in &self.entities {
                assert!(tx.read_entity(execution, *entity).unwrap().base().is_some());
            }
        }
        let mut batch = worth_relational::facade::transactions::WorkerIntentBatch::new("publish")
            .push(
                worth_relational::facade::transactions::MutationIntent::Entity(
                    worth_relational::facade::transactions::EntityMutationIntent::UpdateFields(
                        UpdateEntityFieldsIntent {
                            entity_id: self.entities[0],
                            fields: facts("published", false),
                        },
                    ),
                ),
            );
        let targets = match dependencies {
            crate::leased_preparation::isolated_context::preimage::source::Dependencies::Reads => 1,
            crate::leased_preparation::isolated_context::preimage::source::Dependencies::Endpoints => self.entities.len() - 1,
        };
        for (index, target) in self.entities.iter().skip(1).take(targets).enumerate() {
            batch = batch.push(worth_relational::facade::transactions::MutationIntent::Create(worth_relational::facade::transactions::CreateIntent::Relation(
                RelationSpec {
                    partition_id: worth_relational::facade::identity::PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: worth_relational::facade::symbols::ClientKey::raw(format!("published-{index}")),
                    source: worth_relational::facade::transactions::EntityReference::Existing(self.entities[0]),
                    target: worth_relational::facade::transactions::EntityReference::Existing(*target),
                    fields: worth_relational::facade::transactions::AspectFieldPatch::from_locator(
                        crate::leased_preparation::ordinary_context::support::aspect_field_locator("label"),
                        worth_foundational::facade::AspectValue::String("published".into()),
                    ),
                },
            )));
        }
        tx.push_batch(batch, execution.allocation_policy()).unwrap();
        tx
    }
}

pub(crate) fn seed_record(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    partition_id: PartitionId,
    title: &str,
) -> EntityId {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut tx = runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    tx.push_batch(
        worth_relational::facade::transactions::WorkerIntentBatch::new(title).push(
            worth_relational::facade::transactions::MutationIntent::Create(
                worth_relational::facade::transactions::CreateIntent::Entity(EntitySpec {
                    partition_id,
                    kind_id: KindId(3),
                    client_key: worth_relational::facade::symbols::ClientKey::raw(title),
                    fields: facts(title, true),
                }),
            ),
        ),
        execution.allocation_policy(),
    )
    .unwrap();
    let committed = tx.commit(execution, runtime).unwrap();
    let entity =
        crate::leased_preparation::ordinary_context::support::changed_entity(&committed).unwrap();
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    entity
}
