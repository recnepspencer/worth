use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityContract, ApplicationCapabilityContractBuilder,
        ApplicationCapabilityElevationRule,
    },
    application_schema::ApplicationSchemaDeclarationBuilder,
};

use super::super::{
    CapabilityElevation, CapabilityElevationGrant, CapabilityElevationIdentity,
    CapabilityElevationNotAfter, CapabilityElevationNotBefore, CapabilityElevationReason,
    CapabilityElevationRequester, CapabilityElevationReview, CapabilityElevationStatusField,
    CapabilityGrant, CapabilityReview, CapabilityReviewIdentity, CapabilityReviewKindField,
    CapabilityReviewResource, CapabilityReviewStatusField, RequestCapabilityElevationOperation,
    RequestElevationCapability, RequestElevationInput,
};
use super::{command_composition, command_constraints, command_target, delegation};
use crate::domain_computation::primary_graph::tests::fixture::CapabilityAction;
use crate::domain_computation::primary_graph::tests::fixture::{
    AccountLabel, IdentityExecutionSchema,
};

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
) -> ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema> {
    schema
        .operation_projection_work_budget(RequestCapabilityElevationOperation::reference(), 32)
        .operation_read_field(
            RequestCapabilityElevationOperation::reference(),
            AccountLabel::reference(),
        )
        .operation_create(
            RequestCapabilityElevationOperation::reference(),
            CapabilityElevation::reference(),
        )
        .operation_create(
            RequestCapabilityElevationOperation::reference(),
            CapabilityReview::reference(),
        )
        .operation_write(
            RequestCapabilityElevationOperation::reference(),
            CapabilityElevationIdentity::reference(),
        )
        .operation_write(
            RequestCapabilityElevationOperation::reference(),
            CapabilityElevationReason::reference(),
        )
        .operation_write(
            RequestCapabilityElevationOperation::reference(),
            CapabilityElevationStatusField::reference(),
        )
        .operation_write(
            RequestCapabilityElevationOperation::reference(),
            CapabilityElevationNotBefore::reference(),
        )
        .operation_write(
            RequestCapabilityElevationOperation::reference(),
            CapabilityElevationNotAfter::reference(),
        )
        .operation_write(
            RequestCapabilityElevationOperation::reference(),
            CapabilityReviewIdentity::reference(),
        )
        .operation_write(
            RequestCapabilityElevationOperation::reference(),
            CapabilityReviewKindField::reference(),
        )
        .operation_write(
            RequestCapabilityElevationOperation::reference(),
            CapabilityReviewStatusField::reference(),
        )
        .operation_link(
            RequestCapabilityElevationOperation::reference(),
            CapabilityElevationRequester::reference(),
        )
        .operation_link(
            RequestCapabilityElevationOperation::reference(),
            CapabilityElevationGrant::reference(),
        )
        .operation_link(
            RequestCapabilityElevationOperation::reference(),
            CapabilityElevationReview::reference(),
        )
        .operation_link(
            RequestCapabilityElevationOperation::reference(),
            CapabilityReviewResource::reference(),
        )
        .capability(request_contract())
}

fn request_contract() -> ApplicationCapabilityContract<
    IdentityExecutionSchema,
    RequestElevationCapability,
    RequestCapabilityElevationOperation,
    RequestElevationInput,
> {
    ApplicationCapabilityContractBuilder::new(
        RequestElevationCapability::reference(),
        RequestCapabilityElevationOperation::reference(),
        CapabilityGrant::reference(),
    )
    .target(command_target(CapabilityAction::RequestElevation))
    .constraints(command_constraints())
    .delegation(delegation())
    .composition(command_composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}
