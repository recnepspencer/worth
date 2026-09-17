use bank_server::{
    BankMutationCommitOutcome, BankRecoveryDenialKind, BankRecoveryIdempotencyResolution,
};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use super::super::super::protocol::{
    BankHttpCommitDisposition, BankHttpEstateDisbursementOutcome, BankHttpRecoveryStatus,
};
use super::super::authenticated_owner::BankHttpAuthenticatedOwner;
use super::super::authentication::BankHttpApplicationAuthenticator;
use super::super::recovery_registry::{
    BankHttpCommitReplay, BankHttpRecoveryRegistration, BankHttpRecoveryRegistry,
};
use super::{outcome::*, AdmittedBankHttpDisbursementRequest};

pub(super) async fn execute_disbursement<A>(
    application: &A,
    registry: &mut BankHttpRecoveryRegistry,
    request: AdmittedBankHttpDisbursementRequest,
    cancellation: WorthQueryCancellationSource,
) -> BankHttpEstateDisbursementOutcome
where
    A: BankHttpApplicationAuthenticator,
{
    let request_id = request.request_id;
    let scope = WorthQueryRequestScope::new(request.deadline, cancellation.token());
    let principal = match application.authenticate(request.credential, &scope).await {
        Ok(principal) => principal,
        Err(denial) => return disbursement_denied(Some(request_id), denial),
    };
    let owner = BankHttpAuthenticatedOwner::from_principal(&principal);
    match registry.disbursement_replay(&owner, &request.idempotency_key, request.action) {
        BankHttpCommitReplay::Applied {
            commit,
            recovery,
            completed,
        } => {
            return BankHttpEstateDisbursementOutcome::Applied {
                request_id,
                disposition: BankHttpCommitDisposition::AlreadyCommitted,
                commit,
                recovery: Some(recovery),
                recovery_status: if completed {
                    BankHttpRecoveryStatus::Completed
                } else {
                    BankHttpRecoveryStatus::TokenIssued
                },
            };
        }
        BankHttpCommitReplay::Conflicting => {
            return disbursement_denied(Some(request_id), conflicting_idempotency_key());
        }
        BankHttpCommitReplay::Missing => {}
    }
    let reservation =
        registry.reserve_disbursement(owner, request.idempotency_key.clone(), request.action);
    if reservation.is_none() {
        let resolution = match application.runtime().resolve_estate_action_idempotency(
            &principal,
            request.action,
            &request.idempotency_key,
            &scope,
        ) {
            Ok(resolution) => resolution,
            Err(denial) => return disbursement_denied(Some(request_id), estate_denial(denial)),
        };
        match resolution {
            BankRecoveryIdempotencyResolution::AlreadyCommitted => {}
            BankRecoveryIdempotencyResolution::IntentDrift => {
                return disbursement_denied(Some(request_id), conflicting_idempotency_key());
            }
            BankRecoveryIdempotencyResolution::Unseen => {
                return disbursement_denied(Some(request_id), saturated());
            }
        }
    }
    let outcome = match application.runtime().disburse_estate_with_key(
        &principal,
        request.action,
        &request.idempotency_key,
        &scope,
    ) {
        Ok(outcome) => outcome,
        Err(denial) => return disbursement_denied(Some(request_id), estate_denial(denial)),
    };
    let (disposition, receipt) = match outcome {
        BankMutationCommitOutcome::Committed(receipt) => {
            (BankHttpCommitDisposition::Committed, receipt)
        }
        BankMutationCommitOutcome::AlreadyCommitted(receipt) => {
            (BankHttpCommitDisposition::AlreadyCommitted, receipt)
        }
        other => return disbursement_denied(Some(request_id), commit_denial(other)),
    };
    let commit = commit_description(&receipt);
    let Some(reservation) = reservation else {
        let recovery_status = match omitted_recovery_status(application.runtime(), &receipt) {
            Ok(status) => status,
            Err(_) => return disbursement_denied(Some(request_id), unavailable()),
        };
        return BankHttpEstateDisbursementOutcome::Applied {
            request_id,
            disposition,
            commit,
            recovery: None,
            recovery_status,
        };
    };
    let handle = match application.runtime().open_commit_recovery(&receipt) {
        Ok(handle) => handle,
        Err(denial)
            if disposition == BankHttpCommitDisposition::AlreadyCommitted
                && matches!(
                    denial.kind(),
                    BankRecoveryDenialKind::RecoveryAlreadyMinted
                        | BankRecoveryDenialKind::AlreadyTerminal
                ) =>
        {
            let recovery_status = match omitted_recovery_status(application.runtime(), &receipt) {
                Ok(status) => status,
                Err(_) => return disbursement_denied(Some(request_id), unavailable()),
            };
            return BankHttpEstateDisbursementOutcome::Applied {
                request_id,
                disposition,
                commit,
                recovery: None,
                recovery_status,
            };
        }
        Err(_) => return disbursement_denied(Some(request_id), unavailable()),
    };
    let recovery = reservation.register(BankHttpRecoveryRegistration { commit, handle });
    BankHttpEstateDisbursementOutcome::Applied {
        request_id,
        disposition,
        commit,
        recovery: Some(recovery),
        recovery_status: BankHttpRecoveryStatus::TokenIssued,
    }
}
