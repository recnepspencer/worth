//! Preallocated one-shot transfer from publication to its original active attempt.
use super::super::application_attempt::{
    WorthQueryApplicationCleanupRequired as CleanupRequired,
    WorthQueryPendingApplicationCommitReceipt as Pending,
};
use super::WorthQueryPrimaryGraphCommittedApplication as Completed;
use crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding;
use std::sync::{Arc, Mutex, OnceLock};

// Erase the owned Native graph here so consumers never repeat its auto-trait
// traversal through provider-attempt state. The concrete owner remains sealed.
pub(in crate::domain_computation::primary_graph) trait OriginalCompletionSlot:
    Send + Sync
{
    fn complete(&self, completed: Completed);
    fn take(&self) -> Option<Completed>;
    fn retain_cleanup(&self, original: Pending);
    fn original_cleanup(&self) -> &Pending;
}
struct PendingOriginalSlot {
    completion: Mutex<Option<Completed>>,
    cleanup: OnceLock<Pending>,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl OriginalCompletionSlot for PendingOriginalSlot {
    fn complete(&self, completed: Completed) {
        let mut target = self
            .completion
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            target.is_none(),
            "one performing owner fills its original slot once"
        );
        *target = Some(completed);
    }
    fn take(&self) -> Option<Completed> {
        self.completion
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }
    fn retain_cleanup(&self, original: Pending) {
        assert!(
            self.cleanup.set(original).is_ok(),
            "only the actual original supplies cleanup custody once"
        );
    }
    fn original_cleanup(&self) -> &Pending {
        self.cleanup
            .get()
            .expect("cleanup carrier is issued only after its original is retained")
    }
}
type OriginalSlot = Arc<dyn OriginalCompletionSlot>;
pub(in crate::domain_computation::primary_graph) struct OriginalCompletionPublisher(
    Mutex<Option<OriginalSlot>>,
);
pub struct OriginalCompletionReceiver {
    slot: OriginalSlot,
    envelope: Box<CleanupRequired>,
}
impl OriginalCompletionPublisher {
    pub(in crate::domain_computation::primary_graph) fn pending_slot(
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(Self, OriginalCompletionReceiver), worth_execution::ExecutionAllocationDenial>
    {
        let funding = policy.reserve_memory(
            (std::mem::size_of::<PendingOriginalSlot>() + 2 * std::mem::size_of::<usize>()) as u64,
        )?;
        let envelope = CleanupRequired::reserve(policy)?;
        policy.check_live()?;
        let slot: OriginalSlot = Arc::new(PendingOriginalSlot {
            completion: Mutex::new(None),
            cleanup: OnceLock::new(),
            _funding: funding,
        });
        Ok((
            Self(Mutex::new(Some(Arc::clone(&slot)))),
            OriginalCompletionReceiver { slot, envelope },
        ))
    }
    pub(in crate::domain_computation::primary_graph) fn take(&self) -> Self {
        Self(Mutex::new(
            self.0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take(),
        ))
    }
    pub(in crate::domain_computation::primary_graph) fn complete(self, completed: Completed) {
        if let Some(slot) = self
            .0
            .into_inner()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        {
            slot.complete(completed);
        }
    }
}
impl OriginalCompletionReceiver {
    pub(in crate::domain_computation::primary_graph) fn retain_cleanup(
        &self,
        original: Pending,
    ) -> super::super::application_attempt::RetainedOriginalCleanupCommit {
        self.slot.retain_cleanup(original);
        super::super::application_attempt::RetainedOriginalCleanupCommit::new(Arc::clone(
            &self.slot,
        ))
    }
    pub(in crate::domain_computation::primary_graph) fn cleanup_envelope(
        self,
    ) -> Box<CleanupRequired> {
        self.envelope
    }

    pub(in crate::domain_computation::primary_graph) fn take(
        &self,
        session: &WorthQueryProviderSessionTerminalBinding,
    ) -> Option<Completed> {
        let completed = self.slot.take()?;
        assert!(
            completed
                .commit_evidence()
                .provider_session_binding()
                .same_session(session),
            "direct original completion belongs to its performing session"
        );
        Some(completed)
    }
}

#[cfg(test)]
#[path = "original_attempt_completion/funding_tests.rs"]
mod funding_tests;
