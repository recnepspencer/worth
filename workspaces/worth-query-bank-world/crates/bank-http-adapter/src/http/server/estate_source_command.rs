//! Source custody advances natively; external rail recovery receives only its actual closed witness.
use super::super::protocol::{BankHttpDenial, BankHttpDenialKind, BankHttpMutationOutcome};
use super::{
    mutation_command::{AdmittedMutationCommand, MutationCommand},
    mutation_custody::MutationRecord,
    mutation_refusal::denial,
};
use std::collections::HashMap;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

pub(super) async fn execute<A: super::authentication::BankHttpApplicationAuthenticator>(
    application: &A,
    records: &mut HashMap<String, MutationRecord>,
    command: AdmittedMutationCommand,
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
        MutationCommand::ResumeSource => {
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
        MutationCommand::ReleaseCleanup => {
            if record.released_cleanup.is_some() {
                refused(
                    &command,
                    denial(BankHttpDenialKind::Stale),
                    Some(command.sequence),
                )
            } else {
                match record.custody.release(&request) {
                    Ok(receipt) => {
                        let outcome = BankHttpMutationOutcome::CollectionReleased {
                            request_id: command.request_id.clone(),
                            custody_id: command.custody_id.clone(),
                            sequence: command.sequence,
                            retired_component_count: receipt.retired_component_count(),
                            managed: receipt
                                .managed_cleanup_receipt()
                                .map(super::cleanup_description::describe),
                        };
                        record.released_cleanup = Some(receipt);
                        outcome
                    }
                    Err(cause) => refused(&command, cause, Some(command.sequence)),
                }
            }
        }
        MutationCommand::Close => match record.custody.close(&request) {
            Ok(custody) => {
                record.custody = custody;
                super::estate_source_close::describe(
                    application,
                    &record,
                    &command,
                    principal,
                    &scope,
                )
                .await
            }
            Err((custody, cause)) => {
                record.custody = custody;
                refused(&command, cause, Some(command.sequence))
            }
        },
        _ => refused(
            &command,
            denial(BankHttpDenialKind::MalformedRequest),
            Some(command.sequence),
        ),
    };
    match super::closed_mutation_custody::ClosedMutationRecord::from_estate(record, permit) {
        Ok(record) => {
            if !record.may_release() {
                records.insert(command.custody_id, MutationRecord::Closed(record));
            }
        }
        Err((record, permit)) => {
            records.insert(command.custody_id, MutationRecord::Estate(record, permit));
        }
    }
    outcome
}
fn refused(
    command: &AdmittedMutationCommand,
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
