use std::time::Instant;

use bank_domain::model::{AccountId, BankPrincipalId, InstitutionId, Money, USD};
use bank_domain::proposals::BankIdempotencyKey;
use bank_domain::schema::{Deposit, SendMoney, Withdraw};
use bank_server::{
    mutations, BankCommitReceipt, BankMutationControls, BankMutationDenial, BankMutationOutcome,
    BankMutationStatus,
};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationToken, WorthQueryRequestScope,
};

use super::super::protocol::{
    BankHttpCommitDescription, BankHttpCommitDisposition, BankHttpCredential, BankHttpDenial,
    BankHttpDenialKind, BankHttpMutationFailureKind, BankHttpMutationOutcome, BankHttpNextAction,
};
use super::authentication::BankHttpApplicationAuthenticator;

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

fn describe_outcome(request_id: String, outcome: BankMutationOutcome) -> BankHttpMutationOutcome {
    let provider_work_units = outcome.metadata().provider_work_units();
    match outcome.into_status() {
        BankMutationStatus::Committed(receipt) => applied(
            request_id,
            BankHttpCommitDisposition::Committed,
            receipt,
            provider_work_units,
        ),
        BankMutationStatus::AlreadyCommitted(receipt) => applied(
            request_id,
            BankHttpCommitDisposition::AlreadyCommitted,
            receipt,
            provider_work_units,
        ),
        BankMutationStatus::Stale { stale_fact_count } => BankHttpMutationOutcome::NotApplied {
            request_id: Some(request_id),
            failure: BankHttpMutationFailureKind::Stale,
            stale_fact_count: Some(stale_fact_count),
            denial: BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh),
        },
        BankMutationStatus::Cancelled => not_applied(
            Some(request_id),
            BankHttpMutationFailureKind::Cancelled,
            BankHttpDenial::new(BankHttpDenialKind::Cancelled, BankHttpNextAction::Retry),
        ),
        BankMutationStatus::DeadlineExceeded => not_applied(
            Some(request_id),
            BankHttpMutationFailureKind::DeadlineExceeded,
            BankHttpDenial::new(
                BankHttpDenialKind::DeadlineExceeded,
                BankHttpNextAction::Retry,
            ),
        ),
        BankMutationStatus::Denied(denial) => {
            let wire = mutation_denial(&denial);
            not_applied(Some(request_id), cancelled_or_denied(wire), wire)
        }
        BankMutationStatus::InvariantViolated(_) => not_applied(
            Some(request_id),
            BankHttpMutationFailureKind::InvariantViolated,
            BankHttpDenial::new(
                BankHttpDenialKind::MalformedRequest,
                BankHttpNextAction::CorrectRequest,
            ),
        ),
        BankMutationStatus::Aborted => not_applied(
            Some(request_id),
            BankHttpMutationFailureKind::Aborted,
            BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry),
        ),
        BankMutationStatus::PartialEffect(_) => {
            recovery_required(request_id, BankHttpMutationFailureKind::PartialEffect)
        }
        BankMutationStatus::Indeterminate(_) => {
            recovery_required(request_id, BankHttpMutationFailureKind::Indeterminate)
        }
    }
}

fn applied(
    request_id: String,
    disposition: BankHttpCommitDisposition,
    receipt: BankCommitReceipt,
    provider_work_units: usize,
) -> BankHttpMutationOutcome {
    BankHttpMutationOutcome::Applied {
        request_id,
        disposition,
        commit: BankHttpCommitDescription {
            changed_record_count: receipt.changed_record_count(),
            emitted_effect_count: receipt.emitted_effect_count(),
            expected_version_count: receipt.expected_version_count(),
            expected_fact_count: receipt.expected_fact_count(),
            provider_work_units: Some(provider_work_units),
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

fn mutation_denial(denial: &BankMutationDenial) -> BankHttpDenial {
    match denial {
        BankMutationDenial::Scope(_) => BankHttpDenial::new(
            BankHttpDenialKind::NotFound,
            BankHttpNextAction::CorrectRequest,
        ),
        BankMutationDenial::Authorization(_) => BankHttpDenial::new(
            BankHttpDenialKind::PermissionDenied,
            BankHttpNextAction::None,
        ),
        BankMutationDenial::IdempotencyIntentDrift => BankHttpDenial::new(
            BankHttpDenialKind::Stale,
            BankHttpNextAction::CorrectRequest,
        ),
        BankMutationDenial::Proposal(_) => BankHttpDenial::new(
            BankHttpDenialKind::MalformedRequest,
            BankHttpNextAction::CorrectRequest,
        ),
        BankMutationDenial::Installation(_)
        | BankMutationDenial::Preparation(_)
        | BankMutationDenial::Commit { .. }
        | BankMutationDenial::CustomInvariant { .. } => {
            BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry)
        }
    }
}
