//! The existing capability owner admits action, model, currentness and disclosure.
use super::{grants::*, *};
pub(super) fn definition(
) -> ApplicationCapabilityContract<Schema, ReadDomain, InspectDomain, ReadAccess> {
    let path = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(GrantGrantee::reference())
        .forward(GrantResource::reference())
        .allow(Model::reference());
    ApplicationCapabilityContractBuilder::new(
        ReadDomain::reference(),
        InspectDomain::reference(),
        Grant::reference(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(GrantAction::at("GrantAction"), encoded(1)).unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(GrantResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::bound(GrantDisclosure::at("GrantDisclosure")).unwrap(),
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
        ReadContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(GrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(GrantGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(GrantGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(GrantDepth::at("GrantDepth")).unwrap(),
        ReadProvenance::reference(),
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
            ApplicationCapabilityDisclosureRule::permit([
                ApplicationCapabilityScopeGuard::requiring([
                    ApplicationCapabilityAcceptedValues::one_of(
                        GrantDisclosure::at("GrantDisclosure"),
                        [encoded(1)],
                    )
                    .unwrap(),
                ]),
            ]),
        ),
    ))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}
