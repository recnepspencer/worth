//! The real Native transaction checks reference closure before selected publication.
use super::{fixture::*, original};
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_foundational::facade::*;
use worth_relational::facade::{
    identity::KindId, mvcc::*, runtime::*, schema::*, symbols::ClientKey, transactions::*,
};

pub(crate) const FORMAT: CanonicalDigestId = CanonicalDigestId::new([11; 32]);
const OTHER: CanonicalDigestId = CanonicalDigestId::new([14; 32]);
const NEW_VERSION: CanonicalDigestId = CanonicalDigestId::new([15; 32]);
const LEFT: CanonicalDigestId = CanonicalDigestId::new([12; 32]);
const RIGHT: CanonicalDigestId = CanonicalDigestId::new([13; 32]);

pub(super) fn fixture_builder() -> (RelationalRuntimeBuilder, AspectContract) {
    let (registry, contract) = fixture_registry(
        Arc::new(Codec {
            descriptor: ContentFormat::new("sample-grid", 1).unwrap(),
            alter: false,
        }),
        SchemaVersionId(1),
    );
    (
        RelationalRuntimeApi::builder().schema_registry(registry),
        contract,
    )
}

pub(crate) fn fixture_registry(
    codec: Arc<dyn worth_relational::facade::content::RelationalContentCodec>,
    schema_version_id: SchemaVersionId,
) -> (RelationalSchemaRegistry, AspectContract) {
    let fields = [("left", LEFT), ("right", RIGHT)].map(|(name, binding)| {
        FieldDeclaration::new(
            FieldKey::new(name).unwrap(),
            ScalarAspectType::ContentRef,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .unwrap()
        .with_codec(binding)
    });
    let contract = AspectContract::struct_aspect(
        AspectKey::new("samples").unwrap(),
        AspectIdentity(741),
        AspectContractRevision(1),
        StructAspectShape::new(fields).unwrap(),
    );
    let fixture = worth_testkit_relational::aspect_storage::AspectSchemaFixture {
        schema_version_id,
        entity_aspects: vec![DeclaredAspectContractBinding {
            binding: AspectBinding::EntityField {
                field: FieldKey::new("samples").unwrap(),
            },
            contract: contract.clone(),
        }],
        ..Default::default()
    };
    let registry = fixture
        .build_registry()
        .register_content_format(FORMAT, codec)
        .unwrap()
        .bind_content_field(LEFT, FORMAT)
        .unwrap()
        .bind_content_field(RIGHT, FORMAT)
        .unwrap();
    let registry = registry
        .register_content_format(
            OTHER,
            Arc::new(Codec {
                descriptor: ContentFormat::new("other-grid", 1).unwrap(),
                alter: false,
            }),
        )
        .unwrap()
        .register_content_format(
            NEW_VERSION,
            Arc::new(Codec {
                descriptor: ContentFormat::new("sample-grid", 2).unwrap(),
                alter: false,
            }),
        )
        .unwrap();
    (registry, contract)
}

pub(crate) fn fixture() -> (RelationalRuntime, AspectContract) {
    let (builder, contract) = fixture_builder();
    (builder.build().unwrap(), contract)
}

pub(crate) fn batch(contract: &AspectContract, identity: ContentRefId) -> WorkerIntentBatch {
    let value = StructAspectValue::new(
        [("left", identity), ("right", identity)]
            .map(|(name, id)| (FieldKey::new(name).unwrap(), AspectValue::ContentRef(id))),
    )
    .unwrap();
    let patch = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(contract),
        value: ContractValidationInput::Struct(value),
    }]);
    WorkerIntentBatch::new("content").push(MutationIntent::Create(CreateIntent::EntityAspects(
        EntityAspectCreateIntent {
            partition_id: worth_relational::facade::identity::PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw("grid"),
            aspect_patch: patch,
        },
    )))
}

