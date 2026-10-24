//! Transfer original money meaning through each exact installed Bank action.
use super::super::protocol::{BankHttpDenial, BankHttpDenialKind};
use super::{
    mutation_application::AdmittedBankHttpMutation, mutation_custody::*, mutation_refusal::denial,
};
use bank_server::{
    mutations, BankAuthenticatedPrincipal, BankIdentityRuntime, BankMutationControls,
};
mod capability_revocation;
pub(super) mod correction;
pub(super) fn start(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    input: AdmittedBankHttpMutation,
    controls: BankMutationControls,
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        '_,
        '_,
        '_,
        bank_domain::schema::BankSchema,
    >,
    history: std::num::NonZeroUsize,
    history_page: Option<
        &worth_query_host::facade::primary_graph::WorthQueryCorrectionAlternativesPage,
    >,
) -> Result<(SourceMutationCustody, Option<BankHttpDenial>), BankHttpDenial> {
    let constructed = match input {
        AdmittedBankHttpMutation::ReapplySendMoney { source, .. } => {
            return correction::reapplication::start(
                runtime,
                principal,
                &source,
                controls,
                history,
                history_page.ok_or_else(|| denial(BankHttpDenialKind::Stale))?,
            );
        }
        AdmittedBankHttpMutation::RevokeEstateCapability(action) => {
            return capability_revocation::start(runtime, principal, action, controls, request);
        }
        AdmittedBankHttpMutation::CorrectEstateFreeze(source) => {
            return correction::estate::start(runtime, principal, &source, controls, history);
        }
        AdmittedBankHttpMutation::FreezeEstateAccount(input) => {
            match runtime
                .mutate(mutations::freeze_estate_account(input))
                .as_principal(principal)
                .controls(controls)
                .retain_operation()
            {
                Ok(operation) => {
                    SourceMutationCustody::EstateFreeze(EstateFreezeCustody::Active(operation))
                }
                Err(refusal) => return Err(super::construction_denial::construction(refusal)),
            }
        }
        AdmittedBankHttpMutation::CorrectSendMoney(source) => {
            return correction::start(runtime, principal, &source, controls, history);
        }
        AdmittedBankHttpMutation::Deposit(input) => {
            match runtime
                .mutate(mutations::deposit(input))
                .as_principal(principal)
                .controls(controls)
                .retain_operation()
            {
                Ok(operation) => SourceMutationCustody::Deposit(DepositCustody::Active(operation)),
                Err(refusal) => return Err(super::construction_denial::construction(refusal)),
            }
        }
        AdmittedBankHttpMutation::Withdraw(input) => {
            match runtime
                .mutate(mutations::withdraw(input))
                .as_principal(principal)
                .controls(controls)
                .retain_operation()
            {
                Ok(operation) => {
                    SourceMutationCustody::Withdraw(WithdrawalCustody::Active(operation))
                }
                Err(refusal) => return Err(super::construction_denial::construction(refusal)),
            }
        }
        AdmittedBankHttpMutation::SendMoney(input, expected_revision, expected_status) => {
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
                Ok(operation) => {
                    SourceMutationCustody::SendMoney(SendMoneyCustody::Active(operation))
                }
                Err(refusal) => return Err(super::construction_denial::construction(refusal)),
            }
        }
    };
    Ok(constructed.advance(request, history))
}
