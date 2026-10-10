use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Ten decimal digits; changing text width requires another nine billion instances.
const FIRST_RUNTIME_INSTANCE_ID: u64 = 1_000_000_000;

#[derive(Debug, Clone)]
pub(super) struct RuntimeSequenceState {
    runtime_instance_id: u64,
    next_transaction_id: Arc<AtomicU64>,
    next_proposal_ordinal: Arc<AtomicU64>,
}

impl RuntimeSequenceState {
    pub(super) fn new() -> Self {
        static NEXT_RUNTIME_INSTANCE_ID: AtomicU64 = AtomicU64::new(FIRST_RUNTIME_INSTANCE_ID);
        Self {
            runtime_instance_id: NEXT_RUNTIME_INSTANCE_ID.fetch_add(1, Ordering::Relaxed),
            next_transaction_id: Arc::new(AtomicU64::new(1)),
            next_proposal_ordinal: Arc::new(AtomicU64::new(1)),
        }
    }

    pub(super) fn next_transaction_id(&self) -> crate::transactions::data::TransactionId {
        crate::transactions::data::TransactionId(
            self.next_transaction_id.fetch_add(1, Ordering::Relaxed),
        )
    }

    pub(super) fn next_proposal_ordinal(&self) -> Option<u64> {
        self.next_proposal_ordinal
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                (current != 0).then(|| current.checked_add(1).unwrap_or(0))
            })
            .ok()
    }

    pub(super) fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }
}

impl Default for RuntimeSequenceState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{RuntimeSequenceState, FIRST_RUNTIME_INSTANCE_ID};
    use std::sync::atomic::Ordering;

    #[test]
    fn runtime_instance_ids_keep_branch_reference_text_width() {
        let first = RuntimeSequenceState::new().runtime_instance_id();
        let next = RuntimeSequenceState::new().runtime_instance_id();
        assert!(first >= FIRST_RUNTIME_INSTANCE_ID);
        assert!(next >= FIRST_RUNTIME_INSTANCE_ID);
        assert_eq!(
            format!("relational/{first}/main").len(),
            format!("relational/{next}/main").len()
        );
    }

    #[test]
    fn proposal_ordinal_exhaustion_does_not_repeat_the_final_value() {
        let sequence = RuntimeSequenceState::new();
        sequence
            .next_proposal_ordinal
            .store(u64::MAX, Ordering::Relaxed);

        assert_eq!(sequence.next_proposal_ordinal(), Some(u64::MAX));
        assert_eq!(sequence.next_proposal_ordinal(), None);
        assert_eq!(sequence.next_proposal_ordinal(), None);
    }
}
