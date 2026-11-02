//! Preallocated one-shot transfer from publication to its original active attempt.
use super::WorthQueryPrimaryGraphCommittedApplication as Completed;
use crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding;
use std::sync::{Arc, Mutex};

type OriginalSlot = Arc<Mutex<Option<Completed>>>;
pub(in crate::domain_computation::primary_graph) struct OriginalCompletionPublisher(
    Mutex<Option<OriginalSlot>>,
);
pub(in crate::domain_computation::primary_graph) struct OriginalCompletionReceiver(OriginalSlot);
impl OriginalCompletionPublisher {
    pub(in crate::domain_computation::primary_graph) fn pending_slot(
    ) -> (Self, OriginalCompletionReceiver) {
        // Allocate before the provider can perform an effect. Cutover only moves
        // the actual result into this fixed slot; it allocates no message queue.
        let slot = Arc::new(Mutex::new(None));
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
            let mut target = slot
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            assert!(
                target.is_none(),
                "one original performing owner fills its slot once"
            );
            *target = Some(completed);
        }
    }
}
impl OriginalCompletionReceiver {
    pub(in crate::domain_computation::primary_graph) fn take(
        &self,
        session: &WorthQueryProviderSessionTerminalBinding,
    ) -> Option<Completed> {
        let completed = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()?;
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
