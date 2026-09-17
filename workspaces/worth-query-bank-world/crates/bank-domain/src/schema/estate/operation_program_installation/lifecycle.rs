use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use super::super::*;
use crate::schema::BankSchema;

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let schema = install_request(schema);
    let schema = install_approval(schema);
    let schema = install_close(schema);
    install_review(schema)
}

fn install_request(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let operation = RequestEstateEmergencyAccessOperation::reference();
    schema
        .operation_decision_fact_budget(operation, 1)
        .operation_projection_work_budget(operation, 32)
        .operation_read_field(operation, EstateCaseIdentityField::reference())
        .operation_create(operation, EmergencyAccess::reference())
        .operation_create(operation, MandatoryReview::reference())
        .operation_write(operation, EmergencyAccessIdentityField::reference())
        .operation_write(operation, EmergencyAccessReasonField::reference())
        .operation_write(operation, EmergencyAccessStatusField::reference())
        .operation_write(operation, EmergencyAccessIssuedAtField::reference())
        .operation_write(operation, EmergencyAccessExpiresAtField::reference())
        .operation_write(operation, MandatoryReviewIdentityField::reference())
        .operation_write(operation, MandatoryReviewKindField::reference())
        .operation_write(operation, MandatoryReviewStatusField::reference())
        .operation_link(operation, EmergencyRequester::reference())
        .operation_link(operation, EmergencyGrant::reference())
        .operation_link(operation, EmergencyReview::reference())
        .operation_link(operation, ReviewEstate::reference())
}

fn install_approval(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let operation = ApproveEstateEmergencyAccessOperation::reference();
    install_lifecycle_reads(schema, operation)
        .operation_write(operation, EmergencyAccessStatusField::reference())
        .operation_link(operation, EmergencyApprover::reference())
}

fn install_close(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let operation = RevokeEstateEmergencyAccessOperation::reference();
    install_lifecycle_reads(schema, operation)
        .operation_read_field(operation, EmergencyAccessClosedAtField::reference())
        .operation_write(operation, EmergencyAccessStatusField::reference())
        .operation_write(operation, EmergencyAccessClosedAtField::reference())
}

fn install_review(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let operation = CompleteEstateMandatoryReviewOperation::reference();
    install_lifecycle_reads(schema, operation)
        .operation_read_field(operation, MandatoryReviewReviewedAtField::reference())
        .operation_write(operation, MandatoryReviewStatusField::reference())
        .operation_write(operation, MandatoryReviewReviewedAtField::reference())
        .operation_link(operation, ReviewPrincipal::reference())
}

fn install_lifecycle_reads<Operation>(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
    operation: worth_query_decl::facade::application_schema::ApplicationOperationRef<
        BankSchema,
        Operation,
        crate::estate::EstateAction,
    >,
) -> ApplicationSchemaDeclarationBuilder<BankSchema>
where
    EmergencyAccessIdentityField:
        worth_query_decl::facade::application_schema::OperationReads<Operation>,
    EmergencyAccessReasonField:
        worth_query_decl::facade::application_schema::OperationReads<Operation>,
    EmergencyAccessStatusField:
        worth_query_decl::facade::application_schema::OperationReads<Operation>,
    EmergencyAccessIssuedAtField:
        worth_query_decl::facade::application_schema::OperationReads<Operation>,
    EmergencyAccessExpiresAtField:
        worth_query_decl::facade::application_schema::OperationReads<Operation>,
    MandatoryReviewIdentityField:
        worth_query_decl::facade::application_schema::OperationReads<Operation>,
    MandatoryReviewKindField:
        worth_query_decl::facade::application_schema::OperationReads<Operation>,
    MandatoryReviewStatusField:
        worth_query_decl::facade::application_schema::OperationReads<Operation>,
    EmergencyRequester: worth_query_decl::facade::application_schema::OperationReads<Operation>,
    EmergencyApprover: worth_query_decl::facade::application_schema::OperationReads<Operation>,
    EmergencyGrant: worth_query_decl::facade::application_schema::OperationReads<Operation>,
    EmergencyEstate: worth_query_decl::facade::application_schema::OperationReads<Operation>,
    EmergencyReview: worth_query_decl::facade::application_schema::OperationReads<Operation>,
    ReviewEstate: worth_query_decl::facade::application_schema::OperationReads<Operation>,
    ReviewPrincipal: worth_query_decl::facade::application_schema::OperationReads<Operation>,
{
    schema
        .operation_decision_fact_budget(operation, 16)
        .operation_projection_work_budget(operation, 128)
        .operation_read_field(operation, EmergencyAccessIdentityField::reference())
        .operation_read_field(operation, EmergencyAccessReasonField::reference())
        .operation_read_field(operation, EmergencyAccessStatusField::reference())
        .operation_read_field(operation, EmergencyAccessIssuedAtField::reference())
        .operation_read_field(operation, EmergencyAccessExpiresAtField::reference())
        .operation_read_field(operation, MandatoryReviewIdentityField::reference())
        .operation_read_field(operation, MandatoryReviewKindField::reference())
        .operation_read_field(operation, MandatoryReviewStatusField::reference())
        .operation_read_relation(operation, EmergencyRequester::reference())
        .operation_read_relation(operation, EmergencyApprover::reference())
        .operation_read_relation(operation, EmergencyGrant::reference())
        .operation_read_relation(operation, EmergencyEstate::reference())
        .operation_read_relation(operation, EmergencyReview::reference())
        .operation_read_relation(operation, ReviewEstate::reference())
        .operation_read_relation(operation, ReviewPrincipal::reference())
}
