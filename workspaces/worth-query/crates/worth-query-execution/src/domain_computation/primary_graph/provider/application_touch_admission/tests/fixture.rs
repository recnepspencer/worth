//! Real installation contracts and native candidates for the admission owner.
use super::*;
use worth_query_decl::facade::{
    application_schema::*, worth_query_application_schema, worth_query_aspect, worth_query_entity,
    worth_query_field, worth_query_operation, worth_query_operation_reads,
    worth_query_operation_writes,
};
use worth_query_installation::facade::*;

worth_query_entity!(pub(super) Node for TouchSchema);
worth_query_entity!(pub(super) RequiredNode for TouchSchema);
worth_query_aspect!(pub(super) Identity for TouchSchema, Node; identity = AspectIdentity(0x91000201), revision = AspectContractRevision(1));
worth_query_aspect!(pub(super) Notes for TouchSchema, Node; identity = AspectIdentity(0x91000202), revision = AspectContractRevision(1));
worth_query_aspect!(pub(super) RequiredNotes for TouchSchema, RequiredNode; identity = AspectIdentity(0x91000203), revision = AspectContractRevision(1));
worth_query_field!(pub(super) Key for TouchSchema, Node, Identity: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub(super) A for TouchSchema, Node, Notes: optional String => StringApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub(super) B for TouchSchema, Node, Notes: optional String => StringApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub(super) RequiredA for TouchSchema, RequiredNode, RequiredNotes: optional String => StringApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub(super) RequiredB for TouchSchema, RequiredNode, RequiredNotes: optional String => StringApplicationValueBinding, read_write, no_equality);
#[derive(Clone, serde::Serialize)]
pub(super) struct Input {
    value: String,
}
worth_query_decl::facade::worth_query_structured_value_binding!(pub(super) InputBinding for Input { identity: "worth.query.test.struct-touch-input.v1" });
worth_query_operation!(pub(super) OnlyA for TouchSchema, input InputBinding);
worth_query_operation_reads!(OnlyA => [Node]);
worth_query_operation_writes!(OnlyA => [A]);
worth_query_operation!(pub(super) Both for TouchSchema, input InputBinding);
worth_query_operation_reads!(Both => [Node, RequiredNode]);
worth_query_operation_writes!(Both => [A, B, RequiredA, RequiredB]);
worth_query_application_schema! {
    pub schema TouchSchema {
        owner: struct_touch_admission,
        version: (1, 0),
        members: |schema| {
            let one = OnlyA::reference();
            let both = Both::reference();
            schema.entity(Node::reference()).entity(RequiredNode::reference())
                .aspect(Node::reference(), Identity::reference())
                .field(Node::reference(), Key::reference())
                .optional_aspect(Node::reference(), Notes::reference())
                .field(Node::reference(), A::reference()).field(Node::reference(), B::reference())
                .aspect(RequiredNode::reference(), RequiredNotes::reference())
                .field(RequiredNode::reference(), RequiredA::reference())
                .field(RequiredNode::reference(), RequiredB::reference())
                .operation(one.definition().no_external_effect().no_aftermath().finish())
                .operation_projection_work_budget(one, 16)
                .operation_read_entity(one, Node::reference()).operation_write(one, A::reference())
                .operation(both.definition().no_external_effect().no_aftermath().finish())
                .operation_projection_work_budget(both, 16)
                .operation_read_entity(both, Node::reference())
                .operation_read_entity(both, RequiredNode::reference())
                .operation_write(both, A::reference()).operation_write(both, B::reference())
                .operation_write(both, RequiredA::reference()).operation_write(both, RequiredB::reference())
        }
    }
}

