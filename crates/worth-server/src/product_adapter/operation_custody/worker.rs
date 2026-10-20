//! Fresh commands advance original native owners once, storing every return before replying.
use super::{
    contract::WorthServerQuerySourceAdapter as SourceCommand,
    custody::Record,
    protocol::{
        self, ControlRefusal, WorthServerSourceCommand as Command, WorthServerSourceReply as Reply,
    },
    transport::Job,
};
use crate::{
    WorthServerProductAdapterExecutionError as Error, WorthServerProductOperationDenial,
    WorthServerScheduledProductOperation,
};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
    },
};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;

pub(super) fn run<C: SourceCommand>(
    application: &C,
    receiver: &Receiver<Job>,
    capacity: usize,
    quiesced: &AtomicBool,
) -> WorkerExit {
    let mut records = BTreeMap::new();
    while let Ok(job) = receiver.recv() {
        match job {
            Job::Execute { operation, reply } => {
                let outcome = execute::<C>(
                    application,
                    &operation,
                    &mut records,
                    capacity,
                    quiesced.load(Ordering::Acquire),
                );
                // Operation/refusal custody is back in records before a response can be lost.
                let _ = reply.send(outcome.and_then(|value| protocol::publish::<C>(&value)));
            }
            Job::Finish { reply } => {
                let empty = records.is_empty();
                let _ = reply.send(empty);
                if empty {
                    return WorkerExit::ClosedEmpty;
                }
            }
        }
    }
    // Final host abandonment has no cleanup or durability guarantee. No pending
    // owner is treated as disposed or reconstructed from an identifier.
    WorkerExit::Abandoned
}
pub(super) enum WorkerExit {
    ClosedEmpty,
    Abandoned,
}
fn execute<C: SourceCommand>(
    application: &C,
    operation: &WorthServerScheduledProductOperation,
    records: &mut BTreeMap<String, Box<Record<C>>>,
    capacity: usize,
    quiesced: bool,
) -> Result<Reply<C::CompletionData>, Error> {
    let caller = operation
        .plan()
        .operation_admission()
        .operation_request()
        .resolved_request_context()
        .request_context()
        .authenticated_principal()
        .admitted_transport_caller()
        .ok_or_else(|| denied("source.transport_identity"))?;
    let scope = caller
        .query_request_scope()
        .ok_or_else(|| denied("source.request_scope"))?
        .request_scope();
    let principal = application.authenticate(caller, scope, operation.plan().payload())?;
    let command = protocol::decode::<C>(operation.plan().payload()).map_err(Error::denied)?;
    match command {
        Command::Create {
            custody_id,
            logical_key,
            intent,
        } => {
            let rejection = if quiesced {
                Some(ControlRefusal::Quiesced)
            } else if let Some(held) = records.get(&custody_id) {
                Some(if &held.owner == principal.identity() {
                    ControlRefusal::AlreadyHeld
                } else {
                    ControlRefusal::CustodyUnavailable
                })
            } else if let Some(held) = records
                .values()
                .find(|record| record.logical_key == logical_key)
            {
                Some(if &held.owner == principal.identity() {
                    ControlRefusal::LogicalKeyHeld
                } else {
                    ControlRefusal::CustodyUnavailable
                })
            } else if records.len() >= capacity {
                Some(ControlRefusal::Capacity)
            } else {
                None
            };
            if let Some(reason) = rejection {
                return Ok(Reply::NotExecuted { custody_id, reason });
            }
            let basis = operation
                .plan()
                .operation_admission()
                .operation_request()
                .identity()
                .basis_digest();
            let mut record =
                Record::<C>::new(principal.identity().clone(), logical_key, intent, basis).advance(
                    application,
                    &principal,
                    scope,
                );
            record.sequence = 1;
            let response = held(
                application,
                &custody_id,
                &record,
                &application.program().runtime().request(&principal, scope),
            );
            records.insert(custody_id, record);
            Ok(response)
        }
        command => Ok(control(application, &principal, scope, &command, records)),
    }
}
fn control<C: SourceCommand>(
    application: &C,
    principal: &WorthQueryAuthenticatedExternalPrincipal<C::Schema>,
    scope: &WorthQueryRequestScope,
    command: &Command<C::Input>,
    records: &mut BTreeMap<String, Box<Record<C>>>,
) -> Reply<C::CompletionData> {
    let id = command.custody_id().to_owned();
    let Some(record) = records
        .get(&id)
        .filter(|record| &record.owner == principal.identity())
    else {
        return Reply::NotExecuted {
            custody_id: id,
            reason: ControlRefusal::CustodyUnavailable,
        };
    };
    let sequence = match command {
        Command::Advance { sequence, .. } | Command::Close { sequence, .. } => *sequence,
        Command::Inspect { .. } => {
            let request = application.program().runtime().request(principal, scope);
            return held(application, &id, record, &request);
        }
        Command::Create { .. } => unreachable!("new command custody handled before control"),
    };
    if Some(sequence) != record.sequence.checked_add(1) {
        return Reply::NotExecuted {
            custody_id: id,
            reason: ControlRefusal::StaleSequence,
        };
    }
    let record = records
        .remove(&id)
        .expect("one worker retains the addressed record");
    let request = application.program().runtime().request(principal, scope);
    let mut returned = match command {
        Command::Advance { .. } => record.advance(application, principal, scope),
        Command::Close { .. } => match record.close(&request) {
            Ok(super::custody::Closure::NativeClosed) => {
                return Reply::Closed { custody_id: id };
            }
            Ok(super::custody::Closure::PreparationDiscarded) => {
                return Reply::PreparationDiscarded { custody_id: id };
            }
            Err(record) => record,
        },
        _ => unreachable!("only admitted advancement/close reaches native progression"),
    };
    returned.sequence = sequence;
    let response = held(application, &id, &returned, &request);
    records.insert(id, returned);
    response
}
fn held<C: SourceCommand>(
    application: &C,
    id: &str,
    record: &Record<C>,
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        '_,
        '_,
        '_,
        C::Schema,
    >,
) -> Reply<C::CompletionData> {
    if let Err(cause) = record.assess(application, request) {
        return Reply::DisclosureRefused {
            custody_id: id.to_owned(),
            diagnostic: cause.as_ref().map(super::diagnostic::mutation),
        };
    }
    Reply::Held {
        custody_id: id.to_owned(),
        next_sequence: record.sequence.checked_add(1),
        native: record.view(),
        completion_data: record.completion_data(),
    }
}
fn denied(reason: &str) -> Error {
    Error::denied(WorthServerProductOperationDenial::new(
        reason,
        "native source admission refused before command construction",
    ))
}
