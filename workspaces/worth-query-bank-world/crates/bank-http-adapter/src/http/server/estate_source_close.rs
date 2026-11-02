//! Rail handoff borrows the complete closed source retained by the worker.
use super::super::protocol::{BankHttpDenialKind, BankHttpMutationOutcome, BankHttpRecoveryStatus};
use super::{estate_source_custody::EstateSourceRecord, mutation_command::AdmittedMutationCommand};

pub(super) async fn describe<A: super::authentication::BankHttpApplicationAuthenticator>(
    application: &A,
    record: &EstateSourceRecord,
    command: &AdmittedMutationCommand,
) -> BankHttpMutationOutcome {
    let Some(receipt) = record.custody.live_receipt() else {
        return record
            .custody
            .describe(&command.custody_id, &command.request_id, command.sequence);
    };
    let runtime = application.runtime();
    let commit = super::recovery_executor::outcome::commit_description(receipt);
    let ready = |recovery, recovery_status| BankHttpMutationOutcome::RailRecoveryReady {
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
    if let super::recovery_registry::BankHttpCommitReplay::Applied {
        recovery,
        completed,
        ..
    } = replay
    {
        return ready(
            Some(recovery),
            if completed {
                BankHttpRecoveryStatus::Completed
            } else {
                BankHttpRecoveryStatus::TokenIssued
            },
        );
    }
    let reservation = match record.action {
        bank_domain::estate::EstateAction::NotifyDeath { .. } => {
            registry.reserve_notification(record.owner.clone(), record.key.clone(), record.action)
        }
        bank_domain::estate::EstateAction::DisburseEstate(_) => {
            registry.reserve_disbursement(record.owner.clone(), record.key.clone(), record.action)
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
    let Some(reservation) = reservation else {
        return match runtime.commit_recovery_claim_status(receipt) {
            Ok(status) if status != bank_server::BankRecoveryClaimStatus::Unclaimed => ready(
                None,
                if status == bank_server::BankRecoveryClaimStatus::Completed {
                    BankHttpRecoveryStatus::Completed
                } else {
                    BankHttpRecoveryStatus::OperatorRequired
                },
            ),
            Ok(_) => refused(BankHttpDenialKind::Saturated),
            Err(_) => refused(BankHttpDenialKind::Unavailable),
        };
    };
    match runtime.open_commit_recovery(receipt) {
        Ok(handle) => {
            let recovery =
                reservation.register(super::recovery_registry::BankHttpRecoveryRegistration {
                    commit,
                    handle,
                });
            ready(Some(recovery), BankHttpRecoveryStatus::TokenIssued)
        }
        Err(cause)
            if matches!(
                cause.kind(),
                bank_server::BankRecoveryDenialKind::RecoveryAlreadyMinted
                    | bank_server::BankRecoveryDenialKind::AlreadyTerminal
            ) =>
        {
            match super::recovery_executor::outcome::omitted_recovery_status(runtime, receipt) {
                Ok(status) => ready(None, status),
                Err(_) => refused(BankHttpDenialKind::Unavailable),
            }
        }
        Err(_) => refused(BankHttpDenialKind::Unavailable),
    }
}
