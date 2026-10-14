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
                let outcome = execute(
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
            WorkerCommand::Inspect(response) => {
                let _ = response.send(records.len());
            }
            WorkerCommand::Stop => break,
        }
    }
    // Dropping ordinary descriptors does not settle or dispose native records. No Drop I/O.
}
async fn execute<A: BankHttpApplicationAuthenticator>(
    application: &A,
    records: &mut HashMap<String, MutationRecord>,
    permits: &Arc<Semaphore>,
    quiesced: &AtomicBool,
    command: AdmittedMoneyCommand,
    scope: WorthQueryRequestScope,
) -> BankHttpMutationOutcome {
    let principal = match application
        .authenticate(command.credential.clone(), &scope)
        .await
    {
        Ok(principal) => principal,
        Err(denial) => {
            return BankHttpMutationOutcome::Rejected {
                request_id: Some(command.request_id),
                denial,
            }
        }
    };
    let owner = BankHttpAuthenticatedOwner::from_principal(&principal);
    let runtime = application.runtime();
    let request = runtime.request(&principal, &scope);
    if let MoneyCommand::Create {
        input,
        key,
        history,
    } = command.command
    {
        if quiesced.load(Ordering::Acquire) {
            return reject(Some(command.request_id), BankHttpDenialKind::Unavailable);
        }
        if command.sequence != 1 || records.contains_key(&command.custody_id) {
            return reject(Some(command.request_id), BankHttpDenialKind::Stale);
        }
        let permit = match Arc::clone(permits).try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => return reject(Some(command.request_id), BankHttpDenialKind::Saturated),
        };
        let (account, custody) = match super::mutation_construction::construct(
            runtime,
            &principal,
            input,
            BankMutationControls::new(scope.clone(), key),
        ) {
            Ok(constructed) => constructed,
            Err(denial) => {
                return BankHttpMutationOutcome::Rejected {
                    request_id: Some(command.request_id),
                    denial,
                }
            }
        };
        let (custody, refusal) = custody.advance(&request, history);
        let record = MutationRecord {
            owner,
            sequence: command.sequence,
            account,
            custody,
            _permit: permit,
        };
        let outcome = describe(&command.request_id, &command.custody_id, &record, refusal);
        records.insert(command.custody_id, record);
        return outcome;
    }
    let Some(record) = records.get(&command.custody_id) else {
        return reject(Some(command.request_id), BankHttpDenialKind::NotFound);
    };
    if record.owner != owner {
        return refusal_before_admission(
            &command,
            denial(BankHttpDenialKind::PermissionDenied),
            None,
        );
    }
    // A duplicate/stale sequence acknowledges no further work and never replays protected rows.
    if record.sequence.checked_add(1) != Some(command.sequence) {
        return refusal_before_admission(
            &command,
            denial(BankHttpDenialKind::Stale),
            Some(record.sequence),
        );
    }
    let mut record = records
        .remove(&command.custody_id)
        .expect("serialized worker checked custody");
    record.sequence = command.sequence;
    let outcome = match command.command {
        MoneyCommand::Resume { history } => {
            let (custody, refusal) = record.custody.advance(&request, history);
            record.custody = custody;
            describe(&command.request_id, &command.custody_id, &record, refusal)
        }
        MoneyCommand::Read {
            maximum_results,
            maximum_work,
        } => {
            let outcome = retained_read(
                runtime,
                &principal,
                &scope,
                &command,
                &record,
                maximum_results,
                maximum_work,
            );
            outcome
        }
        MoneyCommand::Close => match record.custody.close(&request) {
            Ok(()) => {
                return BankHttpMutationOutcome::Closed {
                    request_id: command.request_id,
                    custody_id: command.custody_id,
                    sequence: command.sequence,
                }
            }
            Err((custody, denial)) => {
                record.custody = custody;
                refused(&command, denial)
            }
        },
        MoneyCommand::Create { .. } => unreachable!("create returned before record lookup"),
    };
    records.insert(command.custody_id, record);
    outcome
}
fn describe(
    request_id: &str,
    custody_id: &str,
    record: &MutationRecord,
    refusal: Option<BankHttpDenial>,
) -> BankHttpMutationOutcome {
    let request_id = request_id.to_owned();
    let custody_id = custody_id.to_owned();
    let sequence = record.sequence;
    if let Some(denial) = refusal {
        return BankHttpMutationOutcome::Refused {
            request_id,
            custody_id,
            sequence,
            denial,
            accepted_sequence: Some(sequence),
        };
    }
    match record.custody.description() {
        CustodyDescription::Pending(posture) => BankHttpMutationOutcome::Pending {
            request_id,
            custody_id,
            sequence,
            posture,
        },
        CustodyDescription::Completed { recovered, commit } => BankHttpMutationOutcome::Completed {
            request_id,
            custody_id,
            sequence,
            recovered,
            commit,
        },
        CustodyDescription::Terminal => BankHttpMutationOutcome::Terminal {
            request_id,
            custody_id,
            sequence,
        },
    }
}
fn refused(command: &AdmittedMoneyCommand, denial: BankHttpDenial) -> BankHttpMutationOutcome {
    BankHttpMutationOutcome::Refused {
        request_id: command.request_id.clone(),
        custody_id: command.custody_id.clone(),
        sequence: command.sequence,
        denial,
        accepted_sequence: Some(command.sequence),
    }
}
fn refusal_before_admission(
    command: &AdmittedMoneyCommand,
    denial: BankHttpDenial,
    accepted_sequence: Option<u64>,
) -> BankHttpMutationOutcome {
    BankHttpMutationOutcome::Refused {
        request_id: command.request_id.clone(),
        custody_id: command.custody_id.clone(),
        sequence: command.sequence,
        denial,
        accepted_sequence,
    }
}
fn retained_read(
    runtime: &bank_server::BankIdentityRuntime,
    principal: &bank_server::BankAuthenticatedPrincipal,
    scope: &WorthQueryRequestScope,
    command: &AdmittedMoneyCommand,
    record: &MutationRecord,
    maximum_results: usize,
    maximum_work: usize,
) -> BankHttpMutationOutcome {
    let Some(observation) = record.custody.observation() else {
        return refused(command, denial(BankHttpDenialKind::Unavailable));
    };
    let controls = match BankReadControls::current(scope.clone(), maximum_results, maximum_work) {
        Ok(controls) => controls,
        Err(_) => return refused(command, denial(BankHttpDenialKind::MalformedRequest)),
    };
    let published = match runtime
        .account_activity(record.account)
        .as_principal(principal)
        .retained(observation, controls)
    {
        Ok(published) => published,
        Err(cause) => return refused(command, super::query_denial::query_denial(cause)),
    };
    let Some(activity) = published.rows().first().map(BankHttpAccountActivity::from) else {
        return refused(command, denial(BankHttpDenialKind::InternalDenied));
    };
    let publication = super::query_publication::describe_query_publication(
        published.receipt(),
        BankHttpQueryCapabilityPurpose::AccountActivityReview,
    );
    BankHttpMutationOutcome::Delivered {
        request_id: command.request_id.clone(),
        custody_id: command.custody_id.clone(),
        sequence: command.sequence,
        activity,
        publication,
    }
}
