use worth_query_decl::facade::application_schema::{
    ApplicationDeclaredOperation, ApplicationSchemaDeclarationBuilder,
};

use crate::schema::{
    AccountIdentity, BankSchema, EstateAccount, FreezeEstateAccountOperation, Status,
};

impl ApplicationDeclaredOperation<BankSchema> for FreezeEstateAccountOperation {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
        let operation = Self::reference();
        schema
            .operation_projection_work_budget(operation, 32)
            .operation_read_field(operation, AccountIdentity::at("AccountIdentity"))
            .operation_read_field(operation, Status::at("Status"))
            .operation_read_relation(operation, EstateAccount::reference())
            .operation_write(operation, Status::at("Status"))
    }
}
