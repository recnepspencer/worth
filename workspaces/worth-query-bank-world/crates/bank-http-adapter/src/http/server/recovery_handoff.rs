//! Listener-independent access delegates to the existing estate rail recovery owner.
use super::super::protocol::{
    BankHttpDenialKind, BankHttpRecoveryInspectionOutcome, BankHttpRecoveryRequest,
    BankHttpRecoverySafeRetryOutcome,
};
use super::mutation_executor::{BankHttpMutationContinuation, WorkerCommand};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource;

impl BankHttpMutationContinuation {
    /// Inspects the existing rail recovery under fresh credentials after listener loss.
    pub async fn inspect_recovery(
        &self,
        request: BankHttpRecoveryRequest,
    ) -> BankHttpRecoveryInspectionOutcome {
        let request = match super::recovery_routes::admit_recovery(
            request,
            self.transport.maximum_deadline,
        ) {
            Ok(request) => request,
            Err((request_id, denial)) => {
                return BankHttpRecoveryInspectionOutcome::Denied { request_id, denial }
            }
        };
        let id = request.request_id.clone();
        let deadline = request.deadline;
        let cancellation = WorthQueryCancellationSource::new();
        let cancellation_guard = CancelOnDrop(cancellation.clone());
        let slot = match Arc::clone(&self.transport.queue_slots).try_acquire_owned() {
            Ok(slot) => slot,
            Err(_) => return inspection_denied(id, BankHttpDenialKind::Saturated),
        };
        let index =
            super::mutation_executor::routing_slot(&request.token, self.transport.senders.len());
        let (response, receiver) = oneshot::channel();
        let response_wait = tokio::time::timeout_at(deadline.into(), receiver);
        tokio::pin!(response_wait);
        // Cancellation must precede disposal of the response future in every edition.
        let _cancel = cancellation_guard;
        if let Err(error) = self.transport.senders[index].try_send(WorkerCommand::RecoveryInspect {
            request,
            queued_slot: slot,
            cancellation,
            response,
        }) {
            return inspection_denied(id, unavailable_or_full(error));
        }
        match response_wait.await {
            Ok(Ok(outcome)) => outcome,
            _ => inspection_denied(id, BankHttpDenialKind::Unavailable),
        }
    }
    /// Retries the separate native rail obligation; never reruns a SourceOnly handler.
    pub async fn retry_recovery(
        &self,
        request: BankHttpRecoveryRequest,
    ) -> BankHttpRecoverySafeRetryOutcome {
        let request = match super::recovery_routes::admit_recovery(
            request,
            self.transport.maximum_deadline,
        ) {
            Ok(request) => request,
            Err((request_id, denial)) => {
                return BankHttpRecoverySafeRetryOutcome::Denied { request_id, denial }
            }
        };
        let id = request.request_id.clone();
        let token = request.token.clone();
        let deadline = request.deadline;
        let cancellation = WorthQueryCancellationSource::new();
        let cancellation_guard = CancelOnDrop(cancellation.clone());
        let slot = match Arc::clone(&self.transport.queue_slots).try_acquire_owned() {
            Ok(slot) => slot,
            Err(_) => return retry_denied(id, BankHttpDenialKind::Saturated),
        };
        let index = super::mutation_executor::routing_slot(&token, self.transport.senders.len());
        let (response, receiver) = oneshot::channel();
        let response_wait = tokio::time::timeout_at(deadline.into(), receiver);
        tokio::pin!(response_wait);
        // Cancellation must precede disposal of the response future in every edition.
        let _cancel = cancellation_guard;
        if let Err(error) = self.transport.senders[index].try_send(WorkerCommand::RecoveryRetry {
            request,
            queued_slot: slot,
            cancellation,
            response,
        }) {
            return retry_denied(id, unavailable_or_full(error));
        }
        match response_wait.await {
            Ok(Ok(outcome)) => outcome,
            _ => BankHttpRecoverySafeRetryOutcome::OutcomeUnknown {
                request_id: id,
                recovery: token,
            },
        }
    }
}
fn unavailable_or_full(error: mpsc::error::TrySendError<WorkerCommand>) -> BankHttpDenialKind {
    match error {
        mpsc::error::TrySendError::Full(_) => BankHttpDenialKind::Saturated,
        mpsc::error::TrySendError::Closed(_) => BankHttpDenialKind::Unavailable,
    }
}
fn inspection_denied(id: String, kind: BankHttpDenialKind) -> BankHttpRecoveryInspectionOutcome {
    BankHttpRecoveryInspectionOutcome::Denied {
        request_id: Some(id),
        denial: super::mutation_refusal::denial(kind),
    }
}
fn retry_denied(id: String, kind: BankHttpDenialKind) -> BankHttpRecoverySafeRetryOutcome {
    BankHttpRecoverySafeRetryOutcome::Denied {
        request_id: Some(id),
        denial: super::mutation_refusal::denial(kind),
    }
}
struct CancelOnDrop(WorthQueryCancellationSource);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}
