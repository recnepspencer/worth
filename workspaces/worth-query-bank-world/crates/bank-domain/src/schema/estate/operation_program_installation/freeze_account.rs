use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use crate::schema::{
    AccountIdentity, BankSchema, EstateAccount, FreezeEstateAccountOperation, Status,
};

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let operation = FreezeEstateAccountOperation::reference();
    schema
        .operation_projection_work_budget(operation, 32)
        .operation_read_field(operation, AccountIdentity::reference())
        .operation_read_field(operation, Status::reference())
        .operation_read_relation(operation, EstateAccount::reference())
        .operation_write(operation, Status::reference())
}
