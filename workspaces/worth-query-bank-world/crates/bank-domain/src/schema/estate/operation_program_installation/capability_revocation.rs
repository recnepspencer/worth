use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use crate::schema::{BankSchema, EstateCaseIdentityField, RevokeEstateCapabilityOperation};

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let operation = RevokeEstateCapabilityOperation::reference();
    schema
        .operation_projection_work_budget(operation, 32)
        .operation_read_field(operation, EstateCaseIdentityField::reference())
}
