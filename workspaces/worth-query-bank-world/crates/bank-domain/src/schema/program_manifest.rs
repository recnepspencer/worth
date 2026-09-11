use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use super::entities::{
    Account, AccountAuthorization, Approval, JournalEntry, PaymentIntent, Posting,
};
use super::fields::{
    AccountAuthorizationIdentity, AccountDisplayName, AccountIdentity, AccountingRevision,
    AuthorizationRole, Kind, PaymentAmount, PaymentIdentityField, PaymentStatusField, Status,
};
use super::money_movement_program::MoneyMovementProgram;
use super::operations::*;
use super::relations::{
    AccountAuthorizedUser, ApprovalPrincipal, AuthorizationAccount, BusinessAccount,
    InstitutionAccount, JournalReversal, PaymentApproval, PaymentBusiness, PaymentDestination,
    PaymentInitiator, PaymentSource, PersonalOwner,
};
use super::BankSchema;

pub(super) fn install_account_creation_program(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .operation_create(
            CreatePersonalAccountOperation::reference(),
            Account::reference(),
        )
        .operation_write(
            CreatePersonalAccountOperation::reference(),
            AccountIdentity::reference(),
        )
        .operation_write(
            CreatePersonalAccountOperation::reference(),
            AccountDisplayName::reference(),
        )
        .operation_write(
            CreatePersonalAccountOperation::reference(),
            AccountingRevision::reference(),
        )
        .operation_write(
            CreatePersonalAccountOperation::reference(),
            Kind::reference(),
        )
        .operation_write(
            CreatePersonalAccountOperation::reference(),
            Status::reference(),
        )
        .operation_link(
            CreatePersonalAccountOperation::reference(),
            PersonalOwner::reference(),
        )
        .operation_link(
            CreatePersonalAccountOperation::reference(),
            InstitutionAccount::reference(),
        )
        .operation_create(
            CreateBusinessAccountOperation::reference(),
            Account::reference(),
        )
        .operation_write(
            CreateBusinessAccountOperation::reference(),
            AccountIdentity::reference(),
        )
        .operation_write(
            CreateBusinessAccountOperation::reference(),
            AccountDisplayName::reference(),
        )
        .operation_write(
            CreateBusinessAccountOperation::reference(),
            AccountingRevision::reference(),
        )
        .operation_write(
            CreateBusinessAccountOperation::reference(),
            Kind::reference(),
        )
        .operation_write(
            CreateBusinessAccountOperation::reference(),
            Status::reference(),
        )
        .operation_link(
            CreateBusinessAccountOperation::reference(),
            BusinessAccount::reference(),
        )
        .operation_link(
            CreateBusinessAccountOperation::reference(),
            InstitutionAccount::reference(),
        )
}

pub(super) fn install_money_programs(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .operation_create(
            ApplyOpeningFundingOperation::reference(),
            JournalEntry::reference(),
        )
        .operation_create(
            ApplyOpeningFundingOperation::reference(),
            Posting::reference(),
        )
        .money_movement_program(ApplyOpeningFundingOperation::reference())
        .operation_create(DepositOperation::reference(), JournalEntry::reference())
        .operation_create(DepositOperation::reference(), Posting::reference())
        .money_movement_program(DepositOperation::reference())
        .operation_create(WithdrawOperation::reference(), JournalEntry::reference())
        .operation_create(WithdrawOperation::reference(), Posting::reference())
        .money_movement_program(WithdrawOperation::reference())
        .operation_create(SendMoneyOperation::reference(), JournalEntry::reference())
        .operation_create(SendMoneyOperation::reference(), Posting::reference())
        .money_movement_program(SendMoneyOperation::reference())
        .operation_create(
            ReverseJournalOperation::reference(),
            JournalEntry::reference(),
        )
        .operation_create(ReverseJournalOperation::reference(), Posting::reference())
        .money_movement_program(ReverseJournalOperation::reference())
        .operation_link(
            ReverseJournalOperation::reference(),
            JournalReversal::reference(),
        )
}

pub(super) fn install_payment_program(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .operation_create(
            InitiateBusinessPaymentOperation::reference(),
            PaymentIntent::reference(),
        )
        .operation_link(
            InitiateBusinessPaymentOperation::reference(),
            PaymentSource::reference(),
        )
        .operation_link(
            InitiateBusinessPaymentOperation::reference(),
            PaymentDestination::reference(),
        )
        .operation_link(
            InitiateBusinessPaymentOperation::reference(),
            PaymentInitiator::reference(),
        )
        .operation_link(
            InitiateBusinessPaymentOperation::reference(),
            PaymentBusiness::reference(),
        )
        .operation_write(
            InitiateBusinessPaymentOperation::reference(),
            PaymentIdentityField::reference(),
        )
        .operation_write(
            InitiateBusinessPaymentOperation::reference(),
            PaymentAmount::reference(),
        )
        .operation_write(
            InitiateBusinessPaymentOperation::reference(),
            PaymentStatusField::reference(),
        )
        .operation_create(ApprovePaymentOperation::reference(), Approval::reference())
        .operation_create(
            ApprovePaymentOperation::reference(),
            JournalEntry::reference(),
        )
        .operation_create(ApprovePaymentOperation::reference(), Posting::reference())
        .operation_link(
            ApprovePaymentOperation::reference(),
            PaymentApproval::reference(),
        )
        .operation_link(
            ApprovePaymentOperation::reference(),
            ApprovalPrincipal::reference(),
        )
        .operation_write(
            ApprovePaymentOperation::reference(),
            PaymentStatusField::reference(),
        )
        .money_movement_program(ApprovePaymentOperation::reference())
        .operation_write(
            RejectPaymentOperation::reference(),
            PaymentStatusField::reference(),
        )
        .operation_create(RejectPaymentOperation::reference(), Approval::reference())
        .operation_link(
            RejectPaymentOperation::reference(),
            PaymentApproval::reference(),
        )
        .operation_link(
            RejectPaymentOperation::reference(),
            ApprovalPrincipal::reference(),
        )
}

pub(super) fn install_authorization_program(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .operation_create(
            GrantAccountAuthorizationOperation::reference(),
            AccountAuthorization::reference(),
        )
        .operation_write(
            GrantAccountAuthorizationOperation::reference(),
            AccountAuthorizationIdentity::reference(),
        )
        .operation_write(
            GrantAccountAuthorizationOperation::reference(),
            AuthorizationRole::reference(),
        )
        .operation_link(
            GrantAccountAuthorizationOperation::reference(),
            AccountAuthorizedUser::reference(),
        )
        .operation_link(
            GrantAccountAuthorizationOperation::reference(),
            AuthorizationAccount::reference(),
        )
        .operation_unlink(
            RevokeAccountAuthorizationOperation::reference(),
            AccountAuthorizedUser::reference(),
        )
        .operation_unlink(
            RevokeAccountAuthorizationOperation::reference(),
            AuthorizationAccount::reference(),
        )
        .operation_delete(
            RevokeAccountAuthorizationOperation::reference(),
            AccountAuthorization::reference(),
        )
}
