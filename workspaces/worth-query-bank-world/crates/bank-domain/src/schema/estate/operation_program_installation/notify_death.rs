use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use crate::schema::{
    BankSchema, DeathNoticeIdentityField, DeathNoticeStatusField, DeathNoticeSubject,
    EstateDeathNotice, EstateDeathNotificationEffect, EstateDeceased, NotifyDeathEstateOperation,
    PrincipalIdentityField,
};

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let operation = NotifyDeathEstateOperation::reference();
    schema
        .operation_projection_work_budget(operation, 48)
        .operation_read_field(operation, DeathNoticeIdentityField::reference())
        .operation_read_field(operation, DeathNoticeStatusField::reference())
        .operation_read_field(operation, PrincipalIdentityField::reference())
        .operation_read_relation(operation, EstateDeathNotice::reference())
        .operation_read_relation(operation, DeathNoticeSubject::reference())
        .operation_read_relation(operation, EstateDeceased::reference())
        .operation_write(operation, DeathNoticeStatusField::reference())
        .operation_emit(operation, EstateDeathNotificationEffect::reference())
}
