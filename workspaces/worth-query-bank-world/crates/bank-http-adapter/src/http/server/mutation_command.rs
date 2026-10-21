//! Strict transport command admission precedes any native effect.
use super::super::protocol::*;
use super::mutation_application::{
    parse_deposit, parse_send_money, parse_withdraw, AdmittedBankHttpMutation,
};
use super::mutation_refusal::denial;
use bank_domain::proposals::BankIdempotencyKey;
use std::{
    num::NonZeroUsize,
    time::{Duration, Instant},
};
pub(super) enum MoneyCommand {
    Create {
        input: AdmittedBankHttpMutation,
        key: BankIdempotencyKey,
        history: NonZeroUsize,
    },
    Resume {
        history: NonZeroUsize,
    },
    Read {
        maximum_results: usize,
    },
    ResumeSource,
    ReleaseCleanup,
    Close,
}
pub(super) struct AdmittedMoneyCommand {
    pub(super) request_id: String,
    pub(super) credential: BankHttpCredential,
    pub(super) custody_id: String,
    pub(super) sequence: u64,
    pub(super) deadline: Instant,
    pub(super) command: MoneyCommand,
}
#[allow(
    clippy::result_large_err,
    reason = "Admission preserves the by-value protocol refusal without a wrapper allocation"
)]
pub(super) fn admit(
    request: BankHttpMutationRequest,
    maximum_deadline: Duration,
) -> Result<AdmittedMoneyCommand, BankHttpMutationOutcome> {
    let (protocol, request_id, credential, custody_id, sequence, deadline, command) = match request
    {
        BankHttpMutationRequest::Create {
            protocol,
            request_id,
            credential,
            custody_id,
            sequence,
            controls,
            idempotency_key,
            operation,
        } => {
            let input = match operation {
                BankHttpMutationOperation::Deposit {
                    institution,
                    account,
                    amount_minor_units,
                } => parse_deposit(&institution, &account, amount_minor_units),
                BankHttpMutationOperation::Withdraw {
                    institution,
                    account,
                    amount_minor_units,
                } => parse_withdraw(&institution, &account, amount_minor_units),
                BankHttpMutationOperation::SendMoney {
                    from,
                    recipient,
                    amount_minor_units,
                    expected_accounting_revision,
                    expected_status,
                } => parse_send_money(&from, &recipient, amount_minor_units).map(|input| {
                    let AdmittedBankHttpMutation::SendMoney(input, _, _) = input else {
                        unreachable!("send parser")
                    };
                    AdmittedBankHttpMutation::SendMoney(
                        input,
                        expected_accounting_revision,
                        expected_status,
                    )
                }),
            }
            .ok_or_else(|| {
                reject(
                    Some(request_id.clone()),
                    BankHttpDenialKind::MalformedRequest,
                )
            })?;
            let key = BankIdempotencyKey::new(idempotency_key).map_err(|_| {
                reject(
                    Some(request_id.clone()),
                    BankHttpDenialKind::MalformedRequest,
                )
            })?;
            let history = NonZeroUsize::new(controls.maximum_history_work).ok_or_else(|| {
                reject(
                    Some(request_id.clone()),
                    BankHttpDenialKind::MalformedRequest,
                )
            })?;
            (
                protocol,
                request_id,
                credential,
                custody_id,
                sequence,
                controls.deadline_milliseconds,
                MoneyCommand::Create {
                    input,
                    key,
                    history,
                },
            )
        }
        BankHttpMutationRequest::Resume {
            protocol,
            request_id,
            credential,
            custody_id,
            sequence,
            controls,
        } => {
            let history = NonZeroUsize::new(controls.maximum_history_work).ok_or_else(|| {
                reject(
                    Some(request_id.clone()),
                    BankHttpDenialKind::MalformedRequest,
                )
            })?;
            (
                protocol,
                request_id,
                credential,
                custody_id,
                sequence,
                controls.deadline_milliseconds,
                MoneyCommand::Resume { history },
            )
        }
        BankHttpMutationRequest::Read {
            protocol,
            request_id,
            credential,
            custody_id,
            sequence,
            controls,
        } => {
            if controls.maximum_results == 0 {
                return Err(reject(
                    Some(request_id),
                    BankHttpDenialKind::MalformedRequest,
                ));
            }
            (
                protocol,
                request_id,
                credential,
                custody_id,
                sequence,
                controls.deadline_milliseconds,
                MoneyCommand::Read {
                    maximum_results: controls.maximum_results,
                },
            )
        }
        BankHttpMutationRequest::ResumeSource {
            protocol,
            request_id,
            credential,
            custody_id,
            sequence,
            controls,
        } => (
            protocol,
            request_id,
            credential,
            custody_id,
            sequence,
            controls.deadline_milliseconds,
            MoneyCommand::ResumeSource,
        ),
        BankHttpMutationRequest::ReleaseCleanup {
            protocol,
            request_id,
            credential,
            custody_id,
            sequence,
            controls,
        } => (
            protocol,
            request_id,
            credential,
            custody_id,
            sequence,
            controls.deadline_milliseconds,
            MoneyCommand::ReleaseCleanup,
        ),
        BankHttpMutationRequest::Close {
            protocol,
            request_id,
            credential,
            custody_id,
            sequence,
            controls,
        } => (
            protocol,
            request_id,
            credential,
            custody_id,
            sequence,
            controls.deadline_milliseconds,
            MoneyCommand::Close,
        ),
    };
    let basis = super::request_admission::UnadmittedBankHttpControls {
        protocol,
        request_id,
        deadline_milliseconds: deadline,
    }
    .admit(maximum_deadline)
    .map_err(|error| BankHttpMutationOutcome::Rejected {
        request_id: error.request_id,
        denial: error.denial,
    })?;
    if custody_id.is_empty()
        || custody_id.len() > 128
        || custody_id.chars().any(char::is_control)
        || sequence == 0
    {
        return Err(reject(
            Some(basis.request_id),
            BankHttpDenialKind::MalformedRequest,
        ));
    }
    Ok(AdmittedMoneyCommand {
        request_id: basis.request_id,
        credential,
        custody_id,
        sequence,
        deadline: basis.deadline,
        command,
    })
}
pub(super) fn reject(
    request_id: Option<String>,
    kind: BankHttpDenialKind,
) -> BankHttpMutationOutcome {
    BankHttpMutationOutcome::Rejected {
        request_id,
        denial: denial(kind),
    }
}
