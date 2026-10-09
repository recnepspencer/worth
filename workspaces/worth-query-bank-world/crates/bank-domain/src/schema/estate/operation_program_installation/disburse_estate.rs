use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use crate::schema::{
    money_movement_program::MoneyMovementProgram, AccountDisplayName, AccountIdentity,
    AccountingRevision, BankSchema, BusinessAccount, BusinessIdentityField,
    DisburseEstateOperation, EstateAccount, EstateBeneficiary, EstateCaseIdentityField,
    EstateCaseStatusField, EstateExecutor, EstateJointOwner, InstitutionAccount,
    InstitutionIdentityField, JournalEntry, Kind, LegalAuthorityEstate, LegalAuthorityHolder,
    LegalAuthorityIdentityField, LegalAuthorityRecognizedField, PersonalOwner, Posting,
    PostingAccount, PostingAmount, PrincipalIdentityField, Status,
};

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let operation = DisburseEstateOperation::reference();
    schema
        .operation_projection_work_budget(operation, 192)
        .operation_read_field(operation, EstateCaseIdentityField::reference())
        .operation_read_field(operation, EstateCaseStatusField::reference())
        .operation_read_field(operation, PrincipalIdentityField::reference())
        .operation_read_field(operation, LegalAuthorityIdentityField::reference())
        .operation_read_field(operation, LegalAuthorityRecognizedField::reference())
        .operation_read_field(operation, AccountIdentity::reference())
        .operation_read_field(operation, AccountingRevision::reference())
        .operation_read_field(operation, Status::reference())
        .operation_read_field(operation, Kind::reference())
        .operation_read_field(operation, AccountDisplayName::reference())
        .operation_read_field(operation, InstitutionIdentityField::reference())
        .operation_read_field(operation, BusinessIdentityField::reference())
        .operation_read_field(operation, PostingAmount::reference())
        .operation_read_relation(operation, EstateAccount::reference())
        .operation_read_relation(operation, EstateBeneficiary::reference())
        .operation_read_relation(operation, EstateJointOwner::reference())
        .operation_read_relation(operation, LegalAuthorityEstate::reference())
        .operation_read_relation(operation, LegalAuthorityHolder::reference())
        .operation_read_relation(operation, EstateExecutor::reference())
        .operation_read_relation(operation, InstitutionAccount::reference())
        .operation_read_relation(operation, PersonalOwner::reference())
        .operation_read_relation(operation, BusinessAccount::reference())
        .operation_read_relation(operation, PostingAccount::reference())
        .operation_create(operation, JournalEntry::reference())
        .operation_create(operation, Posting::reference())
        .money_movement_program(operation)
}
