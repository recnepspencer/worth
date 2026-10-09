use crate::facade::transactions::EntityAspectCreateIntent;
use crate::tests::support::*;
use worth_foundational::facade::{
    AspectContract, AspectContractRevision, AspectIdentity, AspectKey, AspectValue,
    CanonicalBigInt, CanonicalDate, CanonicalDecimal, CanonicalF32, CanonicalF64,
    CanonicalRational, CanonicalTime, CanonicalTimestamp, CanonicalTimestampTz, ContentRefId,
    ContractValidatedAspectValueView, ContractValidationInput, EntityId as FoundationalEntityId,
    InternedString, PartitionId as FoundationalPartitionId, PortableAspectContractBasis,
    PortableAspectPatchOperation, PortableRecordAspectPatch,
};

#[test]
fn ordinary_native_mutation_roundtrips_every_foundational_scalar_family() {
    let values = scalar_samples();
    let contracts = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            AspectContract::scalar(
                key(index),
                AspectIdentity(index as u64 + 1),
                AspectContractRevision(1),
                value.value_family(),
            )
        })
        .collect::<Vec<_>>();
    let fixture = AspectSchemaFixture {
        entity_aspects: contracts
            .iter()
            .enumerate()
            .map(|(index, contract)| DeclaredAspectContractBinding {
                binding: AspectBinding::EntityField {
                    field: field_key(&format!("value_{index}")),
                },
                contract: contract.clone(),
            })
            .collect(),
        ..AspectSchemaFixture::default()
    };
    let patch =
        PortableRecordAspectPatch::new(contracts.iter().zip(&values).map(|(contract, value)| {
            PortableAspectPatchOperation::SetWhole {
                basis: PortableAspectContractBasis::from_contract(contract),
                value: ContractValidationInput::Scalar(value.clone()),
            }
        }));
    let runtime = fixture.build_runtime();
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("native-scalar-matrix").push(MutationIntent::Create(
                CreateIntent::EntityAspects(EntityAspectCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("native-scalar-matrix"),
                    aspect_patch: patch,
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");

    let committed = transaction
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let entity = changed_entities(&committed)[0];
    {
        let read = runtime
            .read_truth()
            .read_snapshot(&committed.snapshot)
            .unwrap();
        assert_scalar_state(
            read.get_entity(entity)
                .unwrap()
                .authoritative_aspect_state
                .as_ref()
                .unwrap(),
            &contracts,
            &values,
            "ordinary mutation",
        );
    }
    let recovery_fixture = fixture.clone();
    let (_, recovered) =
        checkpoint_and_recover_with(&runtime, move || recovery_fixture.build_runtime());
    let read = recovered.read_truth().read_version(committed.version_id);
    assert_scalar_state(
        read.get_entity(entity)
            .unwrap()
            .authoritative_aspect_state
            .as_ref()
            .unwrap(),
        &contracts,
        &values,
        "checkpoint readmission",
    );
}

fn assert_scalar_state(
    state: &worth_foundational::facade::AuthoritativeRecordAspectState,
    contracts: &[AspectContract],
    values: &[AspectValue],
    context: &str,
) {
    for ((contract, expected), index) in contracts.iter().zip(values).zip(0..) {
        let actual = state.get(contract.key()).unwrap();
        assert!(
            matches!(actual.view(), ContractValidatedAspectValueView::Scalar(value) if value == expected),
            "native family {index} changed across {context}: {actual:?}"
        );
    }
}

fn scalar_samples() -> Vec<AspectValue> {
    vec![
        AspectValue::Null,
        AspectValue::Bool(true),
        AspectValue::Int8(-7),
        AspectValue::Int16(-320),
        AspectValue::Int32(-32_000),
        AspectValue::Int64(-12),
        AspectValue::UInt8(7),
        AspectValue::UInt16(320),
        AspectValue::UInt32(32_000),
        AspectValue::UInt64(12),
        AspectValue::Float32(CanonicalF32::from_f32(1.5)),
        AspectValue::Float64(CanonicalF64::from_f64(2.5)),
        AspectValue::Decimal(CanonicalDecimal::new("12.50")),
        AspectValue::BigInt(CanonicalBigInt::new("-12345678901234567890")),
        AspectValue::Rational(
            CanonicalRational::new(CanonicalBigInt::new("22"), CanonicalBigInt::new("7")).unwrap(),
        ),
        AspectValue::String(InternedString::Raw("alpha".to_string())),
        AspectValue::Bytes(ContentRefId(41)),
        AspectValue::Uuid([7; 16]),
        AspectValue::Date(CanonicalDate {
            days_from_unix_epoch: 20_000,
        }),
        AspectValue::Time(CanonicalTime::new(1_000).unwrap()),
        AspectValue::Timestamp(CanonicalTimestamp {
            micros_since_unix_epoch: 123_456,
        }),
        AspectValue::TimestampTz(CanonicalTimestampTz {
            utc_micros_since_unix_epoch: 123_456,
            offset_minutes: -360,
        }),
        AspectValue::EntityRef(FoundationalEntityId::new(
            FoundationalPartitionId(9),
            10,
            11,
        )),
        AspectValue::ContentRef(ContentRefId(42)),
    ]
}

fn key(index: usize) -> AspectKey {
    AspectKey::new(format!("native_{index}")).unwrap()
}
