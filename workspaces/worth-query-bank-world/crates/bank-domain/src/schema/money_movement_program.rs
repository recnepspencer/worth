use worth_query_decl::facade::application_schema::{
    ApplicationOperationRef, ApplicationSchemaDeclarationBuilder, OperationLinks,
};

use super::{
    AccountActivityEffect, AccountingRevision, BankSchema, JournalIdentityField,
    JournalInstitutionContext, JournalPosting, JournalPurpose, PostingAccount,
    PostingAccountSequence, PostingAmount, PostingIdentityField, Purpose,
};

pub(crate) trait MoneyMovementProgram {
    fn money_movement_program<Operation, Input>(
        self,
        operation: ApplicationOperationRef<BankSchema, Operation, Input>,
    ) -> Self
    where
        JournalPosting: OperationLinks<Operation>,
        PostingAccount: OperationLinks<Operation>,
        AccountActivityEffect:
            worth_query_decl::facade::application_schema::OperationEmits<Operation>;
}

impl MoneyMovementProgram for ApplicationSchemaDeclarationBuilder<BankSchema> {
    fn money_movement_program<Operation, Input>(
        self,
        operation: ApplicationOperationRef<BankSchema, Operation, Input>,
    ) -> Self
    where
        JournalPosting: OperationLinks<Operation>,
        PostingAccount: OperationLinks<Operation>,
        AccountActivityEffect:
            worth_query_decl::facade::application_schema::OperationEmits<Operation>,
    {
        self.operation_write(operation, PostingAmount::at("PostingAmount"))
            .operation_write(
                operation,
                PostingAccountSequence::at("PostingAccountSequence"),
            )
            .operation_write(operation, PostingIdentityField::at("PostingIdentityField"))
            .operation_write(operation, Purpose::at("Purpose"))
            .operation_write(operation, JournalIdentityField::at("JournalIdentityField"))
            .operation_write(
                operation,
                JournalInstitutionContext::at("JournalInstitutionContext"),
            )
            .operation_write(operation, JournalPurpose::at("JournalPurpose"))
            .operation_write(operation, AccountingRevision::at("AccountingRevision"))
            .operation_link(operation, JournalPosting::reference())
            .operation_link(operation, PostingAccount::reference())
            .operation_emit(operation, AccountActivityEffect::reference())
    }
}
