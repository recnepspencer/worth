pub(super) fn denied(
    denial: super::UiAllocationReplanTransactionCommitDenial,
) -> super::UiAllocationReplanTransactionOutcome {
    super::UiAllocationReplanTransactionOutcome::Denied(denial)
}

pub(super) fn retain_denial<T>(
    state: &mut super::ledger_state::UiAllocationReceiptLedgerState,
    transaction: &super::UiAllocationReplanTransaction,
    denial: super::UiAllocationReplanTransactionCommitDenial,
) -> T
where
    T: From<super::UiAllocationReplanTransactionOutcome>,
{
    state
        .denied_transactions
        .edit_or_default(transaction.idempotency_key(), |bucket| {
            if let Some((_, retained)) = bucket
                .iter_mut()
                .find(|(item, _)| item.same_idempotency_basis(transaction))
            {
                *retained = denial;
            } else {
                bucket.push((transaction.clone(), denial));
            }
        });
    super::UiAllocationReplanTransactionOutcome::Denied(denial).into()
}
