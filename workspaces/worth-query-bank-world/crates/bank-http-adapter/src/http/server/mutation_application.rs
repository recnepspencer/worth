use std::time::Instant;

use bank_domain::model::{AccountId, BankPrincipalId, InstitutionId, Money, USD};
use bank_domain::proposals::BankIdempotencyKey;
use bank_domain::schema::{Deposit, SendMoney, Withdraw};
use bank_server::{mutations, BankMoneyMovementExecution, BankMutationControls};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitRecoveryKind,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationToken, WorthQueryRequestScope},
    application_entry::WorthQueryApplicationMutationOutcome,
    primary_graph::{WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt},
};

use super::super::protocol::{
    BankHttpCommitDescription, BankHttpCommitDisposition, BankHttpCredential, BankHttpDenial,
    BankHttpDenialKind, BankHttpMutationFailureKind, BankHttpMutationOutcome, BankHttpNextAction,
    BankHttpProviderRecoveryKind,
};
use super::authentication::BankHttpApplicationAuthenticator;

mod denial;
use denial::request_mutation_denial;

pub(super) enum AdmittedBankHttpMutation {
    Deposit(Deposit),
    Withdraw(Withdraw),
    SendMoney(SendMoney),
}

pub(super) struct AdmittedBankHttpMutationRequest {
    pub(super) request_id: String,
    pub(super) credential: BankHttpCredential,
    pub(super) idempotency_key: BankIdempotencyKey,
    pub(super) operation: AdmittedBankHttpMutation,
    pub(super) deadline: Instant,
}

pub(super) async fn execute_mutation<A>(
    application: &A,
    request: AdmittedBankHttpMutationRequest,
    cancellation: WorthQueryCancellationToken,
) -> BankHttpMutationOutcome
where
    A: BankHttpApplicationAuthenticator,
{
    let request_id = request.request_id;
    let scope = WorthQueryRequestScope::new(request.deadline, cancellation);
    let principal = match application.authenticate(request.credential, &scope).await {
        Ok(principal) => principal,
        Err(denial) => return not_applied(Some(request_id), cancelled_or_denied(denial), denial),
    };
    let controls = BankMutationControls::new(scope, request.idempotency_key);
    let outcome = match request.operation {
        AdmittedBankHttpMutation::Deposit(input) => application
            .runtime()
            .mutate(mutations::deposit(input))
            .as_principal(&principal)
            .controls(controls)
            .execute(),
        AdmittedBankHttpMutation::Withdraw(input) => application
            .runtime()
            .mutate(mutations::withdraw(input))
            .as_principal(&principal)
            .controls(controls)
            .execute(),
        AdmittedBankHttpMutation::SendMoney(input) => application
            .runtime()
            .mutate(mutations::send_money(input))
            .as_principal(&principal)
            .controls(controls)
            .execute(),
    };
    describe_outcome(request_id, outcome)
}

pub(super) fn parse_deposit(
    institution: &str,
    account: &str,
    amount_minor_units: i64,
) -> Option<AdmittedBankHttpMutation> {
    Some(AdmittedBankHttpMutation::Deposit(Deposit {
        institution: InstitutionId::parse_canonical_text(institution)?,
        account: AccountId::parse_canonical_text(account)?,
        amount: Money::<USD>::from_minor(amount_minor_units).ok()?,
    }))
}

pub(super) fn parse_withdraw(
    institution: &str,
    account: &str,
    amount_minor_units: i64,
) -> Option<AdmittedBankHttpMutation> {
    Some(AdmittedBankHttpMutation::Withdraw(Withdraw {
        institution: InstitutionId::parse_canonical_text(institution)?,
        account: AccountId::parse_canonical_text(account)?,
        amount: Money::<USD>::from_minor(amount_minor_units).ok()?,
    }))
}

pub(super) fn parse_send_money(
    from: &str,
    recipient: &str,
    amount_minor_units: i64,
) -> Option<AdmittedBankHttpMutation> {
    Some(AdmittedBankHttpMutation::SendMoney(SendMoney {
        from: AccountId::parse_canonical_text(from)?,
        recipient: BankPrincipalId::parse_canonical_text(recipient)?,
        amount: Money::<USD>::from_minor(amount_minor_units).ok()?,
    }))
}