pub(super) struct World {
    pub native: RelationalRuntime,
    pub layout: WorthQueryPrimaryGraphLayout,
    pub schema: WorthQueryInstalledApplicationSchema<TouchSchema>,
    pub entity: worth_relational::facade::identity::EntityId,
    pub retained_commit: CommitResult,
    pub entity_name: &'static str,
    pub aspect: &'static str,
}
impl World {
    pub fn new(
        execution: &ExecutionRequest<'_, '_>,
        required: bool,
        fields: &[(&str, &str)],
    ) -> Self {
        let declaration = TouchSchema::declaration().unwrap();
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            TouchSchema::OWNER,
            TouchSchema::MAJOR,
            TouchSchema::MINOR,
        ))
        .application_schema(declaration.clone())
        .validate()
        .unwrap();
        let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
            .admit(package)
            .unwrap();
        let (runtime, _) =
            crate::domain_computation::execution_runtime::WorthQueryExecutionRuntimeInstaller::new(
            )
            .install(WorthQueryInstallationGeneration::initial(), [admitted])
            .unwrap()
            .into_parts();
        let schema = runtime
            .installed_packages()
            .bind_application_schema(declaration)
            .unwrap();
        let (layout, registry) = WorthQueryPrimaryGraphLayout::lower(
            schema.installed_declaration(),
            schema.native_contracts(),
            &worth_relational::facade::schema::RelationalSchemaRegistry::new(),
        )
        .unwrap();
        let native = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
            .schema_registry(registry)
            .build();
        let entity_name = if required { "RequiredNode" } else { "Node" };
        let aspect = if required { "RequiredNotes" } else { "Notes" };
        let mut values = std::collections::BTreeMap::new();
        if !required {
            values.insert(
                layout
                    .field_locator("Node", "Identity", "Key")
                    .unwrap()
                    .clone(),
                encode("sentinel"),
            );
        }
        for (field, value) in fields {
            values.insert(
                layout
                    .field_locator(entity_name, aspect, field)
                    .unwrap()
                    .clone(),
                encode(value),
            );
        }
        let (_, basis) = native
            .observe_branch(&native.main_branch_identity())
            .unwrap();
        let mut tx = native
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        tx.push_batch(
            WorkerIntentBatch::new("authentic-touch-baseline").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: worth_relational::facade::identity::PartitionId::main(),
                    kind_id: layout.entity_kind(entity_name).unwrap(),
                    client_key: worth_relational::facade::symbols::ClientKey::raw(
                        "native-touch-baseline",
                    ),
                    fields: AspectFieldPatch::from(values),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
        let birth = tx.commit(execution, &native).unwrap();
        let RecordRef::Entity(entity) = birth.changed_records[0] else {
            panic!("real entity birth");
        };
        Self {
            native,
            layout,
            schema,
            entity,
            retained_commit: birth,
            entity_name,
            aspect,
        }
    }
    pub fn contract(&self) -> &worth_foundational::facade::AspectContract {
        self.layout
            .aspect_contract(
                self.entity_name,
                &worth_foundational::facade::AspectKey::new(self.aspect).unwrap(),
            )
            .unwrap()
    }
    pub fn set(&self, fields: &[(&str, &str)]) -> Op {
        Op::SetWhole {
            basis: worth_foundational::facade::PortableAspectContractBasis::from_contract(
                self.contract(),
            ),
            value: worth_foundational::facade::ContractValidationInput::Struct(
                worth_foundational::facade::StructAspectValue::new(fields.iter().map(
                    |(field, value)| {
                        (
                            worth_foundational::facade::FieldKey::new(*field).unwrap(),
                            encode(value),
                        )
                    },
                ))
                .unwrap(),
            ),
        }
    }
    pub fn clear(&self) -> Op {
        Op::ClearWhole {
            basis: worth_foundational::facade::PortableAspectContractBasis::from_contract(
                self.contract(),
            ),
        }
    }
    pub fn candidate(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        operations: impl IntoIterator<Item = Op>,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        self.candidate_envelopes(execution, [operations])
    }
    pub fn candidate_envelopes(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        envelopes: impl IntoIterator<Item = impl IntoIterator<Item = Op>>,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        let (_, basis) = self
            .native
            .observe_branch(&self.native.main_branch_identity())
            .unwrap();
        let mut tx = self
            .native
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        for operations in envelopes {
            tx.push_batch(
                WorkerIntentBatch::new("native-whole-struct-candidate").push(
                    MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(
                        ApplyEntityAspectPatchIntent {
                            entity_id: self.entity,
                            aspect_patch:
                                worth_foundational::facade::PortableRecordAspectPatch::new(
                                    operations,
                                ),
                        },
                    )),
                ),
                execution.allocation_policy(),
            )
            .unwrap();
        }
        tx.validate(execution, &self.native)
    }
    pub fn commit_admitted(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        proposal: ValidatedRelationalProposal,
    ) {
        self.admit(execution, &proposal, true)
            .expect("full installed grants before effects");
        let candidate = self
            .native
            .prepare_validated_proposal(execution, proposal)
            .unwrap();
        let outcome = self
            .native
            .publication_port()
            .compare_and_publish(execution, candidate);
        let worth_relational::facade::mvcc::RelationalPublicationOutcome::Performed(performed) =
            outcome
        else {
            panic!("uncontended authentic first publication: {outcome:?}");
        };
        let committed = self
            .native
            .settlement_port()
            .settle_performed_publication(performed)
            .unwrap();
        crate::relational_snapshot_release::release_query_snapshot(
            &mut self.native,
            &self.retained_commit.snapshot,
        );
        self.retained_commit = committed;
    }
    pub fn admit(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &ValidatedRelationalProposal,
        both: bool,
    ) -> Result<WorthQueryInstalledTouchAdmissionEvidence, TouchAdmissionFailure> {
        let one = self.schema.installed_operation(OnlyA::reference()).unwrap();
        let full = self.schema.installed_operation(Both::reference()).unwrap();
        let contracts = if both {
            full.contracts()
        } else {
            one.contracts()
        };
        admit_validated_application_touches(
            execution,
            candidate,
            &self.layout,
            contracts.graph_reads(),
            contracts.touches(),
            contracts.read_touch_overlap(),
        )
    }
    pub fn truth(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Vec<Option<worth_foundational::facade::AspectValue>> {
        use worth_relational::facade::runtime::{
            ProjectionAspectRequirement, ProjectionAspectScope,
        };
        let aspect = self.contract().key();
        let fields: Vec<_> = if self.entity_name == "Node" {
            ["A", "B"]
        } else {
            ["RequiredA", "RequiredB"]
        }
        .into_iter()
        .map(|f| worth_foundational::facade::FieldKey::new(f).unwrap())
        .collect();
        let view = self
            .native
            .read_truth()
            .project_snapshot(&self.retained_commit.snapshot)
            .unwrap();
        view.entity_record_with_projection_scope(
            execution,
            self.entity,
            ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
                aspect.clone(),
                fields.clone(),
            )]),
            |record| {
                Some(
                    fields
                        .iter()
                        .map(|f| record.aspect_field_value(aspect, f).cloned())
                        .collect(),
                )
            },
        )
        .unwrap()
        .unwrap()
    }
}
impl Drop for World {
    fn drop(&mut self) {
        crate::relational_snapshot_release::release_query_snapshot(
            &mut self.native,
            &self.retained_commit.snapshot,
        );
    }
}
fn encode(value: &str) -> worth_foundational::facade::AspectValue {
    StringApplicationValueBinding::encode(&value.to_owned()).unwrap()
}
