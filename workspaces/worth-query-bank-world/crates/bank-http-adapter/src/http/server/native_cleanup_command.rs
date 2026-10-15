//! Fresh issuer admission on the original execution thread, without business execution.
use super::super::protocol::{BankHttpDenialKind, BankHttpMutationOutcome};
use super::{
    mutation_command::{AdmittedMoneyCommand, MoneyCommand},
    mutation_custody::MutationRecord,
    mutation_refusal::denial,
    native_cleanup_custody::NativeCleanupCustody,
};
use std::collections::HashMap;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

pub(super) fn execute<A: super::authentication::BankHttpApplicationAuthenticator>(
    application: &A,
    records: &mut HashMap<String, MutationRecord>,
    command: AdmittedMoneyCommand,
    scope: WorthQueryRequestScope,
    owner: super::authenticated_owner::BankHttpAuthenticatedOwner,
) -> BankHttpMutationOutcome {
    let MutationRecord::Native(record) = records.get(&command.custody_id).expect("worker lookup")
    else {
        unreachable!("native dispatch")
    };
    let refused = |kind, accepted_sequence| BankHttpMutationOutcome::Refused {
        request_id: command.request_id.clone(),
        custody_id: command.custody_id.clone(),
        sequence: command.sequence,
        denial: denial(kind),
        accepted_sequence,
    };
    if record.owner != owner {
        return refused(BankHttpDenialKind::PermissionDenied, None);
    }
    if record.sequence.checked_add(1) != Some(command.sequence) {
        return refused(BankHttpDenialKind::Stale, Some(record.sequence));
    }
    if !matches!(command.command, MoneyCommand::ReleaseCleanup) {
        return refused(
            BankHttpDenialKind::ManagedCleanupRequired,
            Some(record.sequence),
        );
    }
    let MutationRecord::Native(mut record) =
        records.remove(&command.custody_id).expect("worker lookup")
    else {
        unreachable!("native dispatch")
    };
    record.sequence = command.sequence;
    let (custody, outcome) = match record.custody {
        NativeCleanupCustody::Pending(custody) => {
            match custody.release(application.runtime(), &scope) {
                Ok(evidence) => {
                    let outcome = released(&command, &evidence);
                    (NativeCleanupCustody::Released(evidence), outcome)
                }
                Err((custody, cause)) => {
                    let kind = cause
                        .or_else(|| custody.last_release_denial().cloned())
                        .map(super::advancement_denial::advancement)
                        .unwrap_or_else(|| denial(BankHttpDenialKind::ManagedCleanupRequired));
                    let outcome = BankHttpMutationOutcome::Refused {
                        request_id: command.request_id.clone(),
                        custody_id: command.custody_id.clone(),
                        sequence: command.sequence,
                        denial: kind,
                        accepted_sequence: Some(command.sequence),
                    };
                    (NativeCleanupCustody::Pending(custody), outcome)
                }
            }
        }
        NativeCleanupCustody::Released(evidence) => {
            let outcome = match application
                .runtime()
                .with_application_advancement(&scope, |_| released(&command, &evidence))
            {
                Ok(outcome) => outcome,
                Err(cause) => BankHttpMutationOutcome::Refused {
                    request_id: command.request_id.clone(),
                    custody_id: command.custody_id.clone(),
                    sequence: command.sequence,
                    denial: super::advancement_denial::advancement(cause),
                    accepted_sequence: Some(command.sequence),
                },
            };
            (NativeCleanupCustody::Released(evidence), outcome)
        }
    };
    record.custody = custody;
    records.insert(command.custody_id, MutationRecord::Native(record));
    outcome
}
fn released(
    command: &AdmittedMoneyCommand,
    evidence: &bank_server::BankReleasedManagedCleanup,
) -> BankHttpMutationOutcome {
    let inspection = evidence.receipt().inspection();
    BankHttpMutationOutcome::CleanupReleased {
        request_id: command.request_id.clone(),
        custody_id: command.custody_id.clone(),
        sequence: command.sequence,
        logical_run_identity: inspection.logical_run_identity().into(),
        run_identity: inspection.run_identity().into(),
        provider_session_identity: inspection.provider_session_identity().into(),
        resources_released: inspection.resources_released(),
    }
}
