//! Raw Native mutation cannot bypass the declared field/format contract.
use super::original;
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_foundational::facade::*;
use worth_relational::facade::{
    mvcc::RelationalTransactionIntent, schema::DeclaredAspectContractBinding, symbols::ClientKey,
    transactions::*,
};

#[test]
fn content_raw_scalar_reference_has_no_unbound_publication_bypass() {
    let cancel = CancellationSource::new();
    let original = original(&cancel);
    let execution = ExecutionRequest::serial(&original);
    let contract = AspectContract::scalar(
        AspectKey::new("samples").unwrap(),
        AspectIdentity(51),
        AspectContractRevision(1),
        ScalarAspectType::ContentRef,
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
    let runtime = fixture.build_runtime();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let patch = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(&contract),
        value: ContractValidationInput::Scalar(AspectValue::ContentRef(ContentRefId([7; 32]))),
    }]);
    transaction
        .push_batch(
            WorkerIntentBatch::new("raw-content").push(MutationIntent::Create(
                CreateIntent::EntityAspects(EntityAspectCreateIntent {
                    partition_id: worth_relational::facade::identity::PartitionId::main(),
                    kind_id: worth_relational::facade::identity::KindId(1),
                    client_key: ClientKey::raw("raw-content"),
                    aspect_patch: patch,
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let TransactionCommitError::Conflict { error, .. } =
        transaction.commit(&execution, &runtime).unwrap_err()
    else {
        panic!("expected typed content refusal");
    };
    assert!(matches!(
        error.class,
        ConflictClass::ContentAdmission {
            denial:
                worth_relational::facade::content::RelationalContentDenial::FormatBindingMissing
        }
    ));
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        basis.descriptor()
    );
}
