use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityActorComposition, ApplicationCapabilityAllowRule,
        ApplicationCapabilityComposition, ApplicationCapabilityConflictRule,
        ApplicationCapabilityContract, ApplicationCapabilityContractBuilder,
        ApplicationCapabilityDecisionComposition, ApplicationCapabilityDenyRule,
        ApplicationCapabilityDistinctActorRule, ApplicationCapabilityElevationRule,
        ApplicationCapabilityGraphClause, ApplicationCapabilityGraphRequirement,
        ApplicationCapabilityGraphRule, ApplicationCapabilityPathContextAnchor,
        ApplicationCapabilitySeparationOfDutyRule,
    },
    application_schema::{
        ApplicationAuthorizationPathBuilder, ApplicationSchemaDeclarationBuilder,
    },
};

use super::super::{
    CapabilityConflictingBeneficiary, CapabilityElevationApprover, CapabilityElevationClosedAt,
    CapabilityElevationGrant, CapabilityElevationIdentity, CapabilityElevationNotAfter,
    CapabilityElevationNotBefore, CapabilityElevationReason, CapabilityElevationRequester,
    CapabilityElevationResource, CapabilityElevationReview, CapabilityElevationSlot,
    CapabilityElevationStatusField, CapabilityGrant, CapabilityGrantor, CapabilityResource,
    CapabilityReviewIdentity, CapabilityReviewKindField, CapabilityReviewResource,
    CapabilityReviewStatusField, CapabilityReviewer, CloseElevationInput,
    RevokeCapabilityElevationOperation, RevokeElevationCapability,
};
use super::{command_constraints, command_propagation, command_target, delegation};
use crate::domain_computation::primary_graph::tests::fixture::{
    Account, CapabilityAction, IdentityExecutionSchema, Principal,
};

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
) -> ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema> {
    let operation = RevokeCapabilityElevationOperation::reference();
    schema
        .operation_decision_fact_budget(operation, 16)
        .operation_projection_work_budget(operation, 96)
        .operation_read_field(operation, CapabilityElevationIdentity::reference())
        .operation_read_field(operation, CapabilityElevationReason::reference())
        .operation_read_field(operation, CapabilityElevationStatusField::reference())
        .operation_read_field(operation, CapabilityElevationNotBefore::reference())
        .operation_read_field(operation, CapabilityElevationNotAfter::reference())
        .operation_read_field(operation, CapabilityElevationClosedAt::reference())
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
        .operation_write(operation, CapabilityElevationClosedAt::reference())
        .capability(contract())
}

fn composition() -> ApplicationCapabilityComposition {
    let command = ApplicationCapabilityGraphClause::new(
        ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
            .forward(CapabilityGrantor::reference())
            .forward(CapabilityResource::reference())
            .allow(Account::reference()),
    );
    let exact_approver = ApplicationCapabilityGraphClause::new(
        ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
            .forward(CapabilityElevationApprover::reference())
            .forward(CapabilityElevationReview::reference())
            .forward(CapabilityReviewResource::reference())
            .allow(Account::reference()),
    )
    .anchored([ApplicationCapabilityPathContextAnchor::after_forward(
        CapabilityElevationApprover::reference(),
        CapabilityElevationSlot::reference(),
    )]);
    let conflict = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(CapabilityConflictingBeneficiary::reference())
        .deny(Account::reference());
    ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(ApplicationCapabilityGraphRule::all([
                ApplicationCapabilityGraphRequirement::any([command]),
                ApplicationCapabilityGraphRequirement::any([exact_approver]),
            ])),
            ApplicationCapabilityDenyRule::not_applicable(),
            ApplicationCapabilityConflictRule::when(ApplicationCapabilityGraphRule::any([
                ApplicationCapabilityGraphClause::new(conflict),
            ])),
        ),
        ApplicationCapabilityActorComposition::new(
            ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
            ApplicationCapabilityDistinctActorRule::not_applicable(),
        ),
        command_propagation(),
    )
}

fn contract() -> ApplicationCapabilityContract<
    IdentityExecutionSchema,
    RevokeElevationCapability,
    RevokeCapabilityElevationOperation,
    CloseElevationInput,
> {
    ApplicationCapabilityContractBuilder::new(
        RevokeElevationCapability::reference(),
        RevokeCapabilityElevationOperation::reference(),
        CapabilityGrant::reference(),
    )
    .target(command_target(CapabilityAction::RevokeElevation))
    .constraints(command_constraints())
    .delegation(delegation())
    .composition(composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}
