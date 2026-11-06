use worth_query_declaration::facade::application_capability::ApplicationCapabilityContract;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityContractBuilder;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityElevationRule;
use worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityAction;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::command_composition;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::command_constraints;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::command_target;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::delegation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationGrant;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationIdentity;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationNotAfter;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationNotBefore;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationReason;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationRequester;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationReview;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationStatusField;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityGrant;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReview;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewIdentity;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewKindField;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewResource;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewStatusField;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::RequestCapabilityElevationOperation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::RequestElevationCapability;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::RequestElevationInput;
use crate::domain_computation::primary_graph::tests::fixture::AccountLabel;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;

pub fn install(
    schema: ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
) -> ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema> {
    schema
        .operation_projection_work_budget(RequestCapabilityElevationOperation::reference(), 32)
        .operation_read_field(
            RequestCapabilityElevationOperation::reference(),
            AccountLabel::at("AccountLabel"),
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
            CapabilityElevationIdentity::at("CapabilityElevationIdentity"),
        )
        .operation_write(
            RequestCapabilityElevationOperation::reference(),
            CapabilityElevationReason::at("CapabilityElevationReason"),
        )
        .operation_write(
            RequestCapabilityElevationOperation::reference(),
            CapabilityElevationStatusField::at("CapabilityElevationStatusField"),
        )
        .operation_write(
            RequestCapabilityElevationOperation::reference(),
            CapabilityElevationNotBefore::at("CapabilityElevationNotBefore"),
        )
        .operation_write(
            RequestCapabilityElevationOperation::reference(),
            CapabilityElevationNotAfter::at("CapabilityElevationNotAfter"),
        )
        .operation_write(
            RequestCapabilityElevationOperation::reference(),
            CapabilityReviewIdentity::at("CapabilityReviewIdentity"),
        )
        .operation_write(
            RequestCapabilityElevationOperation::reference(),
            CapabilityReviewKindField::at("CapabilityReviewKindField"),
        )
        .operation_write(
            RequestCapabilityElevationOperation::reference(),
            CapabilityReviewStatusField::at("CapabilityReviewStatusField"),
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

pub fn request_contract() -> ApplicationCapabilityContract<
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
