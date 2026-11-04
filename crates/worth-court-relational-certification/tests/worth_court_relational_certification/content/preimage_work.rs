//! Lookup work follows real field/aspect declarations, even with unrelated aspects.
use super::{
    fixture::{grid, Codec},
    original,
};
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest, SerialRequest};
use worth_foundational::{
    facade::*, DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    content::*, identity::KindId, mvcc::*, runtime::*, schema::*, symbols::ClientKey,
    transactions::*,
};
#[test]
fn content_original_field_capture_charges_real_aspect_count_and_key_bytes() {
    let mut thresholds = Vec::new();
    for (others, width) in [(0, 8), (127, 8), (127, 256)] {
        let cancel = CancellationSource::new();
        let owner = original(&cancel);
        let execution = ExecutionRequest::serial(&owner);
        let key = AspectKey::new(format!("samples{}", "x".repeat(width))).unwrap();
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
        let mut aspects = vec![DeclaredAspectContractBinding {
            binding: AspectBinding::EntityField {
                field: FieldKey::new("samples").unwrap(),
            },
            contract: contract.clone(),
        }];
        let mut operations = Vec::new();
        for n in 0..others {
            let name = format!("unrelated{n:04}");
            let extra = AspectContract::scalar(
                AspectKey::new(&name).unwrap(),
                AspectIdentity(1000 + n as u64),
                AspectContractRevision(1),
                ScalarAspectType::String,
            );
            operations.push(PortableAspectPatchOperation::SetWhole {
                basis: PortableAspectContractBasis::from_contract(&extra),
                value: ContractValidationInput::Scalar(AspectValue::String("unchanged".into())),
            });
            aspects.push(DeclaredAspectContractBinding {
                binding: AspectBinding::EntityField {
                    field: FieldKey::new(&name).unwrap(),
                },
                contract: extra,
            });
        }
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
                StructAspectValue::new([(
                    FieldKey::new("left").unwrap(),
                    AspectValue::ContentRef(id),
                )])
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
        let mut low = 0u64;
        let mut high = 65_536u64;
        while low < high {
            let work = low + (high - low) / 2;
            let policy = ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    Some(2 * 1024 * 1024),
                    Some(work),
                ),
            );
            let request = SerialRequest::from_policy(&policy, cancel.token(), None);
            let before = request.memory().unwrap().available_memory_bytes();
            match basis.retain_field_content(
                &ExecutionRequest::serial(&request),
                row,
                KindId(1),
                &locator,
                id,
            ) {
                Ok(frame) => {
                    assert_eq!(frame.bytes(), super::expected::FRAME);
                    high = work;
                }
                Err(RelationalContentDenial::Execution(
                    RelationalExecutionDenialCause::WorkExhausted,
                )) => {
                    low = work + 1;
                }
                other => panic!("lookup returned another value/cause: {other:?}"),
            }
            assert_eq!(request.memory().unwrap().available_memory_bytes(), before);
        }
        thresholds.push(low);
        cancel.cancel();
        assert_eq!(
            basis
                .retain_field_content(&execution, row, KindId(1), &locator, id)
                .unwrap_err(),
            RelationalContentDenial::Execution(RelationalExecutionDenialCause::Cancelled)
        );
    }
    assert!(
        thresholds[1] > thresholds[0],
        "unrelated aspects must be charged: {thresholds:?}"
    );
    assert!(
        thresholds[2] > thresholds[1] + 1000,
        "long exact keys must be charged: {thresholds:?}"
    );
    eprintln!("original Native field capture work thresholds: {thresholds:?}");
}
