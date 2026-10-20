//! Test-local contacts at the split commit's owner boundaries.
use std::cell::Cell;
thread_local! {
    static CONTACTS: Cell<(usize, usize, usize)> = const { Cell::new((0, 0, 0)) };
}
pub(crate) fn admission() {
    CONTACTS.with(|counts| {
        let (admissions, revalidations, captures) = counts.get();
        counts.set((admissions + 1, revalidations, captures));
    });
}
pub(crate) fn revalidation() {
    CONTACTS.with(|counts| {
        let (admissions, revalidations, captures) = counts.get();
        counts.set((admissions, revalidations + 1, captures));
    });
}

pub(crate) fn binding_capture() {
    CONTACTS.with(|counts| {
        let (admissions, revalidations, captures) = counts.get();
        counts.set((admissions, revalidations, captures + 1));
    });
}

#[test]
fn split_entries_consult_before_admission_and_currency_contacts() {
    use crate::tests::support::runtime_with_test_schema;
    use crate::transactions::data::{CommitExecutionDenialKind, TransactionCommitError};
    use worth_execution::{
        CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest,
    };
    for entry in ["validate", "prepare transaction", "prepare proposal"] {
        let runtime = runtime_with_test_schema();
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let transaction = runtime
            .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
            .unwrap();
        let cancellation = CancellationSource::new();
        let backing =
            SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
        let execution = ExecutionRequest::serial(&backing);
        let (proposal, transaction) = if entry == "prepare proposal" {
            (
                Some(
                    runtime
                        .validate_branch_transaction(&execution, transaction)
                        .unwrap(),
                ),
                None,
            )
        } else {
            (None, Some(transaction))
        };
        cancellation.cancel();
        CONTACTS.with(|counts| counts.set((0, 0, 0)));
        let error = match proposal {
            Some(proposal) => runtime
                .prepare_validated_proposal(&execution, proposal)
                .unwrap_err(),
            None if entry == "validate" => runtime
                .validate_branch_transaction(&execution, transaction.unwrap())
                .unwrap_err(),
            None => runtime
                .prepare_branch_transaction(&execution, transaction.unwrap())
                .unwrap_err(),
        };
        let TransactionCommitError::Execution { denial, .. } = error else {
            panic!("{entry}: cancellation must retain its typed cause");
        };
        assert_eq!(
            denial.kind,
            CommitExecutionDenialKind::Cause(
                crate::execution::RelationalExecutionDenialCause::Cancelled,
            ),
            "{entry}"
        );
        assert_eq!(
            CONTACTS.with(Cell::get),
            (0, 0, 0),
            "{entry}: no owner contact before consult"
        );
    }
}

#[test]
fn admitted_validation_reaches_the_counted_owner_binding() {
    let runtime = crate::tests::support::runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let transaction = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    let backing = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&backing);
    CONTACTS.with(|counts| counts.set((0, 0, 0)));
    runtime
        .validate_branch_transaction(&execution, transaction)
        .unwrap();
    // Validation really does capture the owner binding once. Its cancelled
    // twin must refuse before this same reachable contact, not only admission.
    assert_eq!(CONTACTS.with(Cell::get), (0, 0, 1));
}
