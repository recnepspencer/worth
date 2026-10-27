use worth_foundational::facade::*;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    config::{CascadeDeletePolicy, CrossContextPolicy},
    history::{RelationalCanonicalRecordHistoryCursor, RelationalRecordHistorySubject},
    identity::{EntityId, KindId, PartitionId, RelationId},
    mvcc::RelationalTransactionIntent,
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    schema::*,
    symbols::ClientKey,
    transactions::*,
};

pub struct NativeHistoryCourt {
    pub runtime: RelationalRuntime,
    pub entities: Vec<EntityId>,
    pub relation: RelationId,
    pub contracts: [AspectContract; 3],
}

impl NativeHistoryCourt {
    pub fn install(execution: worth_execution::ExecutionRequest<'_, '_>) -> Self {
        // Aspect lexical order intentionally disagrees with patch grammar rank.
        let contracts = contracts();
        let registry = RelationalSchemaRegistry::new()
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(1),
                kind_name: "audit-record".into(),
                schema_id: SchemaId("native-history".into()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::new(
                    contracts
                        .iter()
                        .map(|contract| DeclaredAspectContractBinding {
                            binding: AspectBinding::EntityField {
                                field: FieldKey::new(contract.key().as_str()).unwrap(),
                            },
                            contract: contract.clone(),
                        })
                        .collect(),
                ),
            })
            .unwrap()
            .register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "audit-edge".into(),
                schema_id: SchemaId("native-history".into()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::new(vec![]),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
            .unwrap();
        let runtime = RelationalRuntimeApi::builder()
            .schema_registry(registry)
            .build();
        let basis = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1;
        let mut transaction = runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        let mut batch = WorkerIntentBatch::new("seed-history-records");
        for index in 0..5 {
            batch = batch.push(MutationIntent::Create(CreateIntent::EntityAspects(
                EntityAspectCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw(format!("row-{index}")),
                    aspect_patch: initial_patch(&contracts),
                },
            )));
        }
        transaction
            .push_batch(batch, execution.allocation_policy())
            .unwrap();
        let created = transaction.commit(&execution, &runtime).unwrap();
        let entities: Vec<_> = created
            .changed_records
            .iter()
            .filter_map(|record| match record {
                RecordRef::Entity(id) => Some(*id),
                _ => None,
            })
            .collect();
        let basis = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1;
        let mut transaction = runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        transaction
            .push_batch(
                WorkerIntentBatch::new("seed-history-edge").push(MutationIntent::Create(
                    CreateIntent::Relation(RelationSpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(2),
                        client_key: ClientKey::raw("edge"),
                        source: EntityReference::Existing(entities[0]),
                        target: EntityReference::Existing(entities[1]),
                        fields: AspectFieldPatch::default(),
                    }),
                )),
                execution.allocation_policy(),
            )
            .unwrap();
        let created = transaction.commit(&execution, &runtime).unwrap();
        let relation = created
            .changed_records
            .iter()
            .find_map(|record| match record {
                RecordRef::Relation(id) => Some(*id),
                _ => None,
            })
            .unwrap();
        Self {
            runtime,
            entities,
            relation,
            contracts,
        }
    }

    pub fn basis(&self) -> AdmittedRelationalBranchBasis {
        self.runtime
            .observe_branch(&self.runtime.main_branch_identity())
            .unwrap()
            .1
    }

    pub fn update_all(&self, execution: worth_execution::ExecutionRequest<'_, '_>) {
        let mut transaction = self
            .runtime
            .begin_branch_transaction(&self.basis(), RelationalTransactionIntent::ordinary())
            .unwrap();
        let mut batch = WorkerIntentBatch::new("three-grammar-ranks");
        for entity_id in &self.entities {
            batch = batch.push(MutationIntent::Entity(
                EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                    entity_id: *entity_id,
                    aspect_patch: PortableRecordAspectPatch::new([
                        PortableAspectPatchOperation::SetWhole {
                            basis: PortableAspectContractBasis::from_contract(&self.contracts[0]),
                            value: ContractValidationInput::Scalar(AspectValue::String(
                                "changed".into(),
                            )),
                        },
                        PortableAspectPatchOperation::ClearWhole {
                            basis: PortableAspectContractBasis::from_contract(&self.contracts[1]),
                        },
                        PortableAspectPatchOperation::PatchFields {
                            basis: PortableAspectContractBasis::from_contract(&self.contracts[2]),
                            selected_fields: vec![FieldKey::new("title").unwrap()],
                            field_sets: vec![PortableAspectFieldSet::new(
                                FieldKey::new("title").unwrap(),
                                AspectValue::String("after".into()),
                            )],
                            field_clears: vec![],
                        },
                    ]),
                }),
            ));
        }
        transaction
            .push_batch(batch, execution.allocation_policy())
            .unwrap();
        transaction.commit(&execution, &self.runtime).unwrap();
    }

    pub fn equal_scalar_edit(&self, execution: worth_execution::ExecutionRequest<'_, '_>) {
        let mut transaction = self
            .runtime
            .begin_branch_transaction(&self.basis(), RelationalTransactionIntent::ordinary())
            .unwrap();
        transaction
            .push_batch(
                WorkerIntentBatch::new("retained-equal-value-write").push(MutationIntent::Entity(
                    EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                        entity_id: self.entities[0],
                        aspect_patch: PortableRecordAspectPatch::new([
                            PortableAspectPatchOperation::SetWhole {
                                basis: PortableAspectContractBasis::from_contract(
                                    &self.contracts[0],
                                ),
                                value: ContractValidationInput::Scalar(AspectValue::String(
                                    "changed".into(),
                                )),
                            },
                        ]),
                    }),
                )),
                execution.allocation_policy(),
            )
            .unwrap();
        transaction.commit(&execution, &self.runtime).unwrap();
    }

    pub fn cursor(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        subjects: &[RelationalRecordHistorySubject],
        aspects: &[AspectKey],
    ) -> RelationalCanonicalRecordHistoryCursor {
        let observation = basis.observation();
        let selected = self
            .runtime
            .select_exact_commit(&observation, observation.commit_id().unwrap())
            .into_outcome()
            .unwrap();
        self.runtime
            .record_aspect_history_cursor(execution, selected, subjects, aspects)
            .unwrap()
    }
}

