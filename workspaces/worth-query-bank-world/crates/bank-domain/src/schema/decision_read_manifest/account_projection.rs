use worth_query_decl::facade::application_schema::{
    ApplicationOperationRef, ApplicationSchemaDeclarationBuilder, OperationReads,
};

use super::super::authentication::PrincipalIdentityField;
use super::super::fields::{
    AccountDisplayName, AccountIdentity, AccountingRevision, BusinessIdentityField,
    InstitutionIdentityField, JournalIdentityField, JournalPurpose, Kind, PostingAmount,
    PostingIdentityField, Purpose, Status,
};
use super::super::relations::{
    BusinessAccount, InstitutionAccount, JournalPosting, JournalReversal, PersonalOwner,
    PostingAccount,
};
use super::super::BankSchema;

pub(super) fn install_account_projection_reads<Operation, Input>(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
    operation: ApplicationOperationRef<BankSchema, Operation, Input>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema>
where
    InstitutionAccount: OperationReads<Operation>,
    PersonalOwner: OperationReads<Operation>,
    BusinessAccount: OperationReads<Operation>,
{
    schema
        .operation_read_field(operation, AccountIdentity::at("AccountIdentity"))
        .operation_read_field(operation, AccountingRevision::at("AccountingRevision"))
        .operation_read_field(
            operation,
            InstitutionIdentityField::at("InstitutionIdentityField"),
        )
        .operation_read_field(operation, Kind::at("Kind"))
        .operation_read_field(
            operation,
            PrincipalIdentityField::at("PrincipalIdentityField"),
        )
        .operation_read_field(
            operation,
            BusinessIdentityField::at("BusinessIdentityField"),
        )
        .operation_read_field(operation, Status::at("Status"))
        .operation_read_field(operation, AccountDisplayName::at("AccountDisplayName"))
        .operation_read_relation(operation, InstitutionAccount::reference())
        .operation_read_relation(operation, PersonalOwner::reference())
        .operation_read_relation(operation, BusinessAccount::reference())
}

pub(super) fn install_accounting_projection_reads<Operation, Input>(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
    operation: ApplicationOperationRef<BankSchema, Operation, Input>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema>
where
    InstitutionAccount: OperationReads<Operation>,
    PersonalOwner: OperationReads<Operation>,
    BusinessAccount: OperationReads<Operation>,
    PostingAccount: OperationReads<Operation>,
    JournalPosting: OperationReads<Operation>,
    JournalReversal: OperationReads<Operation>,
{
    install_account_projection_reads(schema, operation)
        .operation_read_field(operation, JournalIdentityField::at("JournalIdentityField"))
        .operation_read_field(operation, JournalPurpose::at("JournalPurpose"))
        .operation_read_field(operation, PostingIdentityField::at("PostingIdentityField"))
        .operation_read_field(operation, Purpose::at("Purpose"))
        .operation_read_field(operation, PostingAmount::at("PostingAmount"))
        .operation_read_relation(operation, PostingAccount::reference())
        .operation_read_relation(operation, JournalPosting::reference())
        .operation_read_relation(operation, JournalReversal::reference())
}
