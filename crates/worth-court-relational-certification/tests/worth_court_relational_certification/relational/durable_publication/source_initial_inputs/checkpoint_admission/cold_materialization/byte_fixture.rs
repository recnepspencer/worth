//! Real scalar/Struct byte originals, selected image and schema-matched receiver.
use super::*;
use worth_foundational::facade::*;
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::schema::*;
use worth_testkit_relational::aspect_storage::{definition::*, AspectSchemaFixture};

pub(crate) struct ByteHome {
    pub medium: Arc<TestDurableMedium>,
    pub image: CompleteRelationalNativeCheckpoint,
    pub entity: EntityId,
    pub old_version: worth_relational::facade::identity::VersionId,
    pub current_version: worth_relational::facade::identity::VersionId,
    declarations: RelationalSchemaRegistry,
}
impl ByteHome {
    pub fn new(request: &ExecutionRequest<'_, '_>, count: usize) -> Self {
        let scalar = aspects()
            .contract()
            .for_key(aspect_key("blob"))
            .identified_by(AspectIdentity(712))
            .at_revision(aspects().vocabulary().revision(1))
            .scalar(ScalarAspectType::InlineBytes);
        let structure = aspects()
            .contract()
            .for_key(aspect_key("packet"))
            .identified_by(AspectIdentity(713))
            .at_revision(aspects().vocabulary().revision(1))
            .struct_aspect(
                aspects()
                    .struct_fields()
                    .required("data", ScalarAspectType::InlineBytes)
                    .finish()
                    .unwrap(),
            );
        let fixture = AspectSchemaFixture::default();
        let declarations = RelationalSchemaRegistry::new()
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(902),
                kind_name: "cold.bytes".into(),
                schema_id: fixture.schema_id,
                schema_version_id: fixture.schema_version_id,
                aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                    DeclaredAspectContractBinding {
                        binding: AspectBinding::EntityField {
                            field: field_key("blob"),
                        },
                        contract: scalar.clone(),
                    },
                    DeclaredAspectContractBinding {
                        binding: AspectBinding::EntityField {
                            field: field_key("packet"),
                        },
                        contract: structure.clone(),
                    },
                ]),
            })
            .unwrap();
        let (runtime, medium) =
            super::super::contract_work::initialized_contracts(request, declarations.clone());
        let mut transaction = begin_main_transaction(&runtime);
        transaction
            .push_batch(
                WorkerIntentBatch::new("old-bytes").push(MutationIntent::Create(
                    CreateIntent::EntityAspects(EntityAspectCreateIntent {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(902),
                        client_key: "bytes".into(),
                        aspect_patch: patch(&scalar, &structure, count, 0x17),
                    }),
                )),
                request.allocation_policy(),
            )
            .unwrap();
        let created = transaction.commit(request, &runtime).unwrap();
        let entity = created.semantic_result().created_entities()[0].assigned;
        let old_version = created.version_id;
        release_test_commit_snapshot(&runtime, &created);
        drop(created);
        let mut transaction = begin_main_transaction(&runtime);
        transaction
            .push_batch(
                WorkerIntentBatch::new("current-bytes").push(MutationIntent::Entity(
                    EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                        entity_id: entity,
                        aspect_patch: patch(&scalar, &structure, count, 0x51),
                    }),
                )),
                request.allocation_policy(),
            )
            .unwrap();
        let updated = transaction.commit(request, &runtime).unwrap();
        let current_version = updated.version_id;
        release_test_commit_snapshot(&runtime, &updated);
        drop(updated);
        let image = capture_external(&runtime, &medium, request);
        medium.file_checkpoint(image.checkpoint().clone());
        drop(runtime);
        Self {
            medium,
            image,
            entity,
            old_version,
            current_version,
            declarations,
        }
    }
    pub fn reopen(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> (RelationalRuntime, Arc<SelectedSource>) {
        let source = SelectedSource::new(self.medium.clone());
        let mut receiver =
            worth_testkit_relational::durable_publication::durable_runtime(&self.medium);
        receiver
            .prepare_initial_schema_installation()
            .unwrap()
            .install(self.declarations.clone())
            .unwrap();
        (
            recover(&self.image, source.clone(), receiver, request),
            source,
        )
    }
}
fn patch(
    scalar: &AspectContract,
    structure: &AspectContract,
    count: usize,
    byte: u8,
) -> PortableRecordAspectPatch {
    let bytes = || AspectValue::InlineBytes(InlineBytes::try_new(vec![byte; count]).unwrap());
    PortableRecordAspectPatch::new([
        PortableAspectPatchOperation::SetWhole {
            basis: PortableAspectContractBasis::from_contract(scalar),
            value: ContractValidationInput::Scalar(bytes()),
        },
        PortableAspectPatchOperation::SetWhole {
            basis: PortableAspectContractBasis::from_contract(structure),
            value: ContractValidationInput::Struct(
                StructAspectValue::new([(field_key("data"), bytes())]).unwrap(),
            ),
        },
    ])
}
pub(crate) fn assert_bytes(
    record: &worth_relational::facade::runtime::EntityReadRecord,
    count: usize,
    byte: u8,
) {
    let state = record.authoritative_aspect_state.as_ref().unwrap();
    let ContractValidatedAspectValueView::Scalar(AspectValue::InlineBytes(scalar)) =
        state.get(&aspect_key("blob")).unwrap().view()
    else {
        panic!("actual scalar bytes")
    };
    assert_eq!(scalar.as_bytes(), vec![byte; count]);
    let ContractValidatedAspectValueView::Struct(value) =
        state.get(&aspect_key("packet")).unwrap().view()
    else {
        panic!("actual Struct bytes")
    };
    let AspectValue::InlineBytes(bytes) = value.get(&field_key("data")).unwrap() else {
        panic!("actual Struct field bytes")
    };
    assert_eq!(bytes.as_bytes(), scalar.as_bytes());
}