fn contracts() -> [AspectContract; 3] {
    let vocabulary = aspects().vocabulary();
    [
        aspects()
            .contract()
            .for_key(AspectKey::new("z-set").unwrap())
            .identified_by(AspectIdentity(801))
            .at_revision(vocabulary.revision(1))
            .scalar(ScalarAspectType::String),
        aspects()
            .contract()
            .for_key(AspectKey::new("a-clear").unwrap())
            .identified_by(AspectIdentity(802))
            .at_revision(vocabulary.revision(1))
            .scalar_with(
                ScalarAspectType::String,
                AspectMaskContract::scalar(),
                AbsenceLaw::Optional,
                AspectEquivalenceBasis::ExactCanonicalValue,
                AspectEvolutionPolicy::ExplicitBreakRequired,
            ),
        aspects()
            .contract()
            .for_key(AspectKey::new("m-fields").unwrap())
            .identified_by(AspectIdentity(803))
            .at_revision(vocabulary.revision(1))
            .struct_aspect(
                aspects()
                    .struct_fields()
                    .required("title", ScalarAspectType::String)
                    .finish()
                    .unwrap(),
            ),
    ]
}

fn initial_patch(contracts: &[AspectContract; 3]) -> PortableRecordAspectPatch {
    PortableRecordAspectPatch::new([
        PortableAspectPatchOperation::SetWhole {
            basis: PortableAspectContractBasis::from_contract(&contracts[0]),
            value: ContractValidationInput::Scalar(AspectValue::String("same".into())),
        },
        PortableAspectPatchOperation::SetWhole {
            basis: PortableAspectContractBasis::from_contract(&contracts[1]),
            value: ContractValidationInput::Scalar(AspectValue::String("clear me".into())),
        },
        PortableAspectPatchOperation::SetWhole {
            basis: PortableAspectContractBasis::from_contract(&contracts[2]),
            value: ContractValidationInput::Struct(
                StructAspectValue::new([(
                    FieldKey::new("title").unwrap(),
                    AspectValue::String("before".into()),
                )])
                .unwrap(),
            ),
        },
    ])
}
