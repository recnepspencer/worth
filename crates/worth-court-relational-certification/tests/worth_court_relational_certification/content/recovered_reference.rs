//! Stored bytes, descriptive read selectors and physical certificates are distinct.
use super::{
    fixture::{grid, Codec},
    original,
};
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_foundational::facade::*;
use worth_relational::facade::{
    content::*, durability::*, identity::KindId, mvcc::*, runtime::*, schema::*,
    symbols::ClientKey, transactions::*,
};
const A: CanonicalDigestId = CanonicalDigestId::new([61; 32]);
const B: CanonicalDigestId = CanonicalDigestId::new([62; 32]);

fn fixture() -> (RelationalRuntime, AspectContract) {
    let fields = [("left", A), ("right", B)].map(|(name, codec)| {
        FieldDeclaration::new(
            FieldKey::new(name).unwrap(),
            ScalarAspectType::ContentRef,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .unwrap()
        .with_codec(codec)
    });
    let contract = AspectContract::struct_aspect(
        AspectKey::new("samples").unwrap(),
        AspectIdentity(742),
        AspectContractRevision(1),
        StructAspectShape::new(fields).unwrap(),
    );
    let fixture = worth_testkit_relational::aspect_storage::AspectSchemaFixture {
        entity_aspects: vec![DeclaredAspectContractBinding {
            binding: AspectBinding::EntityField {
                field: FieldKey::new("samples").unwrap(),
            },
            contract: contract.clone(),
        }],
        ..Default::default()
    };
    let mut registry = fixture.build_registry();
    for (id, name) in [(A, "sample-grid"), (B, "other-grid")] {
        registry = registry
            .register_content_format(
                id,
                Arc::new(Codec {
                    descriptor: ContentFormat::new(name, 1).unwrap(),
                    alter: false,
                }),
            )
            .unwrap()
            .bind_content_field(id, id)
            .unwrap();
    }
    (
        RelationalRuntimeApi::builder()
            .schema_registry(registry)
            .build()
            .unwrap(),
        contract,
    )
}
fn captured(
    execution: &ExecutionRequest<'_, '_>,
) -> (RelationalNativeCheckpoint, ContentRefId, ContentRefId) {
    let (runtime, contract) = fixture();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let a = transaction.admit_content(execution, A, &grid()).unwrap();
    let b = transaction.admit_content(execution, B, &grid()).unwrap();
    let value = StructAspectValue::new(
        [("left", a), ("right", b)]
            .map(|(name, id)| (FieldKey::new(name).unwrap(), AspectValue::ContentRef(id))),
    )
    .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("two-formats").push(MutationIntent::Create(
                CreateIntent::EntityAspects(EntityAspectCreateIntent {
                    partition_id: worth_relational::facade::identity::PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("grid"),
                    aspect_patch: PortableRecordAspectPatch::new([
                        PortableAspectPatchOperation::SetWhole {
                            basis: PortableAspectContractBasis::from_contract(&contract),
                            value: ContractValidationInput::Struct(value),
                        },
                    ]),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    transaction.commit(execution, &runtime).unwrap();
    (
        runtime
            .durability_authority()
            .native_checkpoint(execution, execution.allocation_policy())
            .unwrap(),
        a,
        b,
    )
}
#[test]
fn content_recovered_read_swapped_reference_refuses_the_expected_field_format() {
    let cancel = CancellationSource::new();
    let original = original(&cancel);
    let execution = ExecutionRequest::serial(&original);
    let (checkpoint, a, b) = captured(&execution);
    assert_ne!(a, b, "format identity is part of the content digest");
    let (mut restored, _) = fixture();
    restored
        .durability_recovery()
        .restore_native_checkpoint(&execution, &checkpoint)
        .unwrap();
    let (_, selected) = restored
        .observe_branch(&restored.main_branch_identity())
        .unwrap();
    assert_eq!(
        selected.read_content(&execution, a, A).unwrap().bytes(),
        super::expected::FRAME
    );
    assert_eq!(
        selected.read_content(&execution, b, B).unwrap().payload(),
        &super::expected::FRAME[35..]
    );
    // The caller's descriptive reference is corrupted; both genuine stored frames
    // remain intact. This is a read-contract test, not a forged selected root.
    assert_eq!(
        selected.read_content(&execution, b, A).unwrap_err(),
        RelationalContentDenial::FormatMismatch
    );
    assert_eq!(
        selected.read_content(&execution, a, B).unwrap_err(),
        RelationalContentDenial::FormatMismatch
    );
    assert_eq!(
        selected
            .read_content(&execution, ContentRefId([0xff; 32]), A)
            .unwrap_err(),
        RelationalContentDenial::MissingContent
    );
    assert_eq!(
        restored
            .observe_branch(&restored.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        selected.descriptor()
    );
}

#[test]
fn content_stored_authoritative_field_byte_corruption_refuses_at_physical_integrity() {
    let cancel = CancellationSource::new();
    let original = original(&cancel);
    let execution = ExecutionRequest::serial(&original);
    let (checkpoint, a, _) = captured(&execution);
    // Independent MessagePack framing of the actual ContentRef value, not a
    // search for a random digest occurrence in the manifest or content directory.
    let mut field = b"\xaaContentRef\xdc\x00\x20".to_vec();
    for byte in a.0 {
        if byte >= 128 {
            field.push(0xcc);
        }
        field.push(byte);
    }
    let member = checkpoint
        .members()
        .iter()
        .find_map(|member| match member {
            RelationalCheckpointMember::New { bytes, .. }
                if bytes.starts_with(b"WORTH-REL-REG6\n")
                    && bytes.get(b"WORTH-REL-REG6\n".len() + 38) == Some(&0)
                    && bytes.windows(field.len()).any(|word| word == field) =>
            {
                Some(bytes)
            }
            _ => None,
        })
        .expect("actual physical record member contains the authoritative ContentRef value");
    let field_offset = member
        .windows(field.len())
        .position(|word| word == field)
        .unwrap();
    let mut corrupt = checkpoint.bytes().to_vec();
    let offset = corrupt
        .windows(member.len())
        .position(|word| word == member.as_ref())
        .unwrap();
    // Keep MessagePack framing and lengths unchanged, but invalidate the actual
    // field identity. Do not forge physical/history/root commitments to bypass them.
    corrupt[offset + field_offset + field.len() - 1] ^= 1;
    let corrupt = RelationalNativeCheckpoint::from_untrusted_bytes(corrupt);
    let (mut target, _) = fixture();
    let (_, before) = target
        .observe_branch(&target.main_branch_identity())
        .unwrap();
    let refused = target
        .durability_recovery()
        .restore_native_checkpoint(&execution, &corrupt)
        .unwrap_err();
    let RelationalRecoveryStop::Durability(cause) = refused else {
        panic!("{refused:?}")
    };
    assert_eq!(cause.class, RecoveryFailureClass::CorruptCheckpoint);
    assert_eq!(
        cause.detail,
        "inline checkpoint member digest or uniqueness mismatch"
    );
    assert_eq!(
        target
            .observe_branch(&target.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        before.descriptor()
    );
}