fn describe_outcome(
    request_id: String,
    outcome: BankMoneyMovementExecution,
) -> BankHttpMutationOutcome {
    match outcome {
        Err(denial) => {
            let wire = request_mutation_denial(denial.kind());
            not_applied(Some(request_id), cancelled_or_denied(wire), wire)
        }
        Ok(WorthQueryApplicationMutationOutcome::Committed { receipt, .. }) => {
            applied(request_id, BankHttpCommitDisposition::Committed, receipt)
        }
        Ok(WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt)) => applied(
            request_id,
            BankHttpCommitDisposition::AlreadyCommitted,
            receipt,
        ),
        Ok(WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift) => not_applied(
            Some(request_id),
            BankHttpMutationFailureKind::Aborted,
            BankHttpDenial::new(
                BankHttpDenialKind::Stale,
                BankHttpNextAction::CorrectRequest,
            ),
        ),
        Ok(WorthQueryApplicationMutationOutcome::DomainDenied(_)) => not_applied(
            Some(request_id),
            BankHttpMutationFailureKind::InvariantViolated,
            BankHttpDenial::new(
                BankHttpDenialKind::MalformedRequest,
                BankHttpNextAction::CorrectRequest,
            ),
        ),
        Ok(WorthQueryApplicationMutationOutcome::Cancelled) => not_applied(
            Some(request_id),
            BankHttpMutationFailureKind::Cancelled,
            BankHttpDenial::new(BankHttpDenialKind::Cancelled, BankHttpNextAction::Retry),
        ),
        Ok(WorthQueryApplicationMutationOutcome::DeadlineExceeded) => not_applied(
            Some(request_id),
            BankHttpMutationFailureKind::DeadlineExceeded,
            BankHttpDenial::new(
                BankHttpDenialKind::DeadlineExceeded,
                BankHttpNextAction::Retry,
            ),
        ),
        Ok(WorthQueryApplicationMutationOutcome::Commit(outcome)) => {
            describe_commit_outcome(request_id, outcome)
        }
    }
}

fn describe_commit_outcome(
    request_id: String,
    outcome: WorthQueryApplicationCommitOutcome,
) -> BankHttpMutationOutcome {
    match outcome {
        WorthQueryApplicationCommitOutcome::ProductStale(_) => not_applied(
            Some(request_id),
            BankHttpMutationFailureKind::ProductStale,
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh),
        ),
        WorthQueryApplicationCommitOutcome::ProductUnpublished(_) => {
            recovery_required(request_id, BankHttpMutationFailureKind::ProductUnpublished)
        }
        WorthQueryApplicationCommitOutcome::NoEffect(_) => not_applied(
            Some(request_id),
            BankHttpMutationFailureKind::NoEffect,
            BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry),
        ),
        WorthQueryApplicationCommitOutcome::Committed(receipt) => {
            applied(request_id, BankHttpCommitDisposition::Committed, receipt)
        }
        WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) => applied(
            request_id,
            BankHttpCommitDisposition::AlreadyCommitted,
            receipt,
        ),
        WorthQueryApplicationCommitOutcome::Stale(stale) => BankHttpMutationOutcome::NotApplied {
            request_id: Some(request_id),
            failure: BankHttpMutationFailureKind::Stale,
            stale_fact_count: Some(stale.stale_fact_count()),
            provider_recovery: None,
            denial: BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh),
        },
        WorthQueryApplicationCommitOutcome::Cancelled => not_applied(
            Some(request_id),
            BankHttpMutationFailureKind::Cancelled,
            BankHttpDenial::new(BankHttpDenialKind::Cancelled, BankHttpNextAction::Retry),
        ),
        WorthQueryApplicationCommitOutcome::TimedOut => not_applied(
            Some(request_id),
            BankHttpMutationFailureKind::TimedOut,
            BankHttpDenial::new(
                BankHttpDenialKind::DeadlineExceeded,
                BankHttpNextAction::Retry,
            ),
        ),
        WorthQueryApplicationCommitOutcome::Denied(denial) => {
            let (failure, wire) = commit_denial(denial.kind());
            not_applied(Some(request_id), failure, wire)
        }
        WorthQueryApplicationCommitOutcome::Aborted => not_applied(
            Some(request_id),
            BankHttpMutationFailureKind::Aborted,
            BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry),
        ),
        WorthQueryApplicationCommitOutcome::Deferred(_) => {
            recovery_required(request_id, BankHttpMutationFailureKind::Deferred)
        }
        WorthQueryApplicationCommitOutcome::SettlementDeferred(_) => {
            recovery_required(request_id, BankHttpMutationFailureKind::SettlementDeferred)
        }
        WorthQueryApplicationCommitOutcome::Indeterminate(evidence) => {
            let provider_recovery = match evidence.recovery() {
                WorthQueryApplicationCommitRecoveryKind::CommitRecoveryRequired => {
                    BankHttpProviderRecoveryKind::CommitRecoveryRequired
                }
                WorthQueryApplicationCommitRecoveryKind::AbortRecoveryRequired => {
                    BankHttpProviderRecoveryKind::AbortRecoveryRequired
                }
            };
            BankHttpMutationOutcome::NotApplied {
                request_id: Some(request_id),
                failure: BankHttpMutationFailureKind::Indeterminate,
                stale_fact_count: None,
                provider_recovery: Some(provider_recovery),
                denial: BankHttpDenial::new(
                    BankHttpDenialKind::Unavailable,
                    BankHttpNextAction::ContactOperator,
                ),
            }
        }
    }
}

