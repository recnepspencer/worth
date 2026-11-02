//! Final-candidate closure, rather than draft timing, decides attachment custody.
use super::{fixture::*, original, transaction::*};
use worth_execution::{CancellationSource, ExecutionAllocationPolicy, ExecutionRequest};
use worth_foundational::facade::ContentRefId;
use worth_relational::facade::{
    content::RelationalContentDenial, mvcc::RelationalTransactionIntent, transactions::*,
};

#[test]
fn unreferenced_attachment_refuses_with_literal_identity_and_releases_custody() {
    let cancel = CancellationSource::new();
    let original = original(&cancel);
    let execution = ExecutionRequest::serial(&original);
    let (runtime, contract) = fixture();
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
    let mut used = grid();
    used.samples[0] = 19;
    let identity = transaction
        .admit_content(&execution, FORMAT, &used)
        .unwrap();
    transaction
        .push_batch(
            batch(&contract, identity),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(original.memory().unwrap().available_memory_bytes() < before);
    let TransactionCommitError::Conflict { error, .. } =
        transaction.commit(&execution, &runtime).unwrap_err()
    else {
        panic!("expected an exact attachment refusal");
    };
    // Independently computed from the literal sample-grid frame, not the writer's ID.
    let golden =
        ContentRefId::from_hex("f4fb54508d898711c5770a58e4cff9989c1c302e7969d842891c0f5f02dc78e8")
            .unwrap();
    assert_eq!(
        error.class,
        ConflictClass::ContentAdmission {
            denial: RelationalContentDenial::UnreferencedAttachment { identity: golden },
        }
    );
    let (_, selected) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert_eq!(selected.descriptor(), basis.descriptor());
    assert_eq!(original.memory().unwrap().available_memory_bytes(), before);

    // An attachment already referenced by an unchanged record is not surplus.
    let mut first = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let identity = first.admit_content(&execution, FORMAT, &grid()).unwrap();
    first
        .push_batch(
            batch(&contract, identity),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    first.commit(&execution, &runtime).unwrap();
    let (_, selected) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut next = runtime
        .begin_branch_transaction(&selected, RelationalTransactionIntent::ordinary())
        .unwrap();
    next.admit_content(&execution, FORMAT, &grid()).unwrap();
    let identity = next.admit_content(&execution, FORMAT, &used).unwrap();
    let mut second = batch(&contract, identity);
    let MutationIntent::Create(CreateIntent::EntityAspects(intent)) = &mut second.intents[0] else {
        panic!("fixture create");
    };
    intent.client_key = worth_relational::facade::symbols::ClientKey::raw("second-grid");
    next.push_batch(second, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    next.commit(&execution, &runtime).unwrap();
}
