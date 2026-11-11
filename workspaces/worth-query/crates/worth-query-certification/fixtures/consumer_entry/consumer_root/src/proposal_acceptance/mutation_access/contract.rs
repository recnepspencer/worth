//! The existing capability owner binds the edit to current model-scoped grant truth.
use super::{grants::*, *};
pub(super) fn definition(
) -> ApplicationCapabilityContract<Schema, EditRetention, retention::SetRetention, RetentionEdit> {
    let path = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(MutationGrantGrantee::reference())
        .forward(MutationGrantResource::reference())
        .allow(Model::reference());
    ApplicationCapabilityContractBuilder::new(
        EditRetention::reference(),
        retention::SetRetention::reference(),
        MutationGrant::reference(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(GrantAction::at("GrantAction"), encoded(2)).unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(MutationGrantResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(GrantPurpose::at("GrantPurpose"), encoded(1))
            .unwrap(),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(GrantStatus::at("GrantStatus"), encoded(1))
                .unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(GrantScope::at("GrantScope"))
                    .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(ModelName::at("ModelName"))
                    .unwrap(),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(GrantNotBefore::at(
                    "GrantNotBefore",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(GrantNotAfter::at(
                    "GrantNotAfter",
                ))
                .unwrap(),
            ),
        ),
        EditContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(MutationGrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(MutationGrantGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(MutationGrantGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(GrantDepth::at("GrantDepth")).unwrap(),
        EditProvenance::reference(),
    ))
    .composition(ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(ApplicationCapabilityGraphRule::any([
                ApplicationCapabilityGraphClause::new(path),
            ])),
            ApplicationCapabilityDenyRule::not_applicable(),
            ApplicationCapabilityConflictRule::not_applicable(),
        ),
        ApplicationCapabilityActorComposition::new(
            ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
            ApplicationCapabilityDistinctActorRule::not_applicable(),
            ApplicationCapabilityRecordedActorRule::not_applicable(),
        ),
        ApplicationCapabilityPropagationComposition::new(
            ApplicationCapabilityDelegationRule::forbidden(),
            ApplicationCapabilityDisclosureRule::not_applicable(),
        ),
    ))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}
