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
    Account, AccountIdentity, AccountLabel, AccountStatus, IdentityExecutionSchema, Principal,
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
    ApproveCapabilityElevationOperation, CapabilityElevationSlot, CapabilityReviewSlot,
    CompleteCapabilityReviewOperation, ElevatedCapabilityTouchInput,
    ElevatedCapabilityTouchOperation, ElevatedTouchAccountCapability,
    RequestCapabilityElevationOperation, RevokeCapabilityElevationOperation,
};

#[path = "contract/approval.rs"]
mod approval;
#[path = "contract/close.rs"]
mod close;
#[path = "contract/elevation.rs"]
mod elevation;
#[path = "contract/propagation.rs"]
mod propagation;
#[path = "contract/request.rs"]
mod request;
#[path = "contract/review.rs"]
mod review;
pub(super) use propagation::{command_propagation, propagation};

pub(in crate::domain_computation::primary_graph::tests::fixture::capability) fn install(
    schema: ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
) -> ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema> {
    let schema = schema
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
            AccountIdentity::at("AccountIdentity"),
        )
        .operation_read_field(
            ElevatedCapabilityTouchOperation::reference(),
            AccountLabel::at("AccountLabel"),
        )
        .operation_write(
            ElevatedCapabilityTouchOperation::reference(),
            AccountLabel::at("AccountLabel"),
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
            CapabilityActionField::at("CapabilityActionField"),
            ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(
                CapabilityAction::Touch,
            )
            .expect("fixture capability action must encode"),
        )
        .unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::bound(CapabilityDisclosureField::at(
            "CapabilityDisclosureField",
        ))
        .unwrap(),
        ApplicationCapabilityValueBinding::new(
            CapabilityPurposeField::at("CapabilityPurposeField"),
            ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(
                CapabilityPurpose::AccountMaintenance,
            )
            .expect("fixture capability purpose must encode"),
        )
        .unwrap(),
    )
}

pub(super) fn command_target(action: CapabilityAction) -> ApplicationCapabilityTargetDefinition {
    ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            CapabilityActionField::at("CapabilityActionField"),
            ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(action)
                .expect("fixture capability action must encode"),
        )
        .unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(CapabilityResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            CapabilityPurposeField::at("CapabilityPurposeField"),
            ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(
                CapabilityPurpose::AccountMaintenance,
            )
            .expect("fixture capability purpose must encode"),
        )
        .unwrap(),
    )
}

pub(super) fn constraints() -> ApplicationCapabilityConstraintDefinition {
    ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::bound(CapabilityAmountField::at(
            "CapabilityAmountField",
        ))
        .expect("record field contract"),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                CapabilityStatusField::at("CapabilityStatusField"),
                ApplicationEncodedScalarValue::<CapabilityStatusBinding>::try_new(
                    CapabilityStatus::Active,
                )
                .expect("fixture capability status must encode"),
            )
            .unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(CapabilityWorkflowField::at(
                    "CapabilityWorkflowField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(AccountStatus::at(
                    "AccountStatus",
                ))
                .unwrap(),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(CapabilityNotBeforeField::at(
                    "CapabilityNotBeforeField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(CapabilityNotAfterField::at(
                    "CapabilityNotAfterField",
                ))
                .unwrap(),
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
                CapabilityStatusField::at("CapabilityStatusField"),
                ApplicationEncodedScalarValue::<CapabilityStatusBinding>::try_new(
                    CapabilityStatus::Active,
                )
                .expect("fixture capability status must encode"),
            )
            .unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(CapabilityWorkflowField::at(
                    "CapabilityWorkflowField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(AccountStatus::at(
                    "AccountStatus",
                ))
                .unwrap(),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(CapabilityNotBeforeField::at(
                    "CapabilityNotBeforeField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(CapabilityNotAfterField::at(
                    "CapabilityNotAfterField",
                ))
                .unwrap(),
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
        ApplicationCapabilityFieldBinding::from_reference(CapabilityDelegationLimitField::at(
            "CapabilityDelegationLimitField",
        ))
        .unwrap(),
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
            ApplicationCapabilityDistinctActorRule::not_applicable(), worth_query_declaration::facade::application_capability::ApplicationCapabilityRecordedActorRule::not_applicable()),
        propagation,
    )
}
