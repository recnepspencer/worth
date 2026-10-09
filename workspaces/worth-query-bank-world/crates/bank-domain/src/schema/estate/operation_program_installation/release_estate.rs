use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use crate::schema::{
    BankSchema, EstateCaseIdentityField, EstateCaseStatusField, EstateExecutor,
    LegalAuthorityEstate, LegalAuthorityHolder, LegalAuthorityIdentityField,
    LegalAuthorityRecognizedField, MandatoryReviewIdentityField, MandatoryReviewKindField,
    MandatoryReviewStatusField, PrincipalIdentityField, ReleaseEstateOperation, ReviewEstate,
    ReviewPrincipal,
};

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let operation = ReleaseEstateOperation::reference();
    schema
        .operation_projection_work_budget(operation, 96)
        .operation_read_field(operation, EstateCaseIdentityField::reference())
        .operation_read_field(operation, EstateCaseStatusField::reference())
        .operation_read_field(operation, PrincipalIdentityField::reference())
        .operation_read_field(operation, LegalAuthorityIdentityField::reference())
        .operation_read_field(operation, LegalAuthorityRecognizedField::reference())
        .operation_read_field(operation, MandatoryReviewIdentityField::reference())
        .operation_read_field(operation, MandatoryReviewKindField::reference())
        .operation_read_field(operation, MandatoryReviewStatusField::reference())
        .operation_read_relation(operation, EstateExecutor::reference())
        .operation_read_relation(operation, LegalAuthorityEstate::reference())
        .operation_read_relation(operation, LegalAuthorityHolder::reference())
        .operation_read_relation(operation, ReviewEstate::reference())
        .operation_read_relation(operation, ReviewPrincipal::reference())
        .operation_write(operation, EstateCaseStatusField::reference())
}
