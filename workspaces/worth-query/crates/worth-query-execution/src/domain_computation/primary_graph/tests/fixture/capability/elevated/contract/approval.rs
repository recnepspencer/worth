use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityActorComposition, ApplicationCapabilityAllowRule,
        ApplicationCapabilityComposition, ApplicationCapabilityConflictRule,
        ApplicationCapabilityContract, ApplicationCapabilityContractBuilder,
        ApplicationCapabilityDecisionComposition, ApplicationCapabilityDenyRule,
        ApplicationCapabilityDistinctActorRule, ApplicationCapabilityElevationRule,
        ApplicationCapabilityGraphClause, ApplicationCapabilityGraphRule,
        ApplicationCapabilityPathContextAnchor, ApplicationCapabilitySeparationOfDutyRule,
    },
    application_schema::{
        ApplicationAuthorizationPathBuilder, ApplicationSchemaDeclarationBuilder,
    },
};

use super::super::{
    ApproveCapabilityElevationOperation, ApproveElevationCapability, ApproveElevationInput,
    CapabilityConflictingBeneficiary, CapabilityElevationApprover, CapabilityElevationGrant,
    CapabilityElevationIdentity, CapabilityElevationNotAfter, CapabilityElevationNotBefore,
    CapabilityElevationReason, CapabilityElevationRequester, CapabilityElevationResource,
    CapabilityElevationReview, CapabilityElevationSlot, CapabilityElevationStatusField,
    CapabilityGrant, CapabilityGrantor, CapabilityResource, CapabilityReviewIdentity,
    CapabilityReviewKindField, CapabilityReviewResource, CapabilityReviewStatusField,
    CapabilityReviewer,
};
use super::{command_constraints, command_propagation, command_target, delegation};
use crate::domain_computation::primary_graph::tests::fixture::{
    Account, CapabilityAction, IdentityExecutionSchema, Principal,
};

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
) -> ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema> {
    let operation = ApproveCapabilityElevationOperation::reference();
    schema
        .operation_projection_work_budget(operation, 96)
        .operation_read_field(operation, CapabilityElevationIdentity::reference())
        .operation_read_field(operation, CapabilityElevationReason::reference())
        .operation_read_field(operation, CapabilityElevationStatusField::reference())
        .operation_read_field(operation, CapabilityElevationNotBefore::reference())
        .operation_read_field(operation, CapabilityElevationNotAfter::reference())
        .operation_read_field(operation, CapabilityReviewIdentity::reference())
        .operation_read_field(operation, CapabilityReviewKindField::reference())
        .operation_read_field(operation, CapabilityReviewStatusField::reference())
        .operation_read_relation(operation, CapabilityElevationRequester::reference())
        .operation_read_relation(operation, CapabilityElevationApprover::reference())
        .operation_read_relation(operation, CapabilityElevationGrant::reference())
        .operation_read_relation(operation, CapabilityElevationResource::reference())
        .operation_read_relation(operation, CapabilityElevationReview::reference())
        .operation_read_relation(operation, CapabilityReviewResource::reference())
        .operation_read_relation(operation, CapabilityReviewer::reference())
        .operation_write(operation, CapabilityElevationStatusField::reference())
        .operation_link(operation, CapabilityElevationApprover::reference())
        .capability(contract())
}

fn contract() -> ApplicationCapabilityContract<
    IdentityExecutionSchema,
    ApproveElevationCapability,
    ApproveCapabilityElevationOperation,
    ApproveElevationInput,
> {
    ApplicationCapabilityContractBuilder::new(
        ApproveElevationCapability::reference(),
        ApproveCapabilityElevationOperation::reference(),
        CapabilityGrant::reference(),
    )
    .target(command_target(CapabilityAction::ApproveElevation))
    .constraints(command_constraints())
    .delegation(delegation())
    .composition(composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

fn composition() -> ApplicationCapabilityComposition {
    let allow = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(CapabilityGrantor::reference())
        .forward(CapabilityResource::reference())
        .allow(Account::reference());
    let conflict = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(CapabilityConflictingBeneficiary::reference())
        .deny(Account::reference());
    let requester = ApplicationCapabilityGraphClause::new(
        ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
            .forward(CapabilityElevationRequester::reference())
            .forward(CapabilityElevationGrant::reference())
            .forward(CapabilityResource::reference())
            .deny(Account::reference()),
    )
    .anchored([ApplicationCapabilityPathContextAnchor::after_forward(
        CapabilityElevationRequester::reference(),
        CapabilityElevationSlot::reference(),
    )]);
    let graph =
        |path| ApplicationCapabilityGraphRule::any([ApplicationCapabilityGraphClause::new(path)]);
    ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(graph(allow)),
            ApplicationCapabilityDenyRule::not_applicable(),
            ApplicationCapabilityConflictRule::when(graph(conflict)),
        ),
        ApplicationCapabilityActorComposition::new(
            ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
            ApplicationCapabilityDistinctActorRule::when(ApplicationCapabilityGraphRule::any([
                requester,
            ])),
        ),
        command_propagation(),
    )
}
