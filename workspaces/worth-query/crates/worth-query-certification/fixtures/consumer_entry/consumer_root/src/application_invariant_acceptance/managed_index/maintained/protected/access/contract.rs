//! Schema-bound fixture for managed_index::maintained::protected::access.
//! The installed capability contract binds action, purpose, scope, currentness and disclosure.
use super::grants::*;
use super::*;
pub(super) fn definition(
) -> ApplicationCapabilityContract<Catalog, ReadBoxes, InspectBoxes, ReadAccess> {
    let path = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(GrantGrantee::reference())
        .forward(GrantResource::reference())
        .allow(Directory::reference());
    ApplicationCapabilityContractBuilder::new(
        ReadBoxes::reference(),
        InspectBoxes::reference(),
        BoxGrant::reference(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(GrantAction::at("GrantAction"), encoded()).unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(GrantResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::bound(GrantDisclosure::at("GrantDisclosure")).unwrap(),
        ApplicationCapabilityValueBinding::new(GrantPurpose::at("GrantPurpose"), encoded())
            .unwrap(),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(GrantStatus::at("GrantStatus"), encoded())
                .unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(GrantScope::at("GrantScope"))
                    .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(DirectoryIdentity::at(
                    "DirectoryIdentity",
                ))
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
        BoxContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(GrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(GrantGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(GrantGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(GrantDepth::at("GrantDepth")).unwrap(),
        BoxProvenance::reference(),
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
                        [encoded()],
                    )
                    .unwrap(),
                ]),
            ]),
        ),
    ))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}
