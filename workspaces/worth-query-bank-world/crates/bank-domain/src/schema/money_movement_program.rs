use worth_query_decl::facade::application_schema::{
    ApplicationOperationRef, ApplicationSchemaDeclarationBuilder, OperationLinks, OperationWrites,
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
        JournalIdentityField: OperationWrites<Operation>,
        JournalInstitutionContext: OperationWrites<Operation>,
        JournalPurpose: OperationWrites<Operation>,
        PostingIdentityField: OperationWrites<Operation>,
        PostingAmount: OperationWrites<Operation>,
        PostingAccountSequence: OperationWrites<Operation>,
        Purpose: OperationWrites<Operation>,
        AccountingRevision: OperationWrites<Operation>,
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
        JournalIdentityField: OperationWrites<Operation>,
        JournalInstitutionContext: OperationWrites<Operation>,
        JournalPurpose: OperationWrites<Operation>,
        PostingIdentityField: OperationWrites<Operation>,
        PostingAmount: OperationWrites<Operation>,
        PostingAccountSequence: OperationWrites<Operation>,
        Purpose: OperationWrites<Operation>,
        AccountingRevision: OperationWrites<Operation>,
        JournalPosting: OperationLinks<Operation>,
        PostingAccount: OperationLinks<Operation>,
        AccountActivityEffect:
            worth_query_decl::facade::application_schema::OperationEmits<Operation>,
    {
        self.operation_write(operation, PostingAmount::reference())
            .operation_write(operation, PostingAccountSequence::reference())
            .operation_write(operation, PostingIdentityField::reference())
            .operation_write(operation, Purpose::reference())
            .operation_write(operation, JournalIdentityField::reference())
            .operation_write(operation, JournalInstitutionContext::reference())
            .operation_write(operation, JournalPurpose::reference())
            .operation_write(operation, AccountingRevision::reference())
            .operation_link(operation, JournalPosting::reference())
            .operation_link(operation, PostingAccount::reference())
            .operation_emit(operation, AccountActivityEffect::reference())
    }
}
