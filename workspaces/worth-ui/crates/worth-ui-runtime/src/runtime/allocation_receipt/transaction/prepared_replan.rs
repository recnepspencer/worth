#[derive(Debug)]
pub(in crate::runtime) enum UiAllocationLedgerPreparation {
    Resolved(Box<super::UiAllocationReplanTransactionOutcome>),
    Prepared(Box<UiPreparedAllocationLedgerTransition>),
}

#[derive(Debug)]
pub(in crate::runtime) struct UiPreparedAllocationLedgerTransition {
    pub(super) predecessor: super::ledger_state::UiAllocationReceiptLedgerState,
    pub(super) successor: super::ledger_state::UiAllocationReceiptLedgerState,
    pub(super) committed: super::UiCommittedAllocationReplan,
    successor_candidates: Box<[crate::runtime::UiAllocationCandidate]>,
}

impl From<super::UiAllocationReplanTransactionOutcome> for UiAllocationLedgerPreparation {
    fn from(outcome: super::UiAllocationReplanTransactionOutcome) -> Self {
        Self::Resolved(Box::new(outcome))
    }
}

impl UiPreparedAllocationLedgerTransition {
    pub(super) fn new(
        predecessor: super::ledger_state::UiAllocationReceiptLedgerState,
        successor: super::ledger_state::UiAllocationReceiptLedgerState,
        committed: super::UiCommittedAllocationReplan,
        successor_candidates: Vec<crate::runtime::UiAllocationCandidate>,
    ) -> Self {
        Self {
            predecessor,
            successor,
            committed,
            successor_candidates: successor_candidates.into_boxed_slice(),
        }
    }

    pub(in crate::runtime) fn committed(&self) -> &super::UiCommittedAllocationReplan {
        &self.committed
    }

    pub(in crate::runtime) fn successor_candidates(
        &self,
    ) -> &[crate::runtime::UiAllocationCandidate] {
        &self.successor_candidates
    }

    pub(in crate::runtime) fn with_committed(
        mut self,
        committed: super::UiCommittedAllocationReplan,
    ) -> Self {
        let key = committed.transaction().idempotency_key();
        let replaced = self
            .successor
            .completed_transactions
            .get(&key)
            .and_then(|bucket| {
                let at = bucket.iter().position(|retained| {
                    retained
                        .transaction()
                        .same_idempotency_basis(committed.transaction())
                })?;
                let mut bucket = bucket.clone();
                bucket[at] = committed.clone();
                Some(bucket)
            });
        if let Some(bucket) = replaced {
            self.successor.completed_transactions.insert(key, bucket);
        }
        self.committed = committed;
        self
    }
}
