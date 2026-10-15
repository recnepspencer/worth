//! Local host handoff moves actual released evidence; a lost reply restores custody.
use super::super::protocol::{
    BankHttpDenialKind, BankHttpMutationOutcome, BankHttpMutationRequest,
};
use super::{
    mutation_command::{admit, reject, AdmittedMoneyCommand, MoneyCommand},
    mutation_custody::MutationRecord,
    native_cleanup_custody::*,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::oneshot;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

impl super::mutation_executor::BankHttpMutationContinuation {
    /// Moves original business evidence only after physical release, on its original worker.
    /// Uses a current Close command; this local return is not a wire acknowledgement or commit.
    pub async fn take_released_cleanup(
        &self,
        request: BankHttpMutationRequest,
    ) -> Result<BankHttpReleasedCleanup, BankHttpMutationOutcome> {
        let command = admit(request, self.transport.maximum_deadline)?;
        if !matches!(command.command, MoneyCommand::Close) {
            return Err(reject(
                Some(command.request_id),
                BankHttpDenialKind::MalformedRequest,
            ));
        }
        let request_id = command.request_id.clone();
        let custody_id = command.custody_id.clone();
        let deadline = command.deadline;
        let cancellation = WorthQueryCancellationSource::new();
        let _cancel = CancelOnDrop(cancellation.clone());
        let slot = Arc::clone(&self.transport.queue_slots)
            .try_acquire_owned()
            .map_err(|_| reject(Some(request_id.clone()), BankHttpDenialKind::Saturated))?;
        let (response, receiver) = oneshot::channel();
        let index =
            super::mutation_executor::routing_slot(&custody_id, self.transport.senders.len());
        if let Err(error) = self.transport.senders[index].try_send(
            super::mutation_executor::WorkerCommand::Handoff {
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
            return Err(reject(Some(request_id), kind));
        }
        match tokio::time::timeout_at(deadline.into(), receiver).await {
            Ok(Ok(result)) => result.map(super::native_cleanup_delivery::CleanupDelivery::receive),
            _ => Err(BankHttpMutationOutcome::OutcomeUnknown {
                request_id,
                custody_id,
            }),
        }
    }
}
pub(super) async fn execute<A: super::authentication::BankHttpApplicationAuthenticator>(
    application: Arc<A>,
    records: &mut HashMap<String, MutationRecord>,
    command: AdmittedMoneyCommand,
    cancellation: WorthQueryCancellationSource,
    response: oneshot::Sender<
        Result<super::native_cleanup_delivery::CleanupDelivery, BankHttpMutationOutcome>,
    >,
    return_sender: tokio::sync::mpsc::Sender<super::mutation_executor::WorkerCommand>,
) {
    let scope = WorthQueryRequestScope::new(Some(command.deadline), cancellation.token());
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
    let eligible = matches!(records.get(&command.custody_id), Some(MutationRecord::Native(record))
        if record.owner == owner && record.sequence.checked_add(1) == Some(command.sequence)
        && matches!(record.custody, NativeCleanupCustody::Released(_)));
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
    let Some(MutationRecord::Native(record)) = records.remove(&command.custody_id) else {
        unreachable!("serialized handoff")
    };
    let NativeCleanupCustody::Released(evidence) = record.custody else {
        unreachable!("eligible release")
    };
    records.insert(
        command.custody_id.clone(),
        MutationRecord::Handoff(HandoffRecord {
            permit: record.permit,
        }),
    );
    let delivery = super::native_cleanup_delivery::CleanupDelivery::new(
        command.custody_id,
        record.owner,
        record.sequence,
        evidence,
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
