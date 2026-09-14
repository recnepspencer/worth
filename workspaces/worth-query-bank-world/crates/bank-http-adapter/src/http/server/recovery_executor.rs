use std::sync::Arc;
use std::time::Instant;

use bank_domain::estate::EstateAction;
use bank_domain::proposals::BankIdempotencyKey;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource;

use super::super::protocol::{
    BankHttpEstateDisbursementOutcome, BankHttpEstateNotificationOutcome,
    BankHttpRecoveryInspectionOutcome,
};
use super::authentication::BankHttpApplicationAuthenticator;
use super::recovery_registry::BankHttpRecoveryRegistry;

mod disbursement;
mod inspection;
mod notification;
mod outcome;

use disbursement::execute_disbursement;
use inspection::execute_inspection;
use notification::execute_notification;
use outcome::*;

pub(super) struct AdmittedBankHttpRecoveryRequest {
    pub(super) request_id: String,
    pub(super) credential: super::super::protocol::BankHttpCredential,
    pub(super) token: String,
    pub(super) deadline: Instant,
}

pub(super) struct AdmittedBankHttpNotificationRequest {
    pub(super) request_id: String,
    pub(super) credential: super::super::protocol::BankHttpCredential,
    pub(super) idempotency_key: BankIdempotencyKey,
    pub(super) action: EstateAction,
    pub(super) deadline: Instant,
}

pub(super) struct AdmittedBankHttpDisbursementRequest {
    pub(super) request_id: String,
    pub(super) credential: super::super::protocol::BankHttpCredential,
    pub(super) idempotency_key: BankIdempotencyKey,
    pub(super) action: EstateAction,
    pub(super) deadline: Instant,
}

#[derive(Clone)]
pub(super) struct BankHttpRecoveryExecutor {
    sender: mpsc::Sender<RecoveryCommand>,
}

enum RecoveryCommand {
    Notify {
        request: AdmittedBankHttpNotificationRequest,
        cancellation: WorthQueryCancellationSource,
        response: oneshot::Sender<BankHttpEstateNotificationOutcome>,
    },
    Inspect {
        request: AdmittedBankHttpRecoveryRequest,
        cancellation: WorthQueryCancellationSource,
        response: oneshot::Sender<BankHttpRecoveryInspectionOutcome>,
    },
    Disburse {
        request: AdmittedBankHttpDisbursementRequest,
        cancellation: WorthQueryCancellationSource,
        response: oneshot::Sender<BankHttpEstateDisbursementOutcome>,
    },
}

impl BankHttpRecoveryExecutor {
    pub(super) fn start<A>(
        application: Arc<A>,
        queue_capacity: usize,
        registry_capacity: usize,
        lifetime: std::time::Duration,
    ) -> (Self, JoinHandle<()>)
    where
        A: BankHttpApplicationAuthenticator,
    {
        let (sender, receiver) = mpsc::channel(queue_capacity);
        let task = tokio::spawn(run(
            application,
            receiver,
            BankHttpRecoveryRegistry::new(registry_capacity, lifetime),
        ));
        (Self { sender }, task)
    }

    pub(super) async fn notify(
        &self,
        request: AdmittedBankHttpNotificationRequest,
    ) -> BankHttpEstateNotificationOutcome {
        let request_id = request.request_id.clone();
        let deadline = request.deadline;
        let cancellation = WorthQueryCancellationSource::new();
        let _cancel_on_drop = CancelOnDrop(cancellation.clone());
        let (response, receiver) = oneshot::channel();
        if self
            .sender
            .try_send(RecoveryCommand::Notify {
                request,
                cancellation,
                response,
            })
            .is_err()
        {
            return notification_denied(Some(request_id), saturated());
        }
        match tokio::time::timeout_at(deadline.into(), receiver).await {
            Ok(Ok(outcome)) => outcome,
            Ok(Err(_)) => notification_denied(Some(request_id), unavailable()),
            Err(_) => notification_denied(Some(request_id), deadline_exceeded()),
        }
    }

    pub(super) async fn inspect(
        &self,
        request: AdmittedBankHttpRecoveryRequest,
    ) -> BankHttpRecoveryInspectionOutcome {
        let request_id = request.request_id.clone();
        let deadline = request.deadline;
        let cancellation = WorthQueryCancellationSource::new();
        let _cancel_on_drop = CancelOnDrop(cancellation.clone());
        let (response, receiver) = oneshot::channel();
        if self
            .sender
            .try_send(RecoveryCommand::Inspect {
                request,
                cancellation,
                response,
            })
            .is_err()
        {
            return inspection_denied(Some(request_id), saturated());
        }
        match tokio::time::timeout_at(deadline.into(), receiver).await {
            Ok(Ok(outcome)) => outcome,
            Ok(Err(_)) => inspection_denied(Some(request_id), unavailable()),
            Err(_) => inspection_denied(Some(request_id), deadline_exceeded()),
        }
    }

    pub(super) async fn disburse(
        &self,
        request: AdmittedBankHttpDisbursementRequest,
    ) -> BankHttpEstateDisbursementOutcome {
        let request_id = request.request_id.clone();
        let deadline = request.deadline;
        let cancellation = WorthQueryCancellationSource::new();
        let _cancel_on_drop = CancelOnDrop(cancellation.clone());
        let (response, receiver) = oneshot::channel();
        if self
            .sender
            .try_send(RecoveryCommand::Disburse {
                request,
                cancellation,
                response,
            })
            .is_err()
        {
            return disbursement_denied(Some(request_id), saturated());
        }
        match tokio::time::timeout_at(deadline.into(), receiver).await {
            Ok(Ok(outcome)) => outcome,
            Ok(Err(_)) => disbursement_denied(Some(request_id), unavailable()),
            Err(_) => disbursement_denied(Some(request_id), deadline_exceeded()),
        }
    }
}

struct CancelOnDrop(WorthQueryCancellationSource);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

async fn run<A>(
    application: Arc<A>,
    mut receiver: mpsc::Receiver<RecoveryCommand>,
    mut registry: BankHttpRecoveryRegistry,
) where
    A: BankHttpApplicationAuthenticator,
{
    while let Some(command) = receiver.recv().await {
        match command {
            RecoveryCommand::Notify {
                request,
                cancellation,
                response,
            } => {
                let _ = response.send(
                    execute_notification(
                        application.as_ref(),
                        &mut registry,
                        request,
                        cancellation,
                    )
                    .await,
                );
            }
            RecoveryCommand::Inspect {
                request,
                cancellation,
                response,
            } => {
                let _ = response.send(
                    execute_inspection(application.as_ref(), &mut registry, request, cancellation)
                        .await,
                );
            }
            RecoveryCommand::Disburse {
                request,
                cancellation,
                response,
            } => {
                let _ = response.send(
                    execute_disbursement(
                        application.as_ref(),
                        &mut registry,
                        request,
                        cancellation,
                    )
                    .await,
                );
            }
        }
    }
}
