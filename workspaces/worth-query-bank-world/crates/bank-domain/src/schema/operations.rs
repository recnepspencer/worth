use worth_query_decl::facade::worth_query_structured_value_binding;
use worth_query_decl::facade::{
    worth_query_operation, worth_query_operation_creates, worth_query_operation_deletes,
    worth_query_operation_emits, worth_query_operation_expects_fact,
    worth_query_operation_expects_version, worth_query_operation_links,
    worth_query_operation_unlinks, worth_query_operation_writes,
};

mod read_capabilities;

use crate::model::{
    AccountAuthorizationId, AccountId, AccountName, BankPrincipalId, BusinessId, CustomerRole,
    InstitutionId, JournalEntryId, Money, PaymentId, USD,
};

use super::authentication::PrincipalIdentityField;
use super::entities::{
    Account, AccountAuthorization, Approval, JournalEntry, PaymentIntent, Posting,
};
use super::fields::{
    AccountAuthorizationIdentity, AccountDisplayName, AccountIdentity, AccountingRevision,
    AuthorizationRole, BusinessIdentityField, InstitutionIdentityField, JournalIdentityField,
    JournalPurpose, Kind, PaymentAmount, PaymentIdentityField, PaymentStatusField,
    PostingAccountSequence, PostingAmount, PostingIdentityField, Purpose, Status,
};
use super::governance::AccountActivityEffect;
use super::relations::{
    AccountAuthorizedUser, ApprovalPrincipal, AuthorizationAccount, BusinessAccount,
    InstitutionAccount, InstitutionCashAccount, JournalPosting, JournalReversal, PaymentApproval,
    PaymentBusiness, PaymentDestination, PaymentInitiator, PaymentSource, PersonalOwner,
    PostingAccount,
};
use super::BankSchema;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatePersonalAccount {
    pub institution: InstitutionId,
    pub owner: BankPrincipalId,
    pub display_name: AccountName,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateBusinessAccount {
    pub institution: InstitutionId,
    pub business: BusinessId,
    pub display_name: AccountName,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyOpeningFunding {
    pub institution: InstitutionId,
    pub account: AccountId,
    pub amount: Money<USD>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Deposit {
    pub institution: InstitutionId,
    pub account: AccountId,
    pub amount: Money<USD>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Withdraw {
    pub institution: InstitutionId,
    pub account: AccountId,
    pub amount: Money<USD>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SendMoney {
    pub from: AccountId,
    pub recipient: BankPrincipalId,
    pub amount: Money<USD>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitiateBusinessPayment {
    pub business: BusinessId,
    pub from: AccountId,
    pub recipient: BankPrincipalId,
    pub amount: Money<USD>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovePayment {
    pub payment: PaymentId,
    pub approver: BankPrincipalId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectPayment {
    pub payment: PaymentId,
    pub rejecting_principal: BankPrincipalId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrantAccountAuthorization {
    pub account: AccountId,
    pub principal: BankPrincipalId,
    pub role: CustomerRole,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevokeAccountAuthorization {
    pub account: AccountId,
    pub authorization: AccountAuthorizationId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReversalReason {
    Duplicate,
    OperatorCorrection,
    ExternalReturn,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReverseJournal {
    pub institution: InstitutionId,
    pub journal: JournalEntryId,
    pub reason: ReversalReason,
}

worth_query_structured_value_binding!(pub CreatePersonalAccountInputBinding for CreatePersonalAccount { identity: "bank.operation.create-personal-account.input.v1" });
worth_query_structured_value_binding!(pub CreateBusinessAccountInputBinding for CreateBusinessAccount { identity: "bank.operation.create-business-account.input.v1" });
worth_query_structured_value_binding!(pub ApplyOpeningFundingInputBinding for ApplyOpeningFunding { identity: "bank.operation.apply-opening-funding.input.v1" });
worth_query_structured_value_binding!(pub DepositInputBinding for Deposit { identity: "bank.operation.deposit.input.v1" });
worth_query_structured_value_binding!(pub WithdrawInputBinding for Withdraw { identity: "bank.operation.withdraw.input.v1" });
worth_query_structured_value_binding!(pub SendMoneyInputBinding for SendMoney { identity: "bank.operation.send-money.input.v1" });
worth_query_structured_value_binding!(pub InitiateBusinessPaymentInputBinding for InitiateBusinessPayment { identity: "bank.operation.initiate-business-payment.input.v1" });
worth_query_structured_value_binding!(pub ApprovePaymentInputBinding for ApprovePayment { identity: "bank.operation.approve-payment.input.v1" });
worth_query_structured_value_binding!(pub RejectPaymentInputBinding for RejectPayment { identity: "bank.operation.reject-payment.input.v1" });
worth_query_structured_value_binding!(pub GrantAccountAuthorizationInputBinding for GrantAccountAuthorization { identity: "bank.operation.grant-account-authorization.input.v1" });
worth_query_structured_value_binding!(pub RevokeAccountAuthorizationInputBinding for RevokeAccountAuthorization { identity: "bank.operation.revoke-account-authorization.input.v1" });
worth_query_structured_value_binding!(pub ReverseJournalInputBinding for ReverseJournal { identity: "bank.operation.reverse-journal.input.v1" });

worth_query_operation!(pub CreatePersonalAccountOperation for BankSchema, input CreatePersonalAccountInputBinding);
worth_query_operation!(pub CreateBusinessAccountOperation for BankSchema, input CreateBusinessAccountInputBinding);
worth_query_operation!(pub ApplyOpeningFundingOperation for BankSchema, input ApplyOpeningFundingInputBinding);
worth_query_operation!(pub DepositOperation for BankSchema, input DepositInputBinding);
worth_query_operation!(pub WithdrawOperation for BankSchema, input WithdrawInputBinding);
worth_query_operation!(pub SendMoneyOperation for BankSchema, input SendMoneyInputBinding);
worth_query_operation_expects_version!(SendMoneyOperation => [AccountingRevision]);
worth_query_operation_expects_fact!(SendMoneyOperation => [Status]);
worth_query_operation!(pub InitiateBusinessPaymentOperation for BankSchema, input InitiateBusinessPaymentInputBinding);
worth_query_operation!(pub ApprovePaymentOperation for BankSchema, input ApprovePaymentInputBinding);
worth_query_operation!(pub RejectPaymentOperation for BankSchema, input RejectPaymentInputBinding);
worth_query_operation!(pub GrantAccountAuthorizationOperation for BankSchema, input GrantAccountAuthorizationInputBinding);
worth_query_operation!(pub RevokeAccountAuthorizationOperation for BankSchema, input RevokeAccountAuthorizationInputBinding);
worth_query_operation!(pub ReverseJournalOperation for BankSchema, input ReverseJournalInputBinding);
worth_query_operation_writes!(
    CreatePersonalAccountOperation => [
        AccountIdentity,
        AccountDisplayName,
        AccountingRevision,
        Kind,
        Status
    ]
);
worth_query_operation_writes!(
    CreateBusinessAccountOperation => [
        AccountIdentity,
        AccountDisplayName,
        AccountingRevision,
        Kind,
        Status
    ]
);
worth_query_operation_creates!(CreatePersonalAccountOperation => [Account]);
worth_query_operation_creates!(CreateBusinessAccountOperation => [Account]);
worth_query_operation_links!(
    CreatePersonalAccountOperation => [PersonalOwner, InstitutionAccount]
);
worth_query_operation_links!(
    CreateBusinessAccountOperation => [BusinessAccount, InstitutionAccount]
);

worth_query_operation_writes!(
    ApplyOpeningFundingOperation => [
        JournalIdentityField,
        JournalPurpose,
        PostingIdentityField,
        PostingAmount,
        PostingAccountSequence,
        Purpose,
        AccountingRevision
    ]
);
worth_query_operation_writes!(
    DepositOperation => [
        JournalIdentityField,
        JournalPurpose,
        PostingIdentityField,
        PostingAmount,
        PostingAccountSequence,
        Purpose,
        AccountingRevision
    ]
);
worth_query_operation_writes!(
    WithdrawOperation => [
        JournalIdentityField,
        JournalPurpose,
        PostingIdentityField,
        PostingAmount,
        PostingAccountSequence,
        Purpose,
        AccountingRevision
    ]
);
worth_query_operation_writes!(
    SendMoneyOperation => [
        JournalIdentityField,
        JournalPurpose,
        PostingIdentityField,
        PostingAmount,
        PostingAccountSequence,
        Purpose,
        AccountingRevision
    ]
);
worth_query_operation_writes!(
    ReverseJournalOperation => [
        JournalIdentityField,
        JournalPurpose,
        PostingIdentityField,
        PostingAmount,
        PostingAccountSequence,
        Purpose,
        AccountingRevision
    ]
);
worth_query_operation_creates!(ApplyOpeningFundingOperation => [JournalEntry, Posting]);
worth_query_operation_creates!(DepositOperation => [JournalEntry, Posting]);
worth_query_operation_creates!(WithdrawOperation => [JournalEntry, Posting]);
worth_query_operation_creates!(SendMoneyOperation => [JournalEntry, Posting]);
worth_query_operation_creates!(ReverseJournalOperation => [JournalEntry, Posting]);
worth_query_operation_links!(
    ApplyOpeningFundingOperation => [JournalPosting, PostingAccount]
);
worth_query_operation_links!(DepositOperation => [JournalPosting, PostingAccount]);
worth_query_operation_links!(WithdrawOperation => [JournalPosting, PostingAccount]);
worth_query_operation_links!(SendMoneyOperation => [JournalPosting, PostingAccount]);
worth_query_operation_links!(ReverseJournalOperation => [JournalPosting, PostingAccount]);
worth_query_operation_links!(ReverseJournalOperation => [JournalReversal]);

worth_query_operation_creates!(InitiateBusinessPaymentOperation => [PaymentIntent]);
worth_query_operation_writes!(
    InitiateBusinessPaymentOperation => [PaymentIdentityField, PaymentAmount, PaymentStatusField]
);
worth_query_operation_links!(
    InitiateBusinessPaymentOperation => [
        PaymentSource,
        PaymentDestination,
        PaymentBusiness,
        PaymentInitiator
    ]
);
worth_query_operation_creates!(ApprovePaymentOperation => [Approval, JournalEntry, Posting]);
worth_query_operation_links!(
    ApprovePaymentOperation => [
        PaymentApproval,
        ApprovalPrincipal,
        JournalPosting,
        PostingAccount
    ]
);
worth_query_operation_writes!(
    ApprovePaymentOperation => [
        PaymentStatusField,
        AccountingRevision,
        JournalIdentityField,
        JournalPurpose,
        PostingIdentityField,
        PostingAmount,
        PostingAccountSequence,
        Purpose
    ]
);
worth_query_operation_creates!(RejectPaymentOperation => [Approval]);
worth_query_operation_links!(
    RejectPaymentOperation => [PaymentApproval, ApprovalPrincipal]
);
worth_query_operation_writes!(RejectPaymentOperation => [PaymentStatusField]);
worth_query_operation_creates!(
    GrantAccountAuthorizationOperation => [AccountAuthorization]
);
worth_query_operation_writes!(
    GrantAccountAuthorizationOperation => [AccountAuthorizationIdentity, AuthorizationRole]
);
worth_query_operation_links!(
    GrantAccountAuthorizationOperation => [AccountAuthorizedUser, AuthorizationAccount]
);
worth_query_operation_unlinks!(
    RevokeAccountAuthorizationOperation => [AccountAuthorizedUser, AuthorizationAccount]
);
worth_query_operation_deletes!(
    RevokeAccountAuthorizationOperation => [AccountAuthorization]
);

worth_query_operation_emits!(ApplyOpeningFundingOperation => [AccountActivityEffect]);
worth_query_operation_emits!(DepositOperation => [AccountActivityEffect]);
worth_query_operation_emits!(WithdrawOperation => [AccountActivityEffect]);
worth_query_operation_emits!(SendMoneyOperation => [AccountActivityEffect]);
worth_query_operation_emits!(ApprovePaymentOperation => [AccountActivityEffect]);
worth_query_operation_emits!(ReverseJournalOperation => [AccountActivityEffect]);
