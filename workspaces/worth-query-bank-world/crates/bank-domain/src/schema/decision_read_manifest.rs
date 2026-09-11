use worth_query_decl::facade::application_schema::{
    ApplicationOperationRef, ApplicationSchemaDeclarationBuilder, OperationReads,
};

use super::authentication::PrincipalIdentityField;
use super::entities::AccountAuthorization;
use super::fields::{
    AccountAuthorizationIdentity, AccountDisplayName, AccountIdentity, AccountingRevision,
    AuthorizationRole, BusinessIdentityField, InstitutionIdentityField, JournalIdentityField,
    JournalPurpose, Kind, PostingAmount, PostingIdentityField, Purpose, Status,
};
use super::operations::*;
use super::relations::{
    AccountAuthorizedUser, AuthorizationAccount, BusinessAccount, InstitutionAccount,
    InstitutionCashAccount, JournalPosting, JournalReversal, PersonalOwner, PostingAccount,
};
use super::BankSchema;

mod account_projection;
mod payment_projection;

use account_projection::{install_account_projection_reads, install_accounting_projection_reads};
use payment_projection::install_payment_projection_reads;

pub(super) fn install_account_decision_reads(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let schema = install_account_creation_reads(schema);
    let schema = install_account_access_reads(schema);
    install_account_operation_budgets(schema)
}

pub(super) fn install_payment_decision_reads(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let schema = install_money_movement_reads(schema);
    let schema = install_payment_reads(schema);
    install_payment_operation_budgets(schema)
}

fn install_account_creation_reads(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .operation_read_field(
            CreatePersonalAccountOperation::reference(),
            InstitutionIdentityField::reference(),
        )
        .operation_read_field(
            CreatePersonalAccountOperation::reference(),
            PrincipalIdentityField::reference(),
        )
        .operation_read_relation(
            CreatePersonalAccountOperation::reference(),
            PersonalOwner::reference(),
        )
        .operation_read_field(
            CreateBusinessAccountOperation::reference(),
            InstitutionIdentityField::reference(),
        )
        .operation_read_field(
            CreateBusinessAccountOperation::reference(),
            BusinessIdentityField::reference(),
        )
        .operation_read_relation(
            CreateBusinessAccountOperation::reference(),
            BusinessAccount::reference(),
        )
}

fn install_money_movement_reads(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let schema =
        install_institution_movement_reads(schema, ApplyOpeningFundingOperation::reference());
    let schema = install_institution_movement_reads(schema, DepositOperation::reference());
    let schema = install_institution_movement_reads(schema, WithdrawOperation::reference());
    let schema = schema
        .operation_read_field(
            SendMoneyOperation::reference(),
            AccountIdentity::reference(),
        )
        .operation_read_field(
            SendMoneyOperation::reference(),
            AccountingRevision::reference(),
        )
        .operation_read_field(SendMoneyOperation::reference(), Status::reference())
        .operation_read_field(
            SendMoneyOperation::reference(),
            PrincipalIdentityField::reference(),
        )
        .operation_read_relation(SendMoneyOperation::reference(), PersonalOwner::reference());
    install_reversal_reads(schema)
}