fn applied(
    request_id: String,
    disposition: BankHttpCommitDisposition,
    receipt: WorthQueryApplicationCommitReceipt,
) -> BankHttpMutationOutcome {
    let invariant_work_units = receipt
        .mutation_work()
        .and_then(|work| usize::try_from(work.invariant_work_units()).ok());
    let preconditions = receipt.precondition_comparison();
    BankHttpMutationOutcome::Applied {
        request_id,
        disposition,
        commit: BankHttpCommitDescription {
            changed_record_count: receipt.changed_record_count(),
            emitted_effect_count: receipt.emitted_effect_count(),
            expected_version_count: preconditions.expected_version_count(),
            expected_fact_count: preconditions.expected_fact_count(),
            provider_work_units: None,
            invariant_work_units,
        },
    }
}

fn recovery_required(
    request_id: String,
    failure: BankHttpMutationFailureKind,
) -> BankHttpMutationOutcome {
    not_applied(
        Some(request_id),
        failure,
        BankHttpDenial::new(
            BankHttpDenialKind::Unavailable,
            BankHttpNextAction::ContactOperator,
        ),
    )
}

fn not_applied(
    request_id: Option<String>,
    failure: BankHttpMutationFailureKind,
    denial: BankHttpDenial,
) -> BankHttpMutationOutcome {
    BankHttpMutationOutcome::NotApplied {
        request_id,
        failure,
        stale_fact_count: None,
        provider_recovery: None,
        denial,
    }
}

fn cancelled_or_denied(denial: BankHttpDenial) -> BankHttpMutationFailureKind {
    match denial.kind {
        BankHttpDenialKind::Cancelled => BankHttpMutationFailureKind::Cancelled,
        BankHttpDenialKind::DeadlineExceeded => BankHttpMutationFailureKind::DeadlineExceeded,
        _ => BankHttpMutationFailureKind::Aborted,
    }
}

pub(super) fn commit_denial(
    kind: WorthQueryApplicationCommitDenialKind,
) -> (BankHttpMutationFailureKind, BankHttpDenial) {
    use WorthQueryApplicationCommitDenialKind as Denial;
    match kind {
        Denial::ProductBasisStale => (
            BankHttpMutationFailureKind::ProductStale,
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh),
        ),
        Denial::CustomInvariantDenied => (
            BankHttpMutationFailureKind::InvariantViolated,
            BankHttpDenial::new(
                BankHttpDenialKind::MalformedRequest,
                BankHttpNextAction::CorrectRequest,
            ),
        ),
        Denial::IdempotencyIntentDrift => (
            BankHttpMutationFailureKind::Aborted,
            BankHttpDenial::new(
                BankHttpDenialKind::Stale,
                BankHttpNextAction::CorrectRequest,
            ),
        ),
        Denial::CandidateValidatorWorkExceeded { .. }
        | Denial::WorkflowSettlementDenied { .. }
        | Denial::PreparedRootBudgetExhausted { .. }
        | Denial::ElevationTransitionRequired
        | Denial::ElevationRequestProgramMismatch
        | Denial::ElevationApprovalProgramMismatch
        | Denial::ElevationCloseProgramMismatch
        | Denial::MandatoryReviewProgramMismatch
        | Denial::DelegationActivationRequired
        | Denial::CapabilityRevocationRequired
        | Denial::ApplicationProgramRequired
        | Denial::WorkflowAuthorityRequired => (
            BankHttpMutationFailureKind::Aborted,
            BankHttpDenial::new(
                BankHttpDenialKind::MalformedRequest,
                BankHttpNextAction::CorrectRequest,
            ),
        ),
        Denial::ProviderRejected
        | Denial::ActiveSnapshotCapacityExhausted { .. }
        | Denial::RetentionCapacityExhausted
        | Denial::IndexMaintenanceBudgetExceeded => (
            BankHttpMutationFailureKind::Aborted,
            BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry),
        ),
        Denial::RetentionIdentityExhausted
        | Denial::SnapshotIdentityExhausted
        | Denial::CandidateIdentityExhausted
        | Denial::IndexGenerationIdentityExhausted
        | Denial::ProgramActivationUnresolved => (
            BankHttpMutationFailureKind::Aborted,
            BankHttpDenial::new(
                BankHttpDenialKind::Unavailable,
                BankHttpNextAction::ContactOperator,
            ),
        ),
        Denial::ProgramNotActiveOnOccurrence => (
            BankHttpMutationFailureKind::ProductStale,
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh),
        ),
    }
}
