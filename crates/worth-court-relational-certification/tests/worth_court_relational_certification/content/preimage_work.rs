//! Lookup work follows real field/aspect declarations, even with unrelated aspects.
use super::{
    fixture::{grid, Codec},
    original,
};
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_foundational::facade::*;
use worth_relational::facade::{
    content::*, identity::KindId, mvcc::*, runtime::*, schema::*, symbols::ClientKey,
    transactions::*,
};
#[test]
fn content_original_field_capture_preserves_frame_and_cancelled_refusal() {
    let cancel = CancellationSource::new();
    let owner = original(&cancel);
    let execution = ExecutionRequest::serial(&owner);
    let key = AspectKey::new(format!("samples{}", "x".repeat(8))).unwrap();
    let binding = CanonicalDigestId::new([12; 32]);
    let format = CanonicalDigestId::new([11; 32]);
    let field = FieldDeclaration::new(
        FieldKey::new("left").unwrap(),
        ScalarAspectType::ContentRef,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap()
    .with_codec(binding);
    let contract = AspectContract::struct_aspect(
        key.clone(),
        AspectIdentity(741),
        AspectContractRevision(1),
        StructAspectShape::new([field]).unwrap(),
    );
    let aspects = vec![DeclaredAspectContractBinding {
        binding: AspectBinding::EntityField {
            field: FieldKey::new("samples").unwrap(),
        },
        contract: contract.clone(),
    }];
    let mut operations = Vec::new();
    let registry = worth_testkit_relational::aspect_storage::AspectSchemaFixture {
        entity_aspects: aspects,
        ..Default::default()
    }
    .build_registry()
    .register_content_format(
        format,
        Arc::new(Codec {
            descriptor: ContentFormat::new("sample-grid", 1).unwrap(),
            alter: false,
        }),
    )
    .unwrap()
    .bind_content_field(binding, format)
    .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .unwrap();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut txn = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let id = txn.admit_content(&execution, format, &grid()).unwrap();
    operations.push(PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(&contract),
        value: ContractValidationInput::Struct(
            StructAspectValue::new([(FieldKey::new("left").unwrap(), AspectValue::ContentRef(id))])
                .unwrap(),
        ),
    });
    txn.push_batch(
        WorkerIntentBatch::new("many-aspects").push(MutationIntent::Create(
            CreateIntent::EntityAspects(EntityAspectCreateIntent {
                partition_id: worth_relational::facade::identity::PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw("record"),
                aspect_patch: PortableRecordAspectPatch::new(operations),
            }),
        )),
        execution.allocation_policy(),
    )
    .unwrap();
    let committed = txn.commit(&execution, &runtime).unwrap();
    let row = committed
        .changed_records
        .iter()
        .find_map(|r| match r {
            RecordRef::Entity(e) => Some(*e),
            _ => None,
        })
        .unwrap();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        key,
        CanonicalFieldPath::single(FieldKey::new("left").unwrap()),
    );
    let frame = basis
        .retain_field_content(&execution, row, KindId(1), &locator, id)
        .unwrap();
    assert_eq!(frame.bytes(), super::expected::FRAME);
    drop(frame);
    cancel.cancel();
    assert_eq!(
        basis
            .retain_field_content(&execution, row, KindId(1), &locator, id)
            .unwrap_err(),
        RelationalContentDenial::Execution(RelationalExecutionDenialCause::Cancelled)
    );
}
