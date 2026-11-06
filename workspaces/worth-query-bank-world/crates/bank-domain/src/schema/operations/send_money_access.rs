//! Installed access of the correctable SendMoney source.
use crate::schema::money_movement_program::MoneyMovementProgram;
use crate::schema::{
    AccountIdentity, AccountingRevision, BankSchema, JournalEntry, PersonalOwner, Posting,
    PrincipalIdentityField, SendMoneyOperation, Status,
};
use worth_query_decl::facade::application_schema::{
    ApplicationDeclaredOperation, ApplicationSchemaDeclarationBuilder,
};

impl ApplicationDeclaredOperation<BankSchema> for SendMoneyOperation {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
        schema
            .operation_create(Self::reference(), JournalEntry::reference())
            .operation_create(Self::reference(), Posting::reference())
            .money_movement_program(Self::reference())
            .operation_read_field(Self::reference(), AccountIdentity::at("AccountIdentity"))
            .operation_read_field(
                Self::reference(),
                AccountingRevision::at("AccountingRevision"),
            )
            .operation_read_field(Self::reference(), Status::at("Status"))
            .operation_read_field(
                Self::reference(),
                PrincipalIdentityField::at("PrincipalIdentityField"),
            )
            .operation_read_relation(Self::reference(), PersonalOwner::reference())
    }
}
