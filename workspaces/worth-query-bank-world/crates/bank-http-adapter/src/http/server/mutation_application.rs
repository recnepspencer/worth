//! Money input decoding and native denial projection; no operation executor.
use super::super::protocol::{BankHttpDenial, BankHttpDenialKind, BankHttpNextAction};
use bank_domain::model::{AccountId, BankPrincipalId, InstitutionId, Money, USD};
use bank_domain::schema::{Deposit, SendMoney, Withdraw};
mod commit_denial;
pub(super) use commit_denial::{commit_denial, execution_resource};
mod denial;
pub(super) use denial::{
    pending_execution_denial, pending_execution_stop, request_mutation_denial,
};
pub(super) enum AdmittedBankHttpMutation {
    Deposit(Deposit),
    Withdraw(Withdraw),
    SendMoney(
        SendMoney,
        Option<u64>,
        Option<super::super::protocol::BankHttpExpectedAccountStatus>,
    ),
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
    Some(AdmittedBankHttpMutation::SendMoney(
        SendMoney {
            from: AccountId::parse_canonical_text(from)?,
            recipient: BankPrincipalId::parse_canonical_text(recipient)?,
            amount: Money::<USD>::from_minor(amount_minor_units).ok()?,
        },
        None,
        None,
    ))
}
