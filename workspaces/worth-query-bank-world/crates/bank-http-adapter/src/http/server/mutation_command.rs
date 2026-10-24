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
pub(super) enum MutationCommand {
    Create {
        input: AdmittedBankHttpMutation,
        key: BankIdempotencyKey,
        history: NonZeroUsize,
    },
    Resume {
        history: NonZeroUsize,
    },
    Read {
        account: bank_domain::model::AccountId,
        maximum_results: usize,
    },
    ResumeSource,
    ReleaseCleanup,
    Close,
}
pub(super) struct AdmittedMutationCommand {
    pub(super) request_id: String,
    pub(super) credential: BankHttpCredential,
    pub(super) custody_id: String,
    pub(super) sequence: u64,
    pub(super) deadline: Instant,
    pub(super) command: MutationCommand,
}
#[allow(
    clippy::result_large_err,
    reason = "Admission preserves the by-value protocol refusal without a wrapper allocation"
)]
pub(super) fn admit(
    request: BankHttpMutationRequest,
    maximum_deadline: Duration,
) -> Result<AdmittedMutationCommand, BankHttpMutationOutcome> {
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
                BankHttpMutationOperation::RevokeEstateCapability { estate, grant } => {
                    bank_domain::estate::EstateCaseId::new(estate)
                        .zip(bank_domain::estate::CapabilityGrantId::new(grant))
                        .map(|(estate, grant)| {
                            AdmittedBankHttpMutation::RevokeEstateCapability(
                                bank_domain::estate::EstateAction::RevokeCapability {
                                    estate,
                                    grant,
                                },
                            )
                        })
                }
                BankHttpMutationOperation::FreezeEstateAccount { estate, account } => {
                    bank_domain::estate::EstateCaseId::new(estate)
                        .zip(bank_domain::model::AccountId::parse_canonical_text(
                            &account,
                        ))
                        .map(|(estate, account)| {
                            AdmittedBankHttpMutation::FreezeEstateAccount(
                                bank_domain::schema::FreezeEstateAccount::new(estate, account),
                            )
                        })
                }
                BankHttpMutationOperation::CorrectEstateFreeze { source } => {
                    Some(AdmittedBankHttpMutation::CorrectEstateFreeze(source.into()))
                }
                BankHttpMutationOperation::CorrectSendMoney { source } => {
                    Some(AdmittedBankHttpMutation::CorrectSendMoney(source.into()))
                }
                BankHttpMutationOperation::ReapplySendMoney {
                    history_token,
                    source,
                } => super::continuation_registry::BankHttpContinuationRegistry::recognizes_token(
                    &history_token,
                )
                .then(|| AdmittedBankHttpMutation::ReapplySendMoney {
                    history_token,
                    source: source.into(),
                }),
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
                MutationCommand::Create {
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
                MutationCommand::Resume { history },
            )
        }
        BankHttpMutationRequest::Read {
            protocol,
            request_id,
            credential,
            custody_id,
            sequence,
            controls,
            account,
        } => {
            let account = bank_domain::model::AccountId::parse_canonical_text(&account)
                .ok_or_else(|| {
                    reject(
                        Some(request_id.clone()),
                        BankHttpDenialKind::MalformedRequest,
                    )
                })?;
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
                MutationCommand::Read {
                    account,
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
            MutationCommand::ResumeSource,
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
            MutationCommand::ReleaseCleanup,
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
            MutationCommand::Close,
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
    Ok(AdmittedMutationCommand {
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
