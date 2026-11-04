//! A later ordinary edit resolves through the retained Native directory.
use super::{
    fixture::grid,
    original,
    transaction::{batch, fixture, FORMAT},
};
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_relational::facade::{mvcc::*, symbols::ClientKey, transactions::*};

#[test]
fn content_ordinary_second_commit_reuses_retained_bytes_without_staging_them_again() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let (runtime, contract) = fixture();
    let (_, first) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&first, RelationalTransactionIntent::ordinary())
        .unwrap();
    let id = transaction
        .admit_content(&execution, FORMAT, &grid())
        .unwrap();
    transaction
        .push_batch(batch(&contract, id), execution.allocation_policy())
        .unwrap();
    transaction.commit(&execution, &runtime).unwrap();
    let (_, second) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&second, RelationalTransactionIntent::ordinary())
        .unwrap();
    let mut reuse = batch(&contract, id);
    let MutationIntent::Create(CreateIntent::EntityAspects(intent)) = &mut reuse.intents[0] else {
        panic!("fixture create");
    };
    intent.client_key = ClientKey::raw("second-grid");
    transaction
        .push_batch(reuse, execution.allocation_policy())
        .unwrap();
    transaction.commit(&execution, &runtime).unwrap();
    let (_, selected) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert_eq!(
        selected
            .read_content(&execution, id, FORMAT)
            .unwrap()
            .bytes(),
        super::expected::FRAME
    );
}
