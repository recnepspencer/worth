use worth_query_declaration::facade::application_capability::ApplicationCapabilityAcceptedValues;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityActorComposition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityAllowRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityCardinalityDimension;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityComposition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityConflictRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityConstraintDefinition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityContract;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityContractBuilder;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityCurrentnessDefinition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDecisionComposition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationDefinition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationDepth;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDenyRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDisclosureRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityDistinctActorRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityFieldBinding;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityFieldDimension;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityGraphClause;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityGraphRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityMagnitudeDimension;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityPropagationComposition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRelationBinding;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRelationDimension;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityScopeGuard;
use worth_query_declaration::facade::application_capability::ApplicationCapabilitySeparationOfDutyRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityTargetDefinition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityDefinition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValueBinding;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityWorkflowDefinition;
use worth_query_declaration::facade::application_schema::ApplicationAuthorizationPathBuilder;
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
use worth_query_declaration::facade::application_schema::ApplicationOperationDefinition;
use worth_query_declaration::facade::application_schema::ApplicationOperationRef;
use worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder;

pub fn without_external_effect_or_aftermath<Operation, Input>(
    operation: ApplicationOperationRef<IdentityExecutionSchema, Operation, Input>,
) -> ApplicationOperationDefinition<IdentityExecutionSchema, Operation, Input>
where
    Input: worth_query_declaration::facade::portable_identity::WorthQueryPortableType,
{
    operation
        .definition()
        .no_external_effect()
        .no_aftermath()
        .finish()
}

use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityAction;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityActionBinding;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityActionField;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityAmountField;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityConflictingBeneficiary;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityCustodian;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDelegationLimitField;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosureBinding;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosureField;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityGrant;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityGrantee;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityGrantor;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityNotAfterField;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityNotBeforeField;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityParent;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityProvenance;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityPurpose;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityPurposeBinding;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityPurposeField;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityRequestContext;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityResource;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityStatus;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityStatusBinding;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityStatusField;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityWorkflowField;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::ApproveCapabilityElevationOperation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationApprover;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationClosedAt;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationFacts;
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
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReview;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewFacts;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewIdentity;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewKindField;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewResource;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewReviewedAt;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewSlot;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewStatusField;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewer;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CompleteCapabilityReviewOperation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::ElevatedCapabilityTouchInput;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::ElevatedCapabilityTouchOperation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::ElevatedTouchAccountCapability;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::RequestCapabilityElevationOperation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::RevokeCapabilityElevationOperation;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use crate::domain_computation::primary_graph::tests::fixture::AccountLabel;
use crate::domain_computation::primary_graph::tests::fixture::AccountStatus;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::Principal;

pub use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::propagation::command_propagation;
pub use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::propagation::propagation;

pub fn install(
    schema: ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
) -> ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema> {
    let schema = schema
        .entity(CapabilityElevation::reference())
        .aspect(
            CapabilityElevation::reference(),
            CapabilityElevationFacts::reference(),
        )
        .field(
            CapabilityElevation::reference(),
            CapabilityElevationIdentity::reference(),
        )
        .field(
            CapabilityElevation::reference(),
            CapabilityElevationReason::reference(),
        )
        .field(
            CapabilityElevation::reference(),
            CapabilityElevationStatusField::reference(),
        )
        .field(
            CapabilityElevation::reference(),
            CapabilityElevationNotBefore::reference(),
        )
        .field(
            CapabilityElevation::reference(),
            CapabilityElevationNotAfter::reference(),
        )
        .field(
            CapabilityElevation::reference(),
            CapabilityElevationClosedAt::reference(),
        )
        .entity(CapabilityReview::reference())
        .aspect(
            CapabilityReview::reference(),
            CapabilityReviewFacts::reference(),
        )
        .field(
            CapabilityReview::reference(),
            CapabilityReviewIdentity::reference(),
        )
        .field(
            CapabilityReview::reference(),
            CapabilityReviewKindField::reference(),
        )
        .field(
            CapabilityReview::reference(),
            CapabilityReviewStatusField::reference(),
        )
        .field(
            CapabilityReview::reference(),
            CapabilityReviewReviewedAt::reference(),
        )
        .relation(
            CapabilityElevationRequester::reference(),
            Principal::reference(),
            CapabilityElevation::reference(),
        )
        .relation(
            CapabilityElevationApprover::reference(),
            Principal::reference(),
            CapabilityElevation::reference(),
        )
        .relation(
            CapabilityElevationGrant::reference(),
            CapabilityElevation::reference(),
            CapabilityGrant::reference(),
        )
        .relation(
            CapabilityElevationResource::reference(),
            CapabilityElevation::reference(),
            Account::reference(),
        )
        .relation(
            CapabilityElevationReview::reference(),
            CapabilityElevation::reference(),
            CapabilityReview::reference(),
        )
        .relation(
            CapabilityReviewResource::reference(),
            CapabilityReview::reference(),
            Account::reference(),
        )
        .relation(
            CapabilityReviewer::reference(),
            Principal::reference(),
            CapabilityReview::reference(),
        )
        .capability_context_entity_slot(CapabilityElevationSlot::reference())
        .capability_context_entity_slot(CapabilityReviewSlot::reference())
        .operation(without_external_effect_or_aftermath(
            ElevatedCapabilityTouchOperation::reference(),
        ))
        .operation(without_external_effect_or_aftermath(
            RequestCapabilityElevationOperation::reference(),
        ))
        .operation(without_external_effect_or_aftermath(
            ApproveCapabilityElevationOperation::reference(),
        ))
        .operation(without_external_effect_or_aftermath(
            RevokeCapabilityElevationOperation::reference(),
        ))
        .operation(without_external_effect_or_aftermath(
            CompleteCapabilityReviewOperation::reference(),
        ))
        .operation_projection_work_budget(ElevatedCapabilityTouchOperation::reference(), 32)
        .operation_read_field(
            ElevatedCapabilityTouchOperation::reference(),
            AccountIdentity::reference(),
        )
        .operation_read_field(
            ElevatedCapabilityTouchOperation::reference(),
            AccountLabel::reference(),
        )
        .operation_write(
            ElevatedCapabilityTouchOperation::reference(),
            AccountLabel::reference(),
        )
        .capability(elevated_contract());
    let schema = crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::request::install(schema);
    let schema = crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::approval::install(schema);
    let schema = crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::close::install(schema);
    crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::review::install(schema)
}

