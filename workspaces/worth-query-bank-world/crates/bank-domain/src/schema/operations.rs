use worth_query_decl::facade::worth_query_operation;
use worth_query_decl::facade::worth_query_structured_value_binding;

mod account_access_bindings;
mod approved_payment_outputs;
mod create_business_account_binding;
mod create_personal_account_binding;
mod initiate_business_payment_binding;
mod money_movement_bindings;
mod payment_decision_bindings;
mod reverse_journal_binding;
mod send_money_access;
mod send_money_outputs;

pub use account_access_bindings::*;
pub use approved_payment_outputs::*;
pub use create_business_account_binding::*;
pub use create_personal_account_binding::*;
pub use initiate_business_payment_binding::*;
pub use money_movement_bindings::*;
pub use payment_decision_bindings::*;
pub use reverse_journal_binding::*;
pub use send_money_outputs::*;

use crate::model::{
    AccountAuthorizationId, AccountId, AccountName, BankPrincipalId, BusinessId, CustomerRole,
    InstitutionId, JournalEntryId, Money, PaymentId, USD,
};

use super::entities::{
    Account, AccountAuthorization, Approval, JournalEntry, PaymentIntent, Posting,
};
use super::fields::{AccountingRevision, Status};
use super::governance::AccountActivityEffect;
use super::relations::{
    AccountAuthorizedUser, ApprovalPrincipal, AuthorizationAccount, BusinessAccount,
    InstitutionAccount, InstitutionCashAccount, JournalPosting, JournalReversal, PaymentApproval,
    PaymentBusiness, PaymentDestination, PaymentInitiator, PaymentSource, PersonalOwner,
    PostingAccount,
};
use super::workflow::ApprovedPaymentSettlementEffect;
use super::BankSchema;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct CreatePersonalAccount {
    pub institution: InstitutionId,
    pub owner: BankPrincipalId,
    pub display_name: AccountName,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct CreateBusinessAccount {
    pub institution: InstitutionId,
    pub business: BusinessId,
    pub display_name: AccountName,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct ApplyOpeningFunding {
    pub institution: InstitutionId,
    pub account: AccountId,
    pub amount: Money<USD>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct Deposit {
    pub institution: InstitutionId,
    pub account: AccountId,
    pub amount: Money<USD>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct Withdraw {
    pub institution: InstitutionId,
    pub account: AccountId,
    pub amount: Money<USD>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct SendMoney {
    pub from: AccountId,
    pub recipient: BankPrincipalId,
    pub amount: Money<USD>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct InitiateBusinessPayment {
    pub business: BusinessId,
    pub from: AccountId,
    pub recipient: BankPrincipalId,
    pub amount: Money<USD>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct ApprovePayment {
    pub payment: PaymentId,
    pub approver: BankPrincipalId,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RejectPayment {
    #[serde(deserialize_with = "super::correction_input::payment_identity")]
    pub payment: PaymentId,
    #[serde(deserialize_with = "super::correction_input::principal_identity")]
    pub rejecting_principal: BankPrincipalId,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct GrantAccountAuthorization {
    pub account: AccountId,
    pub principal: BankPrincipalId,
    pub role: CustomerRole,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct RevokeAccountAuthorization {
    pub account: AccountId,
    pub authorization: AccountAuthorizationId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum ReversalReason {
    Duplicate,
    OperatorCorrection,
    ExternalReturn,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
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

worth_query_operation!(pub CreatePersonalAccountOperation for BankSchema, input CreatePersonalAccountInputBinding;
    reads [PersonalOwner, InstitutionAccount, BusinessAccount];
    creates [Account];
    links [PersonalOwner, InstitutionAccount];
);

worth_query_operation!(pub CreateBusinessAccountOperation for BankSchema, input CreateBusinessAccountInputBinding;
    reads [BusinessAccount, InstitutionAccount, PersonalOwner];
    creates [Account];
    links [BusinessAccount, InstitutionAccount];
);

worth_query_operation!(pub ApplyOpeningFundingOperation for BankSchema, input ApplyOpeningFundingInputBinding;
    reads [InstitutionAccount, InstitutionCashAccount, PersonalOwner, BusinessAccount, PostingAccount, JournalPosting, JournalReversal];
    creates [JournalEntry, Posting];
    links [JournalPosting, PostingAccount];
    emits [AccountActivityEffect];
);

worth_query_operation!(pub DepositOperation for BankSchema, input DepositInputBinding;
    reads [InstitutionAccount, InstitutionCashAccount, PersonalOwner, BusinessAccount, PostingAccount, JournalPosting, JournalReversal];
    creates [JournalEntry, Posting];
    links [JournalPosting, PostingAccount];
    emits [AccountActivityEffect];
);

worth_query_operation!(pub WithdrawOperation for BankSchema, input WithdrawInputBinding;
    reads [InstitutionAccount, InstitutionCashAccount, PersonalOwner, BusinessAccount, PostingAccount, JournalPosting, JournalReversal];
    creates [JournalEntry, Posting];
    links [JournalPosting, PostingAccount];
    emits [AccountActivityEffect];
);

worth_query_operation!(pub SendMoneyOperation for BankSchema, input SendMoneyInputBinding;
    reads [PersonalOwner, InstitutionAccount, BusinessAccount, PostingAccount, JournalPosting, JournalReversal];
    creates [JournalEntry, Posting];
    links [JournalPosting, PostingAccount];
    emits [AccountActivityEffect];
    expects_version [AccountingRevision];
    expects_fact [Status];
);

worth_query_operation!(pub InitiateBusinessPaymentOperation for BankSchema, input InitiateBusinessPaymentInputBinding;
    reads [BusinessAccount, PersonalOwner, InstitutionAccount];
    creates [PaymentIntent];
    links [PaymentSource, PaymentDestination, PaymentBusiness, PaymentInitiator];
);

worth_query_operation!(pub ApprovePaymentOperation for BankSchema, input ApprovePaymentInputBinding;
    reads [PaymentSource, PaymentDestination, PaymentBusiness, PaymentInitiator, PaymentApproval, ApprovalPrincipal, InstitutionAccount, PersonalOwner, BusinessAccount, PostingAccount, JournalPosting, JournalReversal];
    creates [Approval, JournalEntry, Posting];
    links [PaymentApproval, ApprovalPrincipal, JournalPosting, PostingAccount];
    emits [AccountActivityEffect, ApprovedPaymentSettlementEffect];
);

worth_query_operation!(pub RejectPaymentOperation for BankSchema, input RejectPaymentInputBinding;
    reads [PaymentSource, PaymentDestination, PaymentBusiness, PaymentInitiator, PaymentApproval, ApprovalPrincipal, InstitutionAccount, PersonalOwner, BusinessAccount];
    creates [Approval];
    links [PaymentApproval, ApprovalPrincipal];
);

worth_query_operation!(pub GrantAccountAuthorizationOperation for BankSchema, input GrantAccountAuthorizationInputBinding;
    reads [AccountAuthorizedUser, AuthorizationAccount, InstitutionAccount, PersonalOwner, BusinessAccount];
    creates [AccountAuthorization];
    links [AccountAuthorizedUser, AuthorizationAccount];
);

worth_query_operation!(pub RevokeAccountAuthorizationOperation for BankSchema, input RevokeAccountAuthorizationInputBinding;
    reads [AccountAuthorization, AccountAuthorizedUser, AuthorizationAccount, InstitutionAccount, PersonalOwner, BusinessAccount];
    deletes [AccountAuthorization];
    unlinks [AccountAuthorizedUser, AuthorizationAccount];
);

worth_query_operation!(pub ReverseJournalOperation for BankSchema, input ReverseJournalInputBinding;
    reads [InstitutionAccount, PersonalOwner, BusinessAccount, JournalPosting, PostingAccount, JournalReversal];
    creates [JournalEntry, Posting];
    links [JournalPosting, PostingAccount, JournalReversal];
    emits [AccountActivityEffect];
);
