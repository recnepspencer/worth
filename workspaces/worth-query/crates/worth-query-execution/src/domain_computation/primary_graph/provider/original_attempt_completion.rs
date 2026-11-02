//! One original attempt's direct result handoff, never an idempotency index.
use super::WorthQueryPrimaryGraphCommittedApplication as Completed;
use crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding;
use std::sync::{mpsc, Mutex};

pub(in crate::domain_computation::primary_graph) struct OriginalCompletionPublisher(
    Mutex<Option<mpsc::Sender<Completed>>>,
);
pub(in crate::domain_computation::primary_graph) struct OriginalCompletionReceiver(
    mpsc::Receiver<Completed>,
);
impl OriginalCompletionPublisher {
    pub(in crate::domain_computation::primary_graph) fn channel(
    ) -> (Self, OriginalCompletionReceiver) {
        let (sender, receiver) = mpsc::channel();
        (
            Self(Mutex::new(Some(sender))),
            OriginalCompletionReceiver(receiver),
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
        if let Some(sender) = self
            .0
            .into_inner()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        {
            // A later recovery can complete publication after the original call
            // ended. That cannot resurrect its original managed completion.
            let _ = sender.send(completed);
        }
    }
}
impl OriginalCompletionReceiver {
    pub(in crate::domain_computation::primary_graph) fn take(
        &self,
        session: &WorthQueryProviderSessionTerminalBinding,
    ) -> Option<Completed> {
        let completed = self.0.try_recv().ok()?;
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
