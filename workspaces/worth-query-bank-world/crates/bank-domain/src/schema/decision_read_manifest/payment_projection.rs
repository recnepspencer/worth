use worth_query_decl::facade::application_schema::{
    ApplicationOperationRef, ApplicationSchemaDeclarationBuilder, OperationReads,
};

use super::super::fields::{PaymentAmount, PaymentIdentityField, PaymentStatusField};
use super::super::relations::{
    ApprovalPrincipal, PaymentApproval, PaymentBusiness, PaymentDestination, PaymentInitiator,
    PaymentSource,
};
use super::super::BankSchema;

pub(super) fn install_payment_projection_reads<Operation, Input>(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
    operation: ApplicationOperationRef<BankSchema, Operation, Input>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema>
where
    PaymentSource: OperationReads<Operation>,
    PaymentDestination: OperationReads<Operation>,
    PaymentBusiness: OperationReads<Operation>,
    PaymentInitiator: OperationReads<Operation>,
    PaymentApproval: OperationReads<Operation>,
    ApprovalPrincipal: OperationReads<Operation>,
{
    schema
        .operation_read_field(operation, PaymentIdentityField::at("PaymentIdentityField"))
        .operation_read_field(operation, PaymentAmount::at("PaymentAmount"))
        .operation_read_field(operation, PaymentStatusField::at("PaymentStatusField"))
        .operation_read_relation(operation, PaymentSource::reference())
        .operation_read_relation(operation, PaymentDestination::reference())
        .operation_read_relation(operation, PaymentBusiness::reference())
        .operation_read_relation(operation, PaymentInitiator::reference())
        .operation_read_relation(operation, PaymentApproval::reference())
        .operation_read_relation(operation, ApprovalPrincipal::reference())
}
