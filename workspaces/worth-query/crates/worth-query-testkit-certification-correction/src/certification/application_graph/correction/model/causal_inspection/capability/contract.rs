//! The existing grant graph admits one current diagnostic classification.
use super::*;
pub(super) fn definition(
) -> ApplicationCapabilityContract<NumericSchema, Inspect, InspectCausalEvidence, Input> {
    let allow = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(GrantGrantor::reference())
        .forward(GrantResource::reference())
        .allow(Number::reference());
    ApplicationCapabilityContractBuilder::new(
        Inspect::reference(),
        InspectCausalEvidence::reference(),
        Grant::reference(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(GrantAction::at("GrantAction"), text("inspect"))
            .unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(GrantResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::bound(Disclosure::at("Disclosure")).unwrap(),
        ApplicationCapabilityValueBinding::new(GrantPurpose::at("GrantPurpose"), text("audit"))
            .unwrap(),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(GrantStatus::at("GrantStatus"), text("active"))
                .unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(GrantScopeKey::at(
                    "GrantScopeKey",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(Key::at("Key")).unwrap(),
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
        AdoptContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(GrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(GrantGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(GrantGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(GrantDepth::at("GrantDepth")).unwrap(),
        AdoptProvenance::reference(),
    ))
    .composition(ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(ApplicationCapabilityGraphRule::any([
                ApplicationCapabilityGraphClause::new(allow),
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
                        Disclosure::at("Disclosure"),
                        [text("calculation")],
                    )
                    .unwrap(),
                ]),
            ]),
        ),
    ))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}
