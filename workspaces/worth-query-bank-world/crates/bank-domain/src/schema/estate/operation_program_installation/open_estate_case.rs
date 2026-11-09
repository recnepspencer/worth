use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use crate::schema::{
    BankSchema, DeathNoticeIdentityField, DeathNoticeStatusField, EstateCaseIdentityField,
    EstateCaseStatusField, EstateDeathNotice, OpenEstateCaseOperation,
};

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let operation = OpenEstateCaseOperation::reference();
    schema
        .operation_projection_work_budget(operation, 40)
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
            DeathNoticeIdentityField::at("DeathNoticeIdentityField"),
        )
        .operation_read_field(
            operation,
            DeathNoticeStatusField::at("DeathNoticeStatusField"),
        )
        .operation_read_relation(operation, EstateDeathNotice::reference())
        .operation_write(
            operation,
            EstateCaseStatusField::at("EstateCaseStatusField"),
        )
}
