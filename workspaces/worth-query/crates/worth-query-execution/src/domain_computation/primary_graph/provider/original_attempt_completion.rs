//! Preallocated one-shot transfer from publication to its original active attempt.
use super::WorthQueryPrimaryGraphCommittedApplication as Completed;
use crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding;
use std::sync::{Arc, Mutex};

// Erase the owned Native graph here so consumers never repeat its auto-trait
// traversal through provider-attempt state. The concrete owner remains sealed.
trait OriginalCompletionSlot: Send + Sync {
    fn complete(&self, completed: Completed);
    fn take(&self) -> Option<Completed>;
}
struct PendingOriginalSlot(Mutex<Option<Completed>>);
impl OriginalCompletionSlot for PendingOriginalSlot {
    fn complete(&self, completed: Completed) {
        let mut target = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            target.is_none(),
            "one performing owner fills its original slot once"
        );
        *target = Some(completed);
    }
    fn take(&self) -> Option<Completed> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }
}
type OriginalSlot = Arc<dyn OriginalCompletionSlot>;
pub(in crate::domain_computation::primary_graph) struct OriginalCompletionPublisher(
    Mutex<Option<OriginalSlot>>,
);
pub(in crate::domain_computation::primary_graph) struct OriginalCompletionReceiver(OriginalSlot);
impl OriginalCompletionPublisher {
    pub(in crate::domain_computation::primary_graph) fn pending_slot(
    ) -> (Self, OriginalCompletionReceiver) {
        // Allocate before the provider can perform an effect. Cutover only moves
        // the actual result into this fixed slot; it allocates no message queue.
        let slot: OriginalSlot = Arc::new(PendingOriginalSlot(Mutex::new(None)));
        (
            Self(Mutex::new(Some(Arc::clone(&slot)))),
            OriginalCompletionReceiver(slot),
        )
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
    pub(in crate::domain_computation::primary_graph) fn take(
        &self,
        session: &WorthQueryProviderSessionTerminalBinding,
    ) -> Option<Completed> {
        let completed = self.0.take()?;
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