#[test]
fn content_transaction_publishes_two_fields_only_with_native_admitted_content() {
    let cancel = CancellationSource::new();
    let original = original(&cancel);
    let execution = ExecutionRequest::serial(&original);
    let (runtime, contract) = fixture();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let identity = transaction
        .admit_content(&execution, FORMAT, &grid())
        .unwrap();
    transaction
        .push_batch(
            batch(&contract, identity),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let committed = transaction.commit(&execution, &runtime).unwrap();
    let entity = committed
        .changed_records
        .iter()
        .find_map(|record| match record {
            RecordRef::Entity(id) => Some(*id),
            _ => None,
        })
        .unwrap();
    let read = runtime
        .read_truth()
        .read_snapshot(&committed.snapshot, &execution)
        .unwrap()
        .unwrap();
    let actual_record = read.get_entity(entity).unwrap();
    let state = actual_record.authoritative_aspect_state.as_ref().unwrap();
    let ContractValidatedAspectValueView::Struct(values) =
        state.get(contract.key()).unwrap().view()
    else {
        panic!("expected Native struct");
    };
    for field in ["left", "right"] {
        assert_eq!(
            values.get(&FieldKey::new(field).unwrap()),
            Some(&AspectValue::ContentRef(identity))
        );
    }
    let (_, selected) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert_ne!(
        selected.descriptor().root_identity(),
        basis.descriptor().root_identity()
    );
    let actual = selected.read_content(&execution, identity, FORMAT).unwrap();
    assert_eq!(actual.bytes(), super::expected::FRAME);
}

#[test]
fn content_missing_reference_refuses_before_selected_commit_changes() {
    let cancel = CancellationSource::new();
    let original = original(&cancel);
    let execution = ExecutionRequest::serial(&original);
    let (runtime, contract) = fixture();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            batch(&contract, ContentRefId([0xff; 32])),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let error = transaction.commit(&execution, &runtime).unwrap_err();
    assert!(matches!(
        error,
        TransactionCommitError::Conflict {
            error: CommitConflict {
                class: ConflictClass::ContentAdmission {
                    denial:
                        worth_relational::facade::content::RelationalContentDenial::MissingContent
                },
                ..
            },
            ..
        }
    ));
    let (_, selected) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert_eq!(
        selected.descriptor().root_identity(),
        basis.descriptor().root_identity()
    );
}

#[test]
fn content_transaction_drop_releases_original_funded_staging() {
    let cancel = CancellationSource::new();
    let original = original(&cancel);
    let execution = ExecutionRequest::serial(&original);
    let (runtime, _) = fixture();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let before = original.memory().unwrap().available_memory_bytes();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .admit_content(&execution, FORMAT, &grid())
        .unwrap();
    assert!(original.memory().unwrap().available_memory_bytes() < before);
    cancel.cancel();
    drop(transaction);
    assert_eq!(original.memory().unwrap().available_memory_bytes(), before);
}

#[test]
fn content_wrong_format_and_version_refuse_before_selected_publication() {
    use worth_relational::facade::content::RelationalContentDenial as Denial;
    for (format, expected) in [
        (OTHER, Denial::FormatMismatch),
        (NEW_VERSION, Denial::UnsupportedVersion),
    ] {
        let cancel = CancellationSource::new();
        let original = original(&cancel);
        let execution = ExecutionRequest::serial(&original);
        let (runtime, contract) = fixture();
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let mut transaction = runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        let identity = transaction
            .admit_content(&execution, format, &grid())
            .unwrap();
        transaction
            .push_batch(
                batch(&contract, identity),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let error = transaction.commit(&execution, &runtime).unwrap_err();
        let TransactionCommitError::Conflict { error, .. } = error else {
            panic!("unexpected content denial: {error:?}");
        };
        assert!(
            matches!(error.class, ConflictClass::ContentAdmission { denial } if denial == expected)
        );
        let (_, selected) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        assert_eq!(
            selected.descriptor().root_identity(),
            basis.descriptor().root_identity()
        );
    }
}

#[test]
fn content_savepoint_releases_only_rolled_back_original_staging() {
    let cancel = CancellationSource::new();
    let original = original(&cancel);
    let execution = ExecutionRequest::serial(&original);
    let (runtime, _) = fixture();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let before = original.memory().unwrap().available_memory_bytes().unwrap();
    transaction
        .admit_content(&execution, FORMAT, &grid())
        .unwrap();
    let first = original.memory().unwrap().available_memory_bytes().unwrap();
    let savepoint = transaction.create_savepoint().unwrap();
    transaction
        .admit_content(&execution, FORMAT, &grid())
        .unwrap();
    assert!(original.memory().unwrap().available_memory_bytes().unwrap() < first);
    transaction.rollback_to_savepoint(savepoint).unwrap();
    assert_eq!(
        original.memory().unwrap().available_memory_bytes(),
        Some(first)
    );
    assert!(first < before);
    drop(transaction);
    assert_eq!(
        original.memory().unwrap().available_memory_bytes(),
        Some(before)
    );
}

#[test]
fn content_identity_from_another_owner_is_missing_without_local_admission() {
    let cancel = CancellationSource::new();
    let original = original(&cancel);
    let execution = ExecutionRequest::serial(&original);
    let (left, _) = fixture();
    let (_, left_basis) = left.observe_branch(&left.main_branch_identity()).unwrap();
    let mut left_transaction = left
        .begin_branch_transaction(&left_basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let identity = left_transaction
        .admit_content(&execution, FORMAT, &grid())
        .unwrap();
    let (right, contract) = fixture();
    let (_, right_basis) = right.observe_branch(&right.main_branch_identity()).unwrap();
    let mut transaction = right
        .begin_branch_transaction(&right_basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            batch(&contract, identity),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let TransactionCommitError::Conflict { error, .. } =
        transaction.commit(&execution, &right).unwrap_err()
    else {
        panic!("expected Native content refusal");
    };
    assert!(matches!(
        error.class,
        ConflictClass::ContentAdmission {
            denial: worth_relational::facade::content::RelationalContentDenial::MissingContent
        }
    ));
    assert_eq!(
        right
            .observe_branch(&right.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        right_basis.descriptor()
    );
}
