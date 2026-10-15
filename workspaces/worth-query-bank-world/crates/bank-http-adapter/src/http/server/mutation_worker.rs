//! A worker serializes custody commands and stores native returns before transport replies.
use super::super::protocol::*;
use super::{
    authenticated_owner::BankHttpAuthenticatedOwner,
    authentication::BankHttpApplicationAuthenticator,
    mutation_command::{reject, AdmittedMoneyCommand, MoneyCommand},
    mutation_custody::*,
    mutation_executor::WorkerCommand,
    mutation_refusal::denial,
};
use bank_server::{BankMutationControls, BankReadControls};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::{mpsc, Semaphore};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

pub(super) async fn run<A: BankHttpApplicationAuthenticator>(
    application: Arc<A>,
    mut receiver: mpsc::Receiver<WorkerCommand>,
    permits: Arc<Semaphore>,
    quiesced: Arc<AtomicBool>,
    recovery: Arc<tokio::sync::Mutex<super::recovery_registry::BankHttpRecoveryRegistry>>,
) {
    let mut records = HashMap::new();
    while let Some(command) = receiver.recv().await {
        match command {
            WorkerCommand::Execute {
                command,
                queued_slot: _queued_slot,
                cancellation,
                response,
            } => {
                let scope =
                    WorthQueryRequestScope::new(Some(command.deadline), cancellation.token());
                let outcome = super::mutation_advancement::execute(
                    application.as_ref(),
                    &mut records,
                    &permits,
                    &quiesced,
                    command,
                    scope,
                )
                .await;
                // All native returned custody is already back in records before a reply can be lost.
                let _ = response.send(outcome);
            }
            WorkerCommand::Native {
                id,
                queued_slot: _queued_slot,
                job,
            } => {
                if quiesced.load(Ordering::Acquire) || records.contains_key(&id) {
                    job.reject(BankHttpDenialKind::Stale);
                    continue;
                }
                let permit = match Arc::clone(&permits).try_acquire_owned() {
                    Ok(permit) => permit,
                    Err(_) => {
                        job.reject(BankHttpDenialKind::Saturated);
                        continue;
                    }
                };
                let returned = job.run().await;
                let mut permit = Some(permit);
                if let Some((owner, custody)) = returned.cleanup {
                    records.insert(
                        id.clone(),
                        MutationRecord::Native(
                            super::native_cleanup_custody::NativeCleanupRecord {
                                owner,
                                sequence: 1,
                                custody:
                                    super::native_cleanup_custody::NativeCleanupCustody::Pending(
                                        custody,
                                    ),
                                permit: permit.take().expect("one returned owner"),
                            },
                        ),
                    );
                }
                if let Some(record) = returned.estate {
                    records.insert(
                        id,
                        MutationRecord::Estate(record, permit.take().expect("one returned owner")),
                    );
                }
                (returned.reply)();
            }
            WorkerCommand::Handoff {
                return_sender,
                command,
                queued_slot: _queued_slot,
                cancellation,
                response,
            } => {
                super::native_cleanup_handoff::execute(
                    Arc::clone(&application),
                    &mut records,
                    command,
                    cancellation,
                    response,
                    return_sender,
                )
                .await;
            }
            WorkerCommand::ConfirmHandoff { id } => {
                let Some(MutationRecord::Handoff(_)) = records.remove(&id) else {
                    unreachable!("reserved host acknowledgement")
                };
            }
            WorkerCommand::ReturnHandoff {
                id,
                owner,
                sequence,
                evidence,
            } => {
                let Some(MutationRecord::Handoff(reservation)) = records.remove(&id) else {
                    unreachable!("reserved custody return")
                };
                records.insert(
                    id,
                    MutationRecord::Native(super::native_cleanup_custody::NativeCleanupRecord {
                        owner,
                        sequence,
                        custody: super::native_cleanup_custody::NativeCleanupCustody::Released(
                            evidence,
                        ),
                        permit: reservation.permit,
                    }),
                );
            }
            WorkerCommand::RecoveryInspect {
                request,
                queued_slot: _slot,
                cancellation,
                response,
            } => {
                let mut registry = recovery.lock().await;
                registry.purge_expired(application.runtime());
                let outcome = super::recovery_executor::inspection::execute_inspection(
                    application.as_ref(),
                    &mut registry,
                    request,
                    cancellation,
                )
                .await;
                let _ = response.send(outcome);
            }
            WorkerCommand::RecoveryRetry {
                request,
                queued_slot: _slot,
                cancellation,
                response,
            } => {
                let mut registry = recovery.lock().await;
                registry.purge_expired(application.runtime());
                let outcome = super::recovery_executor::safe_retry::execute_safe_retry(
                    application.as_ref(),
                    &mut registry,
                    request,
                    cancellation,
                )
                .await;
                let _ = response.send(outcome);
            }
            WorkerCommand::Inspect(response) => {
                let _ = response.send(records.len());
            }
            WorkerCommand::Stop => break,
        }
    }
    // Dropping ordinary descriptors does not settle or dispose native records. No Drop I/O.
}
