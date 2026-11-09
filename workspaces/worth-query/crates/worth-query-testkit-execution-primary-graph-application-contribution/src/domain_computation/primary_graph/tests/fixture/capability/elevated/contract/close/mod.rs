use worth_query_declaration::facade::application_capability::ApplicationCapabilityActorComposition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityAllowRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityComposition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityConflictRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityContract;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityContractBuilder;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDecisionComposition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDenyRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDistinctActorRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityElevationRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityGraphClause;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityGraphRequirement;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityGraphRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityPathContextAnchor;
use worth_query_declaration::facade::application_capability::ApplicationCapabilitySeparationOfDutyRule;
use worth_query_declaration::facade::application_schema::ApplicationAuthorizationPathBuilder;
use worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityConflictingBeneficiary;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationApprover;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationClosedAt;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationGrant;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationIdentity;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationNotAfter;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationNotBefore;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationReason;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationRequester;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationResource;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationReview;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationSlot;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationStatusField;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityGrant;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityGrantor;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityResource;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewIdentity;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewKindField;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewResource;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewStatusField;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewer;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::close::CloseElevationInput;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::RevokeCapabilityElevationOperation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::RevokeElevationCapability;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::command_constraints;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::propagation::command_propagation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::command_target;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::delegation;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityAction;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::Principal;

pub fn install(
    schema: ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
) -> ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema> {
    let operation = RevokeCapabilityElevationOperation::reference();
    schema
        .operation_projection_work_budget(operation, 96)
        .operation_read_field(
            operation,
            CapabilityElevationIdentity::at("CapabilityElevationIdentity"),
        )
        .operation_read_field(
            operation,
            CapabilityElevationReason::at("CapabilityElevationReason"),
        )
        .operation_read_field(
            operation,
            CapabilityElevationStatusField::at("CapabilityElevationStatusField"),
        )
        .operation_read_field(
            operation,
            CapabilityElevationNotBefore::at("CapabilityElevationNotBefore"),
        )
        .operation_read_field(
            operation,
            CapabilityElevationNotAfter::at("CapabilityElevationNotAfter"),
        )
        .operation_read_field(
            operation,
            CapabilityElevationClosedAt::at("CapabilityElevationClosedAt"),
        )
        .operation_read_field(
            operation,
            CapabilityReviewIdentity::at("CapabilityReviewIdentity"),
        )
        .operation_read_field(
            operation,
            CapabilityReviewKindField::at("CapabilityReviewKindField"),
        )
        .operation_read_field(
            operation,
            CapabilityReviewStatusField::at("CapabilityReviewStatusField"),
        )
        .operation_read_relation(operation, CapabilityElevationRequester::reference())
        .operation_read_relation(operation, CapabilityElevationApprover::reference())
        .operation_read_relation(operation, CapabilityElevationGrant::reference())
        .operation_read_relation(operation, CapabilityElevationResource::reference())
        .operation_read_relation(operation, CapabilityElevationReview::reference())
        .operation_read_relation(operation, CapabilityReviewResource::reference())
        .operation_read_relation(operation, CapabilityReviewer::reference())
        .operation_write(
            operation,
            CapabilityElevationStatusField::at("CapabilityElevationStatusField"),
        )
        .operation_write(
            operation,
            CapabilityElevationClosedAt::at("CapabilityElevationClosedAt"),
        )
        .capability(contract())
}

pub fn composition() -> ApplicationCapabilityComposition {
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
            ApplicationCapabilityDistinctActorRule::not_applicable(), worth_query_declaration::facade::application_capability::ApplicationCapabilityRecordedActorRule::not_applicable()),
        command_propagation(),
    )
}

pub fn contract() -> ApplicationCapabilityContract<
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
