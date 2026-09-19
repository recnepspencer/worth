use worth_query_decl::facade::{
    application_operation::NoApplicationMutationOutputs,
    application_schema::{NoApplicationUnit, ReadOnly},
    worth_query_mutation_binding, worth_query_structured_value_binding,
};

use crate::model::{AccountId, BankPrincipalId, InstitutionId, JournalEntryId};
use crate::proposals::{
    BankIdempotencyKey, BankInvariantApprovedProposal, BankProposalDenial, CanonicalProposalPayload,
};

use super::create_personal_account_binding::client_key_identity;
use super::{
    ApplyOpeningFunding, ApplyOpeningFundingInputBinding, Deposit, DepositInputBinding, SendMoney,
    SendMoneyInputBinding, Withdraw, WithdrawInputBinding,
};
use crate::schema::{
    Account, AccountIdentity, ApplyOpeningFundingOperation, BankPrincipalBinding,
    BankPrincipalIdBinding, BankSchema, DepositOperation, ExternalPrincipalMapping, Identity,
    Institution, InstitutionIdentity, InstitutionIdentityField, Principal, SendMoneyOperation,
    WithdrawOperation,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MoneyMovementResult {
    pub journal: JournalEntryId,
}

worth_query_structured_value_binding!(
    pub MoneyMovementResultBinding for MoneyMovementResult {
        identity: "bank.operation.money-movement.result.v1"
    }
);

worth_query_structured_value_binding!(
    pub MoneyMovementDenialBinding for BankProposalDenial {
        identity: "bank.operation.money-movement.denial.v1"
    }
);

fn opening_scope(input: &ApplyOpeningFunding) -> InstitutionId {
    input.institution
}

fn deposit_scope(input: &Deposit) -> InstitutionId {
    input.institution
}

fn withdrawal_scope(input: &Withdraw) -> InstitutionId {
    input.institution
}

fn send_scope(input: &SendMoney) -> AccountId {
    input.from
}

fn opening_identity(input: &ApplyOpeningFunding) -> [u8; 32] {
    institution_movement_identity(
        "application-opening-funding",
        input.institution,
        input.account,
        input.amount.minor_units(),
    )
}

fn deposit_identity(input: &Deposit) -> [u8; 32] {
    institution_movement_identity(
        "application-deposit",
        input.institution,
        input.account,
        input.amount.minor_units(),
    )
}

fn withdrawal_identity(input: &Withdraw) -> [u8; 32] {
    institution_movement_identity(
        "application-withdrawal",
        input.institution,
        input.account,
        input.amount.minor_units(),
    )
}

fn institution_movement_identity(
    operation: &'static str,
    institution: InstitutionId,
    account: AccountId,
    amount: i64,
) -> [u8; 32] {
    *CanonicalProposalPayload::new(operation)
        .u64("institution", institution.get())
        .text("account", &account.canonical_text())
        .i64("amount-minor-units", amount)
        .derive_identity()
        .bytes()
}

fn send_identity(input: &SendMoney) -> [u8; 32] {
    *CanonicalProposalPayload::new("application-send-money")
        .text("source", &input.from.canonical_text())
        .u64("recipient", input.recipient.get())
        .i64("amount-minor-units", input.amount.minor_units())
        .derive_identity()
        .bytes()
}

macro_rules! institution_movement_binding {
    ($Binding:ident, $Input:ty, $InputBinding:ty, $Operation:ty, $identity:literal, $handler:literal, $scope:ident, $input_identity:ident) => {
        worth_query_mutation_binding!(
            pub $Binding for $Input, schema BankSchema,
            identity $identity,
            input $InputBinding,
            operation $Operation,
            result MoneyMovementResultBinding,
            idempotency BankIdempotencyKey,
                identity "bank.application-mutation-client-key.v1",
                key_identity client_key_identity,
                input_identity $input_identity,
            decision BankInvariantApprovedProposal,
            denial MoneyMovementDenialBinding,
            handler identity $handler,
            program required,
            outputs NoApplicationMutationOutputs,
            principal BankPrincipalBinding,
                mapping ExternalPrincipalMapping,
                principal_entity Principal,
                principal_identity BankPrincipalId,
                identity_binding BankPrincipalIdBinding,
            scope Institution,
                InstitutionIdentity,
                InstitutionIdentityField,
                InstitutionId,
                ReadOnly,
                NoApplicationUnit,
            field InstitutionIdentityField::reference(),
            value $scope,
            candidates creates 3, deletes 0, links 4, unlinks 0, writes 12, emits 2,
            resources retained_representation_bytes 32768, validator_work 16651
        );
    };
}

institution_movement_binding!(
    ApplyOpeningFundingMutationBinding,
    ApplyOpeningFunding,
    ApplyOpeningFundingInputBinding,
    ApplyOpeningFundingOperation,
    "bank.operation.apply-opening-funding.mutation-binding.v1",
    "bank.operation.apply-opening-funding.handler.v1",
    opening_scope,
    opening_identity
);
institution_movement_binding!(
    DepositMutationBinding,
    Deposit,
    DepositInputBinding,
    DepositOperation,
    "bank.operation.deposit.mutation-binding.v1",
    "bank.operation.deposit.handler.v1",
    deposit_scope,
    deposit_identity
);
institution_movement_binding!(
    WithdrawMutationBinding,
    Withdraw,
    WithdrawInputBinding,
    WithdrawOperation,
    "bank.operation.withdraw.mutation-binding.v1",
    "bank.operation.withdraw.handler.v1",
    withdrawal_scope,
    withdrawal_identity
);

worth_query_mutation_binding!(
    pub SendMoneyMutationBinding for SendMoney, schema BankSchema,
    identity "bank.operation.send-money.mutation-binding.v1",
    input SendMoneyInputBinding,
    operation SendMoneyOperation,
    result MoneyMovementResultBinding,
    idempotency BankIdempotencyKey,
        identity "bank.application-mutation-client-key.v1",
        key_identity client_key_identity,
        input_identity send_identity,
    decision BankInvariantApprovedProposal,
    denial MoneyMovementDenialBinding,
    handler identity "bank.operation.send-money.handler.v1",
    program required,
    outputs NoApplicationMutationOutputs,
    principal BankPrincipalBinding,
        mapping ExternalPrincipalMapping,
        principal_entity Principal,
        principal_identity BankPrincipalId,
        identity_binding BankPrincipalIdBinding,
    scope Account,
        Identity,
        AccountIdentity,
        AccountId,
        ReadOnly,
        NoApplicationUnit,
    field AccountIdentity::reference(),
    value send_scope,
    candidates creates 3, deletes 0, links 4, unlinks 0, writes 12, emits 2,
    resources retained_representation_bytes 32768, validator_work 16651
);
