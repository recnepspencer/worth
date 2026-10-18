//! Transfer original money meaning through each exact installed Bank action.
use super::super::protocol::{BankHttpDenial, BankHttpDenialKind};
use super::{
    mutation_application::AdmittedBankHttpMutation, mutation_custody::*, mutation_refusal::denial,
};
use bank_domain::model::AccountId;
use bank_server::{
    mutations, BankAuthenticatedPrincipal, BankIdentityRuntime, BankMutationControls,
};
pub(super) fn construct(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    input: AdmittedBankHttpMutation,
    controls: BankMutationControls,
) -> Result<(AccountId, MoneyCustody), BankHttpDenial> {
    let constructed = match input {
        AdmittedBankHttpMutation::Deposit(input) => {
            let account = input.account;
            match runtime
                .mutate(mutations::deposit(input))
                .as_principal(principal)
                .controls(controls)
                .retain_operation()
            {
                Ok(operation) => (
                    account,
                    MoneyCustody::Deposit(DepositCustody::Active(operation)),
                ),
                Err(refusal) => return Err(super::construction_denial::construction(refusal)),
            }
        }
        AdmittedBankHttpMutation::Withdraw(input) => {
            let account = input.account;
            match runtime
                .mutate(mutations::withdraw(input))
                .as_principal(principal)
                .controls(controls)
                .retain_operation()
            {
                Ok(operation) => (
                    account,
                    MoneyCustody::Withdraw(WithdrawalCustody::Active(operation)),
                ),
                Err(refusal) => return Err(super::construction_denial::construction(refusal)),
            }
        }
        AdmittedBankHttpMutation::SendMoney(input, expected_revision, expected_status) => {
            let account = input.from;
            let mut original = runtime
                .mutate(mutations::send_money(input))
                .as_principal(principal);
            if let Some(expected) = expected_revision {
                original = match original.expect_version(
                    bank_domain::schema::AccountingRevision::reference(),
                    bank_domain::model::AccountJournalRevision::from_posting_count(expected),
                ) {
                    Ok(original) => original,
                    Err(_) => return Err(denial(BankHttpDenialKind::MalformedRequest)),
                };
            }
            if let Some(expected) = expected_status {
                original = match original
                    .expect_fact(bank_domain::schema::Status::reference(), expected.into())
                {
                    Ok(original) => original,
                    Err(_) => return Err(denial(BankHttpDenialKind::MalformedRequest)),
                };
            }
            match original.controls(controls).retain_operation() {
                Ok(operation) => (
                    account,
                    MoneyCustody::SendMoney(SendMoneyCustody::Active(operation)),
                ),
                Err(refusal) => return Err(super::construction_denial::construction(refusal)),
            }
        }
    };
    Ok(constructed)
}
