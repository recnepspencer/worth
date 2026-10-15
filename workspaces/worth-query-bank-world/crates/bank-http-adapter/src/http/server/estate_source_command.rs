//! Source custody advances natively; external rail recovery receives only its actual closed witness.
use super::super::protocol::{BankHttpDenial, BankHttpDenialKind, BankHttpMutationOutcome};
use super::{
    estate_source_custody::EstateSourceCustody,
    mutation_command::{AdmittedMoneyCommand, MoneyCommand},
    mutation_custody::MutationRecord,
    mutation_refusal::denial,
};
use std::collections::HashMap;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

pub(super) async fn execute<A: super::authentication::BankHttpApplicationAuthenticator>(
    application: &A,
    records: &mut HashMap<String, MutationRecord>,
    command: AdmittedMoneyCommand,
    scope: WorthQueryRequestScope,
    principal: &bank_server::BankAuthenticatedPrincipal,
    owner: super::authenticated_owner::BankHttpAuthenticatedOwner,
) -> BankHttpMutationOutcome {
    let Some(MutationRecord::Estate(record, _)) = records.get(&command.custody_id) else {
        unreachable!("source dispatch")
    };
    if record.owner != owner {
        return refused(&command, denial(BankHttpDenialKind::PermissionDenied), None);
    }
    if record.sequence.checked_add(1) != Some(command.sequence) {
        return refused(
            &command,
            denial(BankHttpDenialKind::Stale),
            Some(record.sequence),
        );
    }
    let Some(MutationRecord::Estate(mut record, permit)) = records.remove(&command.custody_id)
    else {
        unreachable!("serialized source")
    };
    record.sequence = command.sequence;
    let request = application.runtime().request(principal, &scope);
    let outcome = match command.command {
        MoneyCommand::ResumeSource => {
            let (custody, cause) = record.custody.advance(&request);
            record.custody = custody;
            match cause {
                Some(cause) => refused(&command, cause, Some(command.sequence)),
                None => record.custody.describe(
                    &command.custody_id,
                    &command.request_id,
                    command.sequence,
                ),
            }
        }
        MoneyCommand::ReleaseCleanup => match record.custody.release(&request) {
            Ok(receipt) => BankHttpMutationOutcome::CollectionReleased {
                request_id: command.request_id.clone(),
                custody_id: command.custody_id.clone(),
                sequence: command.sequence,
                retired_component_count: receipt.retired_component_count(),
                managed: receipt
                    .managed_cleanup_receipt()
                    .map(super::cleanup_description::describe),
            },
            Err(cause) => refused(&command, cause, Some(command.sequence)),
        },
        MoneyCommand::Close => {
            let mut registry = record.registry.lock().await;
            registry.purge_expired(application.runtime());
            let replay = match record.action {
                bank_domain::estate::EstateAction::NotifyDeath { .. } => {
                    registry.notification_replay(&record.owner, &record.key, record.action)
                }
                bank_domain::estate::EstateAction::DisburseEstate(_) => {
                    registry.disbursement_replay(&record.owner, &record.key, record.action)
                }
                _ => unreachable!("source action declaration"),
            };
            if let super::recovery_registry::BankHttpCommitReplay::Applied {
                recovery,
                completed,
                ..
            } = replay
            {
                match record.custody.close(&request) {
                    Ok(Some(receipt)) => {
                        return BankHttpMutationOutcome::RailRecoveryReady {
                            request_id: command.request_id,
                            custody_id: command.custody_id,
                            sequence: command.sequence,
                            commit: super::recovery_executor::outcome::commit_description(&receipt),
                            recovery: Some(recovery),
                            recovery_status: if completed {
                                super::super::protocol::BankHttpRecoveryStatus::Completed
                            } else {
                                super::super::protocol::BankHttpRecoveryStatus::TokenIssued
                            },
                        }
                    }
                    Ok(None) => {
                        return BankHttpMutationOutcome::Closed {
                            request_id: command.request_id,
                            custody_id: command.custody_id,
                            sequence: command.sequence,
                        }
                    }
                    Err((custody, cause)) => {
                        record.custody = custody;
                        drop(registry);
                        let outcome = refused(&command, cause, Some(command.sequence));
                        records.insert(command.custody_id, MutationRecord::Estate(record, permit));
                        return outcome;
                    }
                }
            }
            let reservation = match record.action {
                bank_domain::estate::EstateAction::NotifyDeath { .. } => registry
                    .reserve_notification(record.owner.clone(), record.key.clone(), record.action),
                bank_domain::estate::EstateAction::DisburseEstate(_) => registry
                    .reserve_disbursement(record.owner.clone(), record.key.clone(), record.action),
                _ => unreachable!("source action declaration"),
            };
            match reservation {
                None => match record.custody.close(&request) {
                    Err((custody, cause)) => {
                        record.custody = custody;
                        refused(&command, cause, Some(command.sequence))
                    }
                    Ok(None) => {
                        return BankHttpMutationOutcome::Closed {
                            request_id: command.request_id,
                            custody_id: command.custody_id,
                            sequence: command.sequence,
                        };
                    }
                    Ok(Some(receipt)) => {
                        // A full host registry cannot change native claim truth.
                        // An unclaimed commit still needs a slot before minting a handle.
                        let status = application.runtime().commit_recovery_claim_status(&receipt);
                        match status {
                            Ok(status)
                                if status != bank_server::BankRecoveryClaimStatus::Unclaimed =>
                            {
                                return BankHttpMutationOutcome::RailRecoveryReady {
                                    request_id: command.request_id,
                                    custody_id: command.custody_id,
                                    sequence: command.sequence,
                                    commit: super::recovery_executor::outcome::commit_description(
                                        &receipt,
                                    ),
                                    recovery: None,
                                    recovery_status: if status
                                        == bank_server::BankRecoveryClaimStatus::Completed
                                    {
                                        super::super::protocol::BankHttpRecoveryStatus::Completed
                                    } else {
                                        super::super::protocol::BankHttpRecoveryStatus::OperatorRequired
                                    },
                                };
                            }
                            status => {
                                record.custody =
                                    EstateSourceCustody::retain_closed(receipt, record.action);
                                refused(
                                    &command,
                                    denial(if status.is_ok() {
                                        BankHttpDenialKind::Saturated
                                    } else {
                                        BankHttpDenialKind::Unavailable
                                    }),
                                    Some(command.sequence),
                                )
                            }
                        }
                    }
                },
                Some(reservation) => match record.custody.close(&request) {
                    Err((custody, cause)) => {
                        record.custody = custody;
                        refused(&command, cause, Some(command.sequence))
                    }
                    Ok(None) => {
                        return BankHttpMutationOutcome::Closed {
                            request_id: command.request_id,
                            custody_id: command.custody_id,
                            sequence: command.sequence,
                        }
                    }
                    Ok(Some(receipt)) => match application.runtime().open_commit_recovery(&receipt)
                    {
                        Ok(handle) => {
                            let commit =
                                super::recovery_executor::outcome::commit_description(&receipt);
                            let recovery = reservation.register(
                                super::recovery_registry::BankHttpRecoveryRegistration {
                                    commit,
                                    handle,
                                },
                            );
                            return BankHttpMutationOutcome::RailRecoveryReady {
                                request_id: command.request_id,
                                custody_id: command.custody_id,
                                sequence: command.sequence,
                                commit,
                                recovery: Some(recovery),
                                recovery_status:
                                    super::super::protocol::BankHttpRecoveryStatus::TokenIssued,
                            };
                        }
                        Err(cause)
                            if matches!(
                                cause.kind(),
                                bank_server::BankRecoveryDenialKind::RecoveryAlreadyMinted
                                    | bank_server::BankRecoveryDenialKind::AlreadyTerminal
                            ) =>
                        {
                            match super::recovery_executor::outcome::omitted_recovery_status(
                                application.runtime(),
                                &receipt,
                            ) {
                                Ok(recovery_status) => {
                                    return BankHttpMutationOutcome::RailRecoveryReady {
                                        request_id: command.request_id,
                                        custody_id: command.custody_id,
                                        sequence: command.sequence,
                                        commit:
                                            super::recovery_executor::outcome::commit_description(
                                                &receipt,
                                            ),
                                        recovery: None,
                                        recovery_status,
                                    }
                                }
                                Err(_) => {
                                    record.custody =
                                        EstateSourceCustody::retain_closed(receipt, record.action);
                                    refused(
                                        &command,
                                        denial(BankHttpDenialKind::Unavailable),
                                        Some(command.sequence),
                                    )
                                }
                            }
                        }
                        Err(_) => {
                            record.custody =
                                EstateSourceCustody::retain_closed(receipt, record.action);
                            refused(
                                &command,
                                denial(BankHttpDenialKind::Unavailable),
                                Some(command.sequence),
                            )
                        }
                    },
                },
            }
        }
        _ => refused(
            &command,
            denial(BankHttpDenialKind::MalformedRequest),
            Some(command.sequence),
        ),
    };
    records.insert(command.custody_id, MutationRecord::Estate(record, permit));
    outcome
}
fn refused(
    command: &AdmittedMoneyCommand,
    cause: BankHttpDenial,
    accepted_sequence: Option<u64>,
) -> BankHttpMutationOutcome {
    BankHttpMutationOutcome::Refused {
        request_id: command.request_id.clone(),
        custody_id: command.custody_id.clone(),
        sequence: command.sequence,
        denial: cause,
        accepted_sequence,
    }
}
