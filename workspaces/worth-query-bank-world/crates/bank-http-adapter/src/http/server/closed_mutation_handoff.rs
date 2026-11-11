//! Fresh authenticated local handoff transfers a closed owner without losing native custody.
use super::super::protocol::{
    BankHttpDenialKind, BankHttpMutationOutcome, BankHttpMutationRequest,
};
use super::{
    closed_mutation_delivery::BankHttpClosedMutationDelivery,
    mutation_command::{admit, reject, AdmittedMutationCommand, MutationCommand},
    mutation_custody::MutationRecord,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::oneshot;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

impl super::mutation_executor::BankHttpMutationContinuation {
    /// Moves the whole closed source/capability/estate owner from its original worker.
    /// Uses a current Close command; this local return is not a wire acknowledgement or commit.
    pub async fn prepare_closed_mutation_handoff(
        &self,
        request: BankHttpMutationRequest,
    ) -> Result<BankHttpClosedMutationDelivery, Box<BankHttpMutationOutcome>> {
        let command = admit(request, self.transport.maximum_deadline)?;
        if !matches!(command.command, MutationCommand::Close) {
            return Err(Box::new(reject(
                Some(command.request_id),
                BankHttpDenialKind::MalformedRequest,
            )));
        }
        let request_id = command.request_id.clone();
        let custody_id = command.custody_id.clone();
        let deadline = command.deadline;
        let cancellation = WorthQueryCancellationSource::new();
        let cancellation_guard = CancelOnDrop(cancellation.clone());
        let slot = Arc::clone(&self.transport.queue_slots)
            .try_acquire_owned()
            .map_err(|_| reject(Some(request_id.clone()), BankHttpDenialKind::Saturated))?;
        let (response, receiver) = oneshot::channel();
        let response_wait = tokio::time::timeout_at(deadline.into(), receiver);
        tokio::pin!(response_wait);
        // Cancellation must precede disposal of the response future in every edition.
        let _cancel = cancellation_guard;
        let index =
            super::mutation_executor::routing_slot(&custody_id, self.transport.senders.len());
        if let Err(error) = self.transport.senders[index].try_send(
            super::mutation_executor::WorkerCommand::ClosedHandoff {
                return_sender: self.transport.senders[index].clone(),
                command,
                queued_slot: slot,
                cancellation,
                response,
            },
        ) {
            let kind = match error {
                tokio::sync::mpsc::error::TrySendError::Full(_) => BankHttpDenialKind::Saturated,
                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    BankHttpDenialKind::Unavailable
                }
            };
            return Err(Box::new(reject(Some(request_id), kind)));
        }
        match response_wait.await {
            Ok(Ok(result)) => result.map_err(Box::new),
            _ => Err(Box::new(BankHttpMutationOutcome::OutcomeUnknown {
                request_id,
                custody_id,
            })),
        }
    }
}
pub(super) async fn execute<A: super::authentication::BankHttpApplicationAuthenticator>(
    application: Arc<A>,
    records: &mut HashMap<String, MutationRecord>,
    command: AdmittedMutationCommand,
    cancellation: WorthQueryCancellationSource,
    response: oneshot::Sender<
        Result<
            super::closed_mutation_delivery::BankHttpClosedMutationDelivery,
            BankHttpMutationOutcome,
        >,
    >,
    return_sender: tokio::sync::mpsc::Sender<super::mutation_executor::WorkerCommand>,
) {
    let scope = WorthQueryRequestScope::new(Some(command.deadline), cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let principal = match application.authenticate(command.credential, &scope).await {
        Ok(principal) => principal,
        Err(denial) => {
            let _ = response.send(Err(BankHttpMutationOutcome::Rejected {
                request_id: Some(command.request_id),
                denial,
            }));
            return;
        }
    };
    let owner = super::authenticated_owner::BankHttpAuthenticatedOwner::from_principal(&principal);
    let eligible = records.get(&command.custody_id).is_some_and(|record| {
        matches!(record, MutationRecord::Closed(record) if super::closed_mutation_delivery::eligible(record, &owner, command.sequence))
    });
    if !eligible {
        let _ = response.send(Err(reject(
            Some(command.request_id),
            BankHttpDenialKind::Stale,
        )));
        return;
    }
    let admitted = application
        .runtime()
        .with_application_advancement(&scope, |_| ());
    if let Err(cause) = admitted {
        let _ = response.send(Err(BankHttpMutationOutcome::Rejected {
            request_id: Some(command.request_id),
            denial: super::advancement_denial::advancement(cause),
        }));
        return;
    }
    let return_slot = match return_sender.try_reserve_owned() {
        Ok(slot) => slot,
        Err(error) => {
            let kind = match error {
                tokio::sync::mpsc::error::TrySendError::Full(_) => BankHttpDenialKind::Saturated,
                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    BankHttpDenialKind::Unavailable
                }
            };
            let _ = response.send(Err(reject(Some(command.request_id), kind)));
            return;
        }
    };
    let Some(MutationRecord::Closed(record)) = records.remove(&command.custody_id) else {
        unreachable!("serialized closed handoff")
    };
    records.insert(command.custody_id.clone(), MutationRecord::ClosedHandoff);
    let delivery = super::closed_mutation_delivery::BankHttpClosedMutationDelivery::new(
        command.custody_id,
        record,
        application,
        return_slot,
    );
    // A failed send or receiver cancellation drops the guard, which returns the exact owner.
    let _ = response.send(Ok(delivery));
}
struct CancelOnDrop(WorthQueryCancellationSource);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}
