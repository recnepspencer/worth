use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityAcceptedValues, ApplicationCapabilityActorComposition,
        ApplicationCapabilityAllowRule, ApplicationCapabilityCardinalityDimension,
        ApplicationCapabilityComposition, ApplicationCapabilityConflictRule,
        ApplicationCapabilityConstraintDefinition, ApplicationCapabilityContract,
        ApplicationCapabilityContractBuilder, ApplicationCapabilityCurrentnessDefinition,
        ApplicationCapabilityDecisionComposition, ApplicationCapabilityDelegationDefinition,
        ApplicationCapabilityDelegationDepth, ApplicationCapabilityDelegationRule,
        ApplicationCapabilityDenyRule, ApplicationCapabilityDisclosureRule,
        ApplicationCapabilityDistinctActorRule, ApplicationCapabilityFieldBinding,
        ApplicationCapabilityFieldDimension, ApplicationCapabilityGraphClause,
        ApplicationCapabilityGraphRule, ApplicationCapabilityMagnitudeDimension,
        ApplicationCapabilityPropagationComposition, ApplicationCapabilityRelationBinding,
        ApplicationCapabilityRelationDimension, ApplicationCapabilityScopeGuard,
        ApplicationCapabilitySeparationOfDutyRule, ApplicationCapabilityTargetDefinition,
        ApplicationCapabilityValidityDefinition, ApplicationCapabilityValidityTimeline,
        ApplicationCapabilityValueBinding, ApplicationCapabilityWorkflowDefinition,
    },
    application_schema::{
        ApplicationAuthorizationPathBuilder, ApplicationEncodedScalarValue,
        ApplicationOperationDefinition, ApplicationOperationRef,
        ApplicationSchemaDeclarationBuilder,
    },
};

fn without_external_effect_or_aftermath<Operation, Input>(
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

use super::super::super::{
    Account, AccountLabel, AccountStatus, IdentityExecutionSchema, Principal,
};
use super::super::declaration::{
    CapabilityAction, CapabilityActionBinding, CapabilityActionField, CapabilityAmountField,
    CapabilityConflictingBeneficiary, CapabilityCustodian, CapabilityDelegationLimitField,
    CapabilityDisclosure, CapabilityDisclosureBinding, CapabilityDisclosureField, CapabilityGrant,
    CapabilityGrantee, CapabilityGrantor, CapabilityNotAfterField, CapabilityNotBeforeField,
    CapabilityParent, CapabilityProvenance, CapabilityPurpose, CapabilityPurposeBinding,
    CapabilityPurposeField, CapabilityRequestContext, CapabilityResource, CapabilityStatus,
    CapabilityStatusBinding, CapabilityStatusField, CapabilityWorkflowField,
};
use super::{
    ApproveCapabilityElevationOperation, CapabilityElevation, CapabilityElevationApprover,
    CapabilityElevationFacts, CapabilityElevationGrant, CapabilityElevationIdentity,
    CapabilityElevationNotAfter, CapabilityElevationNotBefore, CapabilityElevationReason,
    CapabilityElevationRequester, CapabilityElevationResource, CapabilityElevationReview,
    CapabilityElevationSlot, CapabilityElevationStatusField, CapabilityReview,
    CapabilityReviewFacts, CapabilityReviewIdentity, CapabilityReviewKindField,
    CapabilityReviewResource, CapabilityReviewSlot, CapabilityReviewStatusField,
    CapabilityReviewer, CompleteCapabilityReviewOperation, ElevatedCapabilityTouchInput,
    ElevatedCapabilityTouchOperation, ElevatedTouchAccountCapability,
    RequestCapabilityElevationOperation, RevokeCapabilityElevationOperation,
};

#[path = "contract/approval.rs"]
mod approval;
#[path = "contract/close.rs"]
mod close;
#[path = "contract/elevation.rs"]
mod elevation;
#[path = "contract/request.rs"]
mod request;
#[path = "contract/review.rs"]
mod review;

pub(in crate::domain_computation::primary_graph::tests::fixture::capability) fn install(
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
        .operation_decision_fact_budget(ElevatedCapabilityTouchOperation::reference(), 1)
        .operation_projection_work_budget(ElevatedCapabilityTouchOperation::reference(), 32)
        .operation_read_field(
            ElevatedCapabilityTouchOperation::reference(),
            AccountLabel::reference(),
        )
        .operation_write(
            ElevatedCapabilityTouchOperation::reference(),
            AccountLabel::reference(),
        )
        .capability(elevated_contract());
    let schema = request::install(schema);
    let schema = approval::install(schema);
    let schema = close::install(schema);
    review::install(schema)
}

fn elevated_contract() -> ApplicationCapabilityContract<
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
    .elevation(elevation::definition())
    .build()
}

pub(super) fn target() -> ApplicationCapabilityTargetDefinition {
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

pub(super) fn command_target(action: CapabilityAction) -> ApplicationCapabilityTargetDefinition {
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

pub(super) fn constraints() -> ApplicationCapabilityConstraintDefinition {
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

pub(super) fn command_constraints() -> ApplicationCapabilityConstraintDefinition {
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

pub(super) fn delegation() -> ApplicationCapabilityDelegationDefinition {
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

pub(super) fn composition() -> ApplicationCapabilityComposition {
    composition_with_propagation(propagation())
}

pub(super) fn command_composition() -> ApplicationCapabilityComposition {
    composition_with_propagation(command_propagation())
}

fn composition_with_propagation(
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
            ApplicationCapabilityDistinctActorRule::not_applicable(),
        ),
        propagation,
    )
}

pub(super) fn propagation() -> ApplicationCapabilityPropagationComposition {
    ApplicationCapabilityPropagationComposition::new(
        ApplicationCapabilityDelegationRule::narrow_all_dimensions(
            ApplicationCapabilityDelegationDepth::new(2).unwrap(),
        ),
        ApplicationCapabilityDisclosureRule::permit([ApplicationCapabilityScopeGuard::requiring(
            [ApplicationCapabilityAcceptedValues::one_of(
                CapabilityDisclosureField::reference(),
                [
                    ApplicationEncodedScalarValue::<CapabilityDisclosureBinding>::try_new(
                        CapabilityDisclosure::AccountActivity,
                    )
                    .expect("fixture capability disclosure must encode"),
                ],
            )],
        )]),
    )
}

pub(super) fn command_propagation() -> ApplicationCapabilityPropagationComposition {
    ApplicationCapabilityPropagationComposition::new(
        ApplicationCapabilityDelegationRule::narrow_all_dimensions(
            ApplicationCapabilityDelegationDepth::new(2).unwrap(),
        ),
        ApplicationCapabilityDisclosureRule::not_applicable(),
    )
}
