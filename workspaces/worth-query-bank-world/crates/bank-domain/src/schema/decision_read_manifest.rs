use worth_query_decl::facade::application_schema::{
    ApplicationOperationRef, ApplicationSchemaDeclarationBuilder, OperationReads,
};

use super::authentication::PrincipalIdentityField;
use super::entities::AccountAuthorization;
use super::fields::{
    AccountAuthorizationIdentity, AccountIdentity, AuthorizationRole, BusinessIdentityField,
    InstitutionIdentityField, PaymentIdentityField,
};
use super::operations::*;
use super::relations::{
    AccountAuthorizedUser, AuthorizationAccount, BusinessAccount, InstitutionAccount,
    InstitutionCashAccount, JournalPosting, JournalReversal, PersonalOwner, PostingAccount,
};
use super::workflow::{
    ApprovedBusinessPaymentAdvanceOperation, ApprovedBusinessPaymentApprovalOperation,
    ApprovedBusinessPaymentAuthoringOperation, ApprovedBusinessPaymentInstanceStartOperation,
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

pub(super) fn install_recorded_money_decision_reads(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let schema = install_money_movement_reads(schema);
    install_reversal_budget(install_money_movement_budgets(schema))
}

fn install_account_creation_reads(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .operation_read_field(
            CreatePersonalAccountOperation::reference(),
            InstitutionIdentityField::at("InstitutionIdentityField"),
        )
        .operation_read_field(
            CreatePersonalAccountOperation::reference(),
            PrincipalIdentityField::at("PrincipalIdentityField"),
        )
        .operation_read_relation(
            CreatePersonalAccountOperation::reference(),
            PersonalOwner::reference(),
        )
        .operation_read_field(
            CreateBusinessAccountOperation::reference(),
            InstitutionIdentityField::at("InstitutionIdentityField"),
        )
        .operation_read_field(
            CreateBusinessAccountOperation::reference(),
            BusinessIdentityField::at("BusinessIdentityField"),
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
    install_reversal_reads(schema)
}

fn install_institution_movement_reads<Operation, Input>(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
    operation: ApplicationOperationRef<BankSchema, Operation, Input>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema>
where
    InstitutionAccount: OperationReads<Operation>,
    InstitutionCashAccount: OperationReads<Operation>,
    PersonalOwner: OperationReads<Operation>,
    BusinessAccount: OperationReads<Operation>,
    PostingAccount: OperationReads<Operation>,
    JournalPosting: OperationReads<Operation>,
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
    let schema = install_account_projection_reads(schema, RejectPaymentOperation::reference());
    install_payment_projection_reads(schema, RejectPaymentOperation::reference())
}

pub(super) fn install_approved_payment_reads(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let schema = install_accounting_projection_reads(schema, ApprovePaymentOperation::reference());
    install_payment_projection_reads(schema, ApprovePaymentOperation::reference())
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
                AccountAuthorizationIdentity::at("AccountAuthorizationIdentity"),
            )
            .operation_read_field(
                GrantAccountAuthorizationOperation::reference(),
                AuthorizationRole::at("AuthorizationRole"),
            );
    schema
        .operation_read_field(
            RevokeAccountAuthorizationOperation::reference(),
            AccountIdentity::at("AccountIdentity"),
        )
        .operation_read_field(
            RevokeAccountAuthorizationOperation::reference(),
            PrincipalIdentityField::at("PrincipalIdentityField"),
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
            AuthorizationRole::at("AuthorizationRole"),
        )
        .operation_read_field(
            RevokeAccountAuthorizationOperation::reference(),
            AccountAuthorizationIdentity::at("AccountAuthorizationIdentity"),
        )
}

fn install_account_operation_budgets(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .operation_projection_work_budget(CreatePersonalAccountOperation::reference(), 256)
        .operation_projection_work_budget(CreateBusinessAccountOperation::reference(), 256)
        .operation_projection_work_budget(GrantAccountAuthorizationOperation::reference(), 512)
        .operation_projection_work_budget(RevokeAccountAuthorizationOperation::reference(), 512)
}

fn install_payment_operation_budgets(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let schema = schema
        .operation_read_field(
            ApprovedBusinessPaymentAuthoringOperation::reference(),
            PaymentIdentityField::at("PaymentIdentityField"),
        )
        .operation_read_field(
            ApprovedBusinessPaymentApprovalOperation::reference(),
            PaymentIdentityField::at("PaymentIdentityField"),
        )
        .operation_read_field(
            ApprovedBusinessPaymentInstanceStartOperation::reference(),
            PaymentIdentityField::at("PaymentIdentityField"),
        )
        .operation_read_field(
            ApprovedBusinessPaymentAdvanceOperation::reference(),
            PaymentIdentityField::at("PaymentIdentityField"),
        );
    let schema = install_money_movement_budgets(schema)
        .operation_projection_work_budget(InitiateBusinessPaymentOperation::reference(), 8_192)
        .operation_projection_work_budget(
            ApprovedBusinessPaymentAuthoringOperation::reference(),
            1_024,
        )
        .operation_projection_work_budget(
            ApprovedBusinessPaymentApprovalOperation::reference(),
            1_024,
        )
        .operation_projection_work_budget(
            ApprovedBusinessPaymentInstanceStartOperation::reference(),
            1_024,
        )
        .operation_projection_work_budget(
            ApprovedBusinessPaymentAdvanceOperation::reference(),
            1_024,
        )
        .operation_projection_work_budget(ApprovePaymentOperation::reference(), 8_192)
        .operation_projection_work_budget(RejectPaymentOperation::reference(), 512);
    install_reversal_budget(schema)
}

fn install_money_movement_budgets(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .operation_projection_work_budget(ApplyOpeningFundingOperation::reference(), 4_096)
        .operation_projection_work_budget(DepositOperation::reference(), 4_096)
        .operation_projection_work_budget(WithdrawOperation::reference(), 4_096)
        .operation_projection_work_budget(SendMoneyOperation::reference(), 8_192)
}

fn install_reversal_budget(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema.operation_projection_work_budget(ReverseJournalOperation::reference(), 8_192)
}
