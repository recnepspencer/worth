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
use worth_query_declaration::facade::application_capability::ApplicationCapabilityGraphRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityPathContextAnchor;
use worth_query_declaration::facade::application_capability::ApplicationCapabilitySeparationOfDutyRule;
use worth_query_declaration::facade::application_schema::ApplicationAuthorizationPathBuilder;
use worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityConflictingBeneficiary;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationApprover;
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
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewReviewedAt;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewSlot;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewStatusField;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewer;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CompleteCapabilityReviewOperation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CompleteElevationReviewCapability;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::review::CompleteElevationReviewInput;
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
    let operation = CompleteCapabilityReviewOperation::reference();
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
        .operation_read_field(operation, CapabilityReviewReviewedAt::reference())
        .operation_read_relation(operation, CapabilityElevationRequester::reference())
        .operation_read_relation(operation, CapabilityElevationApprover::reference())
        .operation_read_relation(operation, CapabilityElevationGrant::reference())
        .operation_read_relation(operation, CapabilityElevationResource::reference())
        .operation_read_relation(operation, CapabilityElevationReview::reference())
        .operation_read_relation(operation, CapabilityReviewResource::reference())
        .operation_read_relation(operation, CapabilityReviewer::reference())
        .operation_write(operation, CapabilityReviewStatusField::reference())
        .operation_write(operation, CapabilityReviewReviewedAt::reference())
        .operation_link(operation, CapabilityReviewer::reference())
        .capability(contract())
}

pub fn contract() -> ApplicationCapabilityContract<
    IdentityExecutionSchema,
    CompleteElevationReviewCapability,
    CompleteCapabilityReviewOperation,
    CompleteElevationReviewInput,
> {
    ApplicationCapabilityContractBuilder::new(
        CompleteElevationReviewCapability::reference(),
        CompleteCapabilityReviewOperation::reference(),
        CapabilityGrant::reference(),
    )
    .target(command_target(CapabilityAction::CompleteReview))
    .constraints(command_constraints())
    .delegation(delegation())
    .composition(composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

pub fn composition() -> ApplicationCapabilityComposition {
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
            .forward(CapabilityElevationReview::reference())
            .forward(CapabilityReviewResource::reference())
            .deny(Account::reference()),
    )
    .anchored([
        ApplicationCapabilityPathContextAnchor::after_forward(
            CapabilityElevationRequester::reference(),
            CapabilityElevationSlot::reference(),
        ),
        ApplicationCapabilityPathContextAnchor::after_forward(
            CapabilityElevationReview::reference(),
            CapabilityReviewSlot::reference(),
        ),
    ]);
    let approver = ApplicationCapabilityGraphClause::new(
        ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
            .forward(CapabilityElevationApprover::reference())
            .forward(CapabilityElevationReview::reference())
            .forward(CapabilityReviewResource::reference())
            .deny(Account::reference()),
    )
    .anchored([
        ApplicationCapabilityPathContextAnchor::after_forward(
            CapabilityElevationApprover::reference(),
            CapabilityElevationSlot::reference(),
        ),
        ApplicationCapabilityPathContextAnchor::after_forward(
            CapabilityElevationReview::reference(),
            CapabilityReviewSlot::reference(),
        ),
    ]);
    ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(ApplicationCapabilityGraphRule::any([
                ApplicationCapabilityGraphClause::new(allow),
            ])),
            ApplicationCapabilityDenyRule::not_applicable(),
            ApplicationCapabilityConflictRule::when(ApplicationCapabilityGraphRule::any([
                ApplicationCapabilityGraphClause::new(conflict),
            ])),
        ),
        ApplicationCapabilityActorComposition::new(
            ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
            ApplicationCapabilityDistinctActorRule::when(ApplicationCapabilityGraphRule::any([
                requester, approver,
            ])), worth_query_declaration::facade::application_capability::ApplicationCapabilityRecordedActorRule::not_applicable()),
        command_propagation(),
    )
}