fn install_institution_movement_reads<Operation, Input>(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
    operation: ApplicationOperationRef<BankSchema, Operation, Input>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema>
where
    InstitutionIdentityField: OperationReads<Operation>,
    AccountIdentity: OperationReads<Operation>,
    AccountingRevision: OperationReads<Operation>,
    Status: OperationReads<Operation>,
    InstitutionAccount: OperationReads<Operation>,
    InstitutionCashAccount: OperationReads<Operation>,
    Kind: OperationReads<Operation>,
    PersonalOwner: OperationReads<Operation>,
    BusinessAccount: OperationReads<Operation>,
    PrincipalIdentityField: OperationReads<Operation>,
    BusinessIdentityField: OperationReads<Operation>,
    AccountDisplayName: OperationReads<Operation>,
    PostingAccount: OperationReads<Operation>,
    JournalPosting: OperationReads<Operation>,
    JournalIdentityField: OperationReads<Operation>,
    JournalPurpose: OperationReads<Operation>,
    PostingIdentityField: OperationReads<Operation>,
    Purpose: OperationReads<Operation>,
    PostingAmount: OperationReads<Operation>,
    JournalReversal: OperationReads<Operation>,
{
    install_accounting_projection_reads(schema, operation)
        .operation_read_relation(operation, InstitutionCashAccount::reference())
}

fn install_reversal_reads(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    install_accounting_projection_reads(schema, ReverseJournalOperation::reference())
}

fn install_payment_reads(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let schema =
        install_account_projection_reads(schema, InitiateBusinessPaymentOperation::reference());
    let schema = install_accounting_projection_reads(schema, ApprovePaymentOperation::reference());
    let schema = install_payment_projection_reads(schema, ApprovePaymentOperation::reference());
    let schema = install_account_projection_reads(schema, RejectPaymentOperation::reference());
    install_payment_projection_reads(schema, RejectPaymentOperation::reference())
}

fn install_account_access_reads(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let schema =
        install_account_projection_reads(schema, GrantAccountAuthorizationOperation::reference())
            .operation_read_relation(
                GrantAccountAuthorizationOperation::reference(),
                AccountAuthorizedUser::reference(),
            )
            .operation_read_relation(
                GrantAccountAuthorizationOperation::reference(),
                AuthorizationAccount::reference(),
            )
            .operation_read_field(
                GrantAccountAuthorizationOperation::reference(),
                AccountAuthorizationIdentity::reference(),
            )
            .operation_read_field(
                GrantAccountAuthorizationOperation::reference(),
                AuthorizationRole::reference(),
            );
    schema
        .operation_read_field(
            RevokeAccountAuthorizationOperation::reference(),
            AccountIdentity::reference(),
        )
        .operation_read_field(
            RevokeAccountAuthorizationOperation::reference(),
            PrincipalIdentityField::reference(),
        )
        .operation_read_entity(
            RevokeAccountAuthorizationOperation::reference(),
            AccountAuthorization::reference(),
        )
        .operation_read_relation(
            RevokeAccountAuthorizationOperation::reference(),
            AccountAuthorizedUser::reference(),
        )
        .operation_read_relation(
            RevokeAccountAuthorizationOperation::reference(),
            AuthorizationAccount::reference(),
        )
        .operation_read_field(
            RevokeAccountAuthorizationOperation::reference(),
            AuthorizationRole::reference(),
        )
        .operation_read_field(
            RevokeAccountAuthorizationOperation::reference(),
            AccountAuthorizationIdentity::reference(),
        )
}

fn install_account_operation_budgets(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .operation_decision_fact_budget(CreatePersonalAccountOperation::reference(), 8)
        .operation_decision_fact_budget(CreateBusinessAccountOperation::reference(), 8)
        .operation_decision_fact_budget(GrantAccountAuthorizationOperation::reference(), 96)
        .operation_decision_fact_budget(RevokeAccountAuthorizationOperation::reference(), 96)
        .operation_projection_work_budget(CreatePersonalAccountOperation::reference(), 256)
        .operation_projection_work_budget(CreateBusinessAccountOperation::reference(), 256)
        .operation_projection_work_budget(GrantAccountAuthorizationOperation::reference(), 512)
        .operation_projection_work_budget(RevokeAccountAuthorizationOperation::reference(), 512)
}

fn install_payment_operation_budgets(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .operation_decision_fact_budget(ApplyOpeningFundingOperation::reference(), 256)
        .operation_decision_fact_budget(DepositOperation::reference(), 256)
        .operation_decision_fact_budget(WithdrawOperation::reference(), 256)
        .operation_decision_fact_budget(SendMoneyOperation::reference(), 256)
        .operation_decision_fact_budget(InitiateBusinessPaymentOperation::reference(), 64)
        .operation_decision_fact_budget(ApprovePaymentOperation::reference(), 256)
        .operation_decision_fact_budget(RejectPaymentOperation::reference(), 128)
        .operation_decision_fact_budget(ReverseJournalOperation::reference(), 256)
        .operation_projection_work_budget(ApplyOpeningFundingOperation::reference(), 4_096)
        .operation_projection_work_budget(DepositOperation::reference(), 4_096)
        .operation_projection_work_budget(WithdrawOperation::reference(), 4_096)
        .operation_projection_work_budget(SendMoneyOperation::reference(), 8_192)
        .operation_projection_work_budget(InitiateBusinessPaymentOperation::reference(), 4_096)
        .operation_projection_work_budget(ApprovePaymentOperation::reference(), 8_192)
        .operation_projection_work_budget(RejectPaymentOperation::reference(), 512)
        .operation_projection_work_budget(ReverseJournalOperation::reference(), 8_192)
}