pub fn elevated_contract() -> ApplicationCapabilityContract<
    IdentityExecutionSchema,
    ElevatedTouchAccountCapability,
    ElevatedCapabilityTouchOperation,
    ElevatedCapabilityTouchInput,
> {
    ApplicationCapabilityContractBuilder::new(
        ElevatedTouchAccountCapability::reference(),
        ElevatedCapabilityTouchOperation::reference(),
        CapabilityGrant::reference(),
    )
    .target(target())
    .constraints(constraints())
    .delegation(delegation())
    .composition(composition())
    .elevation(crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::elevation::definition())
    .build()
}

pub fn target() -> ApplicationCapabilityTargetDefinition {
    ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            CapabilityActionField::reference(),
            ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(
                CapabilityAction::Touch,
            )
            .expect("fixture capability action must encode"),
        ),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::bound(CapabilityDisclosureField::reference()),
        ApplicationCapabilityValueBinding::new(
            CapabilityPurposeField::reference(),
            ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(
                CapabilityPurpose::AccountMaintenance,
            )
            .expect("fixture capability purpose must encode"),
        ),
    )
}

pub fn command_target(action: CapabilityAction) -> ApplicationCapabilityTargetDefinition {
    ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            CapabilityActionField::reference(),
            ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(action)
                .expect("fixture capability action must encode"),
        ),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            CapabilityPurposeField::reference(),
            ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(
                CapabilityPurpose::AccountMaintenance,
            )
            .expect("fixture capability purpose must encode"),
        ),
    )
}

pub fn constraints() -> ApplicationCapabilityConstraintDefinition {
    ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::bound(CapabilityAmountField::reference()),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                CapabilityStatusField::reference(),
                ApplicationEncodedScalarValue::<CapabilityStatusBinding>::try_new(
                    CapabilityStatus::Active,
                )
                .expect("fixture capability status must encode"),
            ),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(
                    CapabilityWorkflowField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(AccountStatus::reference()),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(
                    CapabilityNotBeforeField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(
                    CapabilityNotAfterField::reference(),
                ),
            ),
        ),
        CapabilityRequestContext::reference(),
    )
}

pub fn command_constraints() -> ApplicationCapabilityConstraintDefinition {
    ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                CapabilityStatusField::reference(),
                ApplicationEncodedScalarValue::<CapabilityStatusBinding>::try_new(
                    CapabilityStatus::Active,
                )
                .expect("fixture capability status must encode"),
            ),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(
                    CapabilityWorkflowField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(AccountStatus::reference()),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(
                    CapabilityNotBeforeField::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(
                    CapabilityNotAfterField::reference(),
                ),
            ),
        ),
        CapabilityRequestContext::reference(),
    )
}

pub fn delegation() -> ApplicationCapabilityDelegationDefinition {
    ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(CapabilityParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(
            CapabilityDelegationLimitField::reference(),
        ),
        CapabilityProvenance::reference(),
    )
}

pub fn composition() -> ApplicationCapabilityComposition {
    composition_with_propagation(propagation())
}

pub fn command_composition() -> ApplicationCapabilityComposition {
    composition_with_propagation(command_propagation())
}

pub fn composition_with_propagation(
    propagation: ApplicationCapabilityPropagationComposition,
) -> ApplicationCapabilityComposition {
    let grantor = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(CapabilityGrantor::reference())
        .forward(CapabilityResource::reference())
        .allow(Account::reference());
    let custodian = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(CapabilityCustodian::reference())
        .forward(CapabilityResource::reference())
        .allow(Account::reference());
    let conflict = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(CapabilityConflictingBeneficiary::reference())
        .deny(Account::reference());
    ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(ApplicationCapabilityGraphRule::any([
                ApplicationCapabilityGraphClause::new(grantor),
                ApplicationCapabilityGraphClause::new(custodian),
            ])),
            ApplicationCapabilityDenyRule::not_applicable(),
            ApplicationCapabilityConflictRule::when(ApplicationCapabilityGraphRule::any([
                ApplicationCapabilityGraphClause::new(conflict),
            ])),
        ),
        ApplicationCapabilityActorComposition::new(
            ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
            ApplicationCapabilityDistinctActorRule::not_applicable(), worth_query_declaration::facade::application_capability::ApplicationCapabilityRecordedActorRule::not_applicable()),
        propagation,
    )
}

#[path = "approval/mod.rs"]
pub mod approval;
#[path = "close/mod.rs"]
pub mod close;
#[path = "elevation/mod.rs"]
pub mod elevation;
#[path = "propagation/mod.rs"]
pub mod propagation;
#[path = "request/mod.rs"]
pub mod request;
#[path = "review/mod.rs"]
pub mod review;
