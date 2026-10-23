//! Recorded account status restoration under current estate freeze authority.
mod binding;
mod correction;
mod request;

pub use binding::*;
pub use correction::RestoreRecordedEstateFreeze;

use crate::schema::{AccountIdentity, BankSchema, EstateAccount, Status};
use worth_query_decl::facade::{
    application_schema::ApplicationSchemaDeclarationBuilder, worth_query_capability,
    worth_query_operation, worth_query_operation_reads, worth_query_operation_writes,
};

worth_query_operation!(pub RestoreFrozenEstateAccountOperation for BankSchema, input RestoreFrozenEstateAccountInputBinding);
worth_query_operation_reads!(RestoreFrozenEstateAccountOperation => [AccountIdentity, Status, EstateAccount]);
worth_query_operation_writes!(RestoreFrozenEstateAccountOperation => [Status]);
worth_query_capability!(pub RestoreFrozenEstateAccountCapability in BankSchema);

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let operation = RestoreFrozenEstateAccountOperation::reference();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<RestoreFrozenEstateAccountBinding>()
        .operation_read_field(operation, AccountIdentity::reference())
        .operation_read_field(operation, Status::reference())
        .operation_read_relation(operation, EstateAccount::reference())
        .operation_write(operation, Status::reference())
}
