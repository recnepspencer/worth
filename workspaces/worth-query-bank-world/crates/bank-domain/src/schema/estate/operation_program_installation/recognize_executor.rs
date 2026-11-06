use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use crate::schema::{
    BankSchema, EstateCaseIdentityField, EstateExecutor, LegalAuthorityEstate,
    LegalAuthorityHolder, LegalAuthorityIdentityField, LegalAuthorityRecognizedField,
    PrincipalIdentityField, RecognizeEstateExecutorOperation,
};

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let operation = RecognizeEstateExecutorOperation::reference();
    schema
        .operation_projection_work_budget(operation, 48)
        .operation_read_field(
            operation,
            EstateCaseIdentityField::at("EstateCaseIdentityField"),
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
            PrincipalIdentityField::at("PrincipalIdentityField"),
        )
        .operation_read_relation(operation, LegalAuthorityEstate::reference())
        .operation_read_relation(operation, LegalAuthorityHolder::reference())
        .operation_read_relation(operation, EstateExecutor::reference())
        .operation_link(operation, EstateExecutor::reference())
}
