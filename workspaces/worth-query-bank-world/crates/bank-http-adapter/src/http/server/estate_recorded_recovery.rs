//! Recorded completion can describe an existing claim, never create one.
use super::super::protocol::{BankHttpDenialKind, BankHttpMutationOutcome, BankHttpRecoveryStatus};
use super::{estate_source_custody::EstateSourceRecord, mutation_command::AdmittedMutationCommand};

pub(super) async fn describe<A: super::authentication::BankHttpApplicationAuthenticator>(
    application: &A,
    record: &EstateSourceRecord,
    command: &AdmittedMutationCommand,
    principal: &bank_server::BankAuthenticatedPrincipal,
    scope: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
) -> BankHttpMutationOutcome {
    let described =
        record
            .custody
            .describe(&command.custody_id, &command.request_id, command.sequence);
    let BankHttpMutationOutcome::Recorded { commit, .. } = described else {
        return described;
    };
    let ready = |recovery, recovery_status| BankHttpMutationOutcome::RecordedRailRecoveryReady {
        request_id: command.request_id.clone(),
        custody_id: command.custody_id.clone(),
        sequence: command.sequence,
        commit,
        recovery,
        recovery_status,
    };
    let refused = |kind| BankHttpMutationOutcome::Refused {
        request_id: command.request_id.clone(),
        custody_id: command.custody_id.clone(),
        sequence: command.sequence,
        denial: super::mutation_refusal::denial(kind),
        accepted_sequence: Some(command.sequence),
    };
    let runtime = application.runtime();
    let status = match runtime.observe_estate_recovery_claim_status(
        principal,
        record.action,
        &record.key,
        scope,
    ) {
        Ok(Some(status)) => status,
        Ok(None) => return refused(BankHttpDenialKind::Stale),
        Err(cause) => {
            return BankHttpMutationOutcome::Refused {
                request_id: command.request_id.clone(),
                custody_id: command.custody_id.clone(),
                sequence: command.sequence,
                denial: super::estate_denial::estate_denial(cause),
                accepted_sequence: Some(command.sequence),
            }
        }
    };
    let mut registry = record.registry.lock().await;
    registry.purge_expired(runtime);
    let replay = match record.action {
        bank_domain::estate::EstateAction::NotifyDeath { .. } => {
            registry.notification_replay(&record.owner, &record.key, record.action)
        }
        bank_domain::estate::EstateAction::DisburseEstate(_) => {
            registry.disbursement_replay(&record.owner, &record.key, record.action)
        }
        bank_domain::estate::EstateAction::RetransmitDeathNotice { .. }
        | bank_domain::estate::EstateAction::FreezeAccount { .. }
        | bank_domain::estate::EstateAction::OpenEstateCase { .. }
        | bank_domain::estate::EstateAction::RecognizeExecutor { .. }
        | bank_domain::estate::EstateAction::DelegateCapability { .. }
        | bank_domain::estate::EstateAction::RevokeCapability { .. }
        | bank_domain::estate::EstateAction::RequestEmergencyAccess { .. }
        | bank_domain::estate::EstateAction::ApproveEmergencyAccess { .. }
        | bank_domain::estate::EstateAction::RevokeEmergencyAccess { .. }
        | bank_domain::estate::EstateAction::CompleteMandatoryReview { .. }
        | bank_domain::estate::EstateAction::ReleaseEstate { .. }
        | bank_domain::estate::EstateAction::ViewRestrictedEstate { .. }
        | bank_domain::estate::EstateAction::ViewRestrictedEstateWithEmergencyAccess { .. } => {
            unreachable!("only the declared estate source bindings enter this owner")
        }
    };
    match replay {
        super::recovery_registry::BankHttpCommitReplay::Applied {
            recovery,
            completed,
        } => {
            return ready(
                Some(recovery),
                if completed {
                    BankHttpRecoveryStatus::Completed
                } else {
                    BankHttpRecoveryStatus::TokenIssued
                },
            );
        }
        super::recovery_registry::BankHttpCommitReplay::Conflicting => {
            return refused(BankHttpDenialKind::Stale)
        }
        super::recovery_registry::BankHttpCommitReplay::Missing => {}
    }
    match status {
        bank_server::BankRecoveryClaimStatus::Completed => {
            ready(None, BankHttpRecoveryStatus::Completed)
        }
        bank_server::BankRecoveryClaimStatus::Live
        | bank_server::BankRecoveryClaimStatus::Consumed
        | bank_server::BankRecoveryClaimStatus::Expired
        | bank_server::BankRecoveryClaimStatus::Disposed
        | bank_server::BankRecoveryClaimStatus::ForceTerminated => {
            ready(None, BankHttpRecoveryStatus::OperatorRequired)
        }
        bank_server::BankRecoveryClaimStatus::Unclaimed => refused(BankHttpDenialKind::Unavailable),
    }
}
