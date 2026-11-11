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
        .operation_read_field(
            operation,
            EstateCaseIdentityField::at("EstateCaseIdentityField"),
        )
        .operation_read_field(
            operation,
            EstateCaseStatusField::at("EstateCaseStatusField"),
        )
        .operation_read_field(
            operation,
            PrincipalIdentityField::at("PrincipalIdentityField"),
        )
        .operation_read_field(
            operation,
            LegalAuthorityIdentityField::at("LegalAuthorityIdentityField"),
        )
        .operation_read_field(
            operation,
            LegalAuthorityRecognizedField::at("LegalAuthorityRecognizedField"),
        )
        .operation_read_field(
            operation,
            MandatoryReviewIdentityField::at("MandatoryReviewIdentityField"),
        )
        .operation_read_field(
            operation,
            MandatoryReviewKindField::at("MandatoryReviewKindField"),
        )
        .operation_read_field(
            operation,
            MandatoryReviewStatusField::at("MandatoryReviewStatusField"),
        )
        .operation_read_relation(operation, EstateExecutor::reference())
        .operation_read_relation(operation, LegalAuthorityEstate::reference())
        .operation_read_relation(operation, LegalAuthorityHolder::reference())
        .operation_read_relation(operation, ReviewEstate::reference())
        .operation_read_relation(operation, ReviewPrincipal::reference())
        .operation_write(
            operation,
            EstateCaseStatusField::at("EstateCaseStatusField"),
        )
}
