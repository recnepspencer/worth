use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use super::{AccountingRevision, BankSchema, SendMoneyOperation, Status};

pub(super) fn install_payment_preconditions(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .operation_expected_version(
            SendMoneyOperation::reference(),
            AccountingRevision::at("AccountingRevision"),
        )
        .operation_expected_fact(SendMoneyOperation::reference(), Status::at("Status"))
}
