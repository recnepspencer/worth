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
        .operation_projection_work_budget(operation, 32)
        .operation_read_field(
            operation,
            EstateCaseIdentityField::at("EstateCaseIdentityField"),
        )
        .operation_create(operation, EmergencyAccess::reference())
        .operation_create(operation, MandatoryReview::reference())
        .operation_write(
            operation,
            EmergencyAccessIdentityField::at("EmergencyAccessIdentityField"),
        )
        .operation_write(
            operation,
            EmergencyAccessReasonField::at("EmergencyAccessReasonField"),
        )
        .operation_write(
            operation,
            EmergencyAccessStatusField::at("EmergencyAccessStatusField"),
        )
        .operation_write(
            operation,
            EmergencyAccessIssuedAtField::at("EmergencyAccessIssuedAtField"),
        )
        .operation_write(
            operation,
            EmergencyAccessExpiresAtField::at("EmergencyAccessExpiresAtField"),
        )
        .operation_write(
            operation,
            MandatoryReviewIdentityField::at("MandatoryReviewIdentityField"),
        )
        .operation_write(
            operation,
            MandatoryReviewKindField::at("MandatoryReviewKindField"),
        )
        .operation_write(
            operation,
            MandatoryReviewStatusField::at("MandatoryReviewStatusField"),
        )
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
        .operation_write(
            operation,
            EmergencyAccessStatusField::at("EmergencyAccessStatusField"),
        )
        .operation_link(operation, EmergencyApprover::reference())
}

fn install_close(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let operation = RevokeEstateEmergencyAccessOperation::reference();
    install_lifecycle_reads(schema, operation)
        .operation_read_field(
            operation,
            EmergencyAccessClosedAtField::at("EmergencyAccessClosedAtField"),
        )
        .operation_write(
            operation,
            EmergencyAccessStatusField::at("EmergencyAccessStatusField"),
        )
        .operation_write(
            operation,
            EmergencyAccessClosedAtField::at("EmergencyAccessClosedAtField"),
        )
}

fn install_review(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let operation = CompleteEstateMandatoryReviewOperation::reference();
    install_lifecycle_reads(schema, operation)
        .operation_read_field(
            operation,
            MandatoryReviewReviewedAtField::at("MandatoryReviewReviewedAtField"),
        )
        .operation_write(
            operation,
            MandatoryReviewStatusField::at("MandatoryReviewStatusField"),
        )
        .operation_write(
            operation,
            MandatoryReviewReviewedAtField::at("MandatoryReviewReviewedAtField"),
        )
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
    EmergencyRequester: worth_query_decl::facade::application_schema::OperationReads<Operation>,
    EmergencyApprover: worth_query_decl::facade::application_schema::OperationReads<Operation>,
    EmergencyGrant: worth_query_decl::facade::application_schema::OperationReads<Operation>,
    EmergencyEstate: worth_query_decl::facade::application_schema::OperationReads<Operation>,
    EmergencyReview: worth_query_decl::facade::application_schema::OperationReads<Operation>,
    ReviewEstate: worth_query_decl::facade::application_schema::OperationReads<Operation>,
    ReviewPrincipal: worth_query_decl::facade::application_schema::OperationReads<Operation>,
{
    schema
        .operation_projection_work_budget(operation, 128)
        .operation_read_field(
            operation,
            EmergencyAccessIdentityField::at("EmergencyAccessIdentityField"),
        )
        .operation_read_field(
            operation,
            EmergencyAccessReasonField::at("EmergencyAccessReasonField"),
        )
        .operation_read_field(
            operation,
            EmergencyAccessStatusField::at("EmergencyAccessStatusField"),
        )
        .operation_read_field(
            operation,
            EmergencyAccessIssuedAtField::at("EmergencyAccessIssuedAtField"),
        )
        .operation_read_field(
            operation,
            EmergencyAccessExpiresAtField::at("EmergencyAccessExpiresAtField"),
        )
        .operation_read_field(
            operation,
            MandatoryReviewIdentityField::at("MandatoryReviewIdentityField"),
        )
        .operation_read_field(
            operation,
            MandatoryReviewKindField::at("MandatoryReviewKindField"),
        )
        .operation_read_field(
            operation,
            MandatoryReviewStatusField::at("MandatoryReviewStatusField"),
        )
        .operation_read_relation(operation, EmergencyRequester::reference())
        .operation_read_relation(operation, EmergencyApprover::reference())
        .operation_read_relation(operation, EmergencyGrant::reference())
        .operation_read_relation(operation, EmergencyEstate::reference())
        .operation_read_relation(operation, EmergencyReview::reference())
        .operation_read_relation(operation, ReviewEstate::reference())
        .operation_read_relation(operation, ReviewPrincipal::reference())
}
