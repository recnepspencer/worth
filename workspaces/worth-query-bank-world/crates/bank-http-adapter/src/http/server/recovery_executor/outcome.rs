use bank_server::{
    BankCommitReceipt, BankIdentityRuntime, BankMutationCommitOutcome, BankRecoveryClaimStatus,
    BankRecoveryDenial, BankRecoveryPosture,
};

use super::super::super::protocol::{
    BankHttpCommitDescription, BankHttpDenial, BankHttpDenialKind,
    BankHttpEstateDisbursementOutcome, BankHttpEstateNotificationOutcome, BankHttpNextAction,
    BankHttpRecoveryInspectionOutcome, BankHttpRecoveryPosture, BankHttpRecoverySafeRetryOutcome,
    BankHttpRecoveryStatus,
};

pub(super) fn omitted_recovery_status(
    runtime: &BankIdentityRuntime,
    receipt: &BankCommitReceipt,
) -> Result<BankHttpRecoveryStatus, BankRecoveryDenial> {
    Ok(match runtime.commit_recovery_claim_status(receipt)? {
        BankRecoveryClaimStatus::Completed => BankHttpRecoveryStatus::Completed,
        BankRecoveryClaimStatus::Unclaimed
        | BankRecoveryClaimStatus::Live
        | BankRecoveryClaimStatus::Consumed
        | BankRecoveryClaimStatus::Expired
        | BankRecoveryClaimStatus::Disposed
        | BankRecoveryClaimStatus::ForceTerminated => BankHttpRecoveryStatus::OperatorRequired,
    })
}

pub(super) fn commit_description(
    receipt: &bank_server::BankCommitReceipt,
) -> BankHttpCommitDescription {
    BankHttpCommitDescription {
        changed_record_count: receipt.changed_record_count(),
        emitted_effect_count: receipt.emitted_effect_count(),
        expected_version_count: receipt.expected_version_count(),
        expected_fact_count: receipt.expected_fact_count(),
        provider_work_units: None,
        invariant_work_units: None,
    }
}

pub(super) const fn recovery_posture(posture: BankRecoveryPosture) -> BankHttpRecoveryPosture {
    match posture {
        BankRecoveryPosture::Reversible => BankHttpRecoveryPosture::Reversible,
        BankRecoveryPosture::Compensatable => BankHttpRecoveryPosture::Compensatable,
        BankRecoveryPosture::Reconcilable => BankHttpRecoveryPosture::Reconcilable,
        BankRecoveryPosture::Irreversible => BankHttpRecoveryPosture::Irreversible,
    }
}

pub(super) fn commit_denial(outcome: BankMutationCommitOutcome) -> BankHttpDenial {
    match outcome {
        BankMutationCommitOutcome::ProductStale(_) => stale(),
        BankMutationCommitOutcome::ProductUnpublished(_) => BankHttpDenial::new(
            BankHttpDenialKind::Unavailable,
            BankHttpNextAction::ContactOperator,
        ),
        BankMutationCommitOutcome::NoEffect(_) => unavailable(),
        BankMutationCommitOutcome::Stale { .. } => stale(),
        BankMutationCommitOutcome::Cancelled => {
            BankHttpDenial::new(BankHttpDenialKind::Cancelled, BankHttpNextAction::Retry)
        }
        BankMutationCommitOutcome::TimedOut => deadline_exceeded(),
        BankMutationCommitOutcome::Denied { .. }
        | BankMutationCommitOutcome::CustomInvariantDenied { .. }
        | BankMutationCommitOutcome::Aborted => unavailable(),
        BankMutationCommitOutcome::Deferred(_)
        | BankMutationCommitOutcome::SettlementDeferred(_)
        | BankMutationCommitOutcome::Indeterminate(_) => BankHttpDenial::new(
            BankHttpDenialKind::Unavailable,
            BankHttpNextAction::ContactOperator,
        ),
        BankMutationCommitOutcome::Committed(_)
        | BankMutationCommitOutcome::AlreadyCommitted(_) => unavailable(),
    }
}

pub(super) use super::super::estate_denial::estate_denial;

pub(super) fn notification_denied(
    request_id: Option<String>,
    denial: BankHttpDenial,
) -> BankHttpEstateNotificationOutcome {
    BankHttpEstateNotificationOutcome::Denied { request_id, denial }
}

pub(super) fn inspection_denied(
    request_id: Option<String>,
    denial: BankHttpDenial,
) -> BankHttpRecoveryInspectionOutcome {
    BankHttpRecoveryInspectionOutcome::Denied { request_id, denial }
}

pub(super) fn safe_retry_denied(
    request_id: Option<String>,
    denial: BankHttpDenial,
) -> BankHttpRecoverySafeRetryOutcome {
    BankHttpRecoverySafeRetryOutcome::Denied { request_id, denial }
}

pub(super) fn disbursement_denied(
    request_id: Option<String>,
    denial: BankHttpDenial,
) -> BankHttpEstateDisbursementOutcome {
    BankHttpEstateDisbursementOutcome::Denied { request_id, denial }
}

pub(super) const fn stale() -> BankHttpDenial {
    BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)
}

pub(super) const fn saturated() -> BankHttpDenial {
    BankHttpDenial::new(BankHttpDenialKind::Saturated, BankHttpNextAction::Retry)
}

pub(super) const fn unavailable() -> BankHttpDenial {
    BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry)
}

pub(super) const fn deadline_exceeded() -> BankHttpDenial {
    BankHttpDenial::new(
        BankHttpDenialKind::DeadlineExceeded,
        BankHttpNextAction::Retry,
    )
}

pub(super) const fn conflicting_idempotency_key() -> BankHttpDenial {
    BankHttpDenial::new(
        BankHttpDenialKind::MalformedRequest,
        BankHttpNextAction::CorrectRequest,
    )
}
