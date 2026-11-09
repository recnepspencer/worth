//! Original recorded inspection grant, independent of producer and historical READ grants.
use super::*;
use grant::{
    text, Action, After, Before, Depth, Grant, Grantee, Grantor, Owner, Parent, Purpose, Resource,
    Status, Workflow,
};
pub(super) fn definition(
) -> ApplicationCapabilityContract<Schema, Inspect, InspectOperation, query::model::Read> {
    let allow = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(Owner::reference())
        .allow(Model::reference());
    ApplicationCapabilityContractBuilder::new(
        Inspect::reference(),
        InspectOperation::reference(),
        Grant::reference(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(Action::at("Action"), text("inspect")).unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(Resource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::bound(Workflow::at("Workflow")).unwrap(),
        ApplicationCapabilityValueBinding::new(Purpose::at("Purpose"), text("audit")).unwrap(),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(Status::at("Status"), text("active")).unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(Workflow::at("Workflow"))
                    .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(ModelKey::at("ModelKey"))
                    .unwrap(),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(Before::at("Before")).unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(After::at("After")).unwrap(),
            ),
        ),
        Context::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(Parent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(Grantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(Grantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(Depth::at("Depth")).unwrap(),
        Provenance::reference(),
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
                        Workflow::at("Workflow"),
                        [text("a")],
                    )
                    .unwrap(),
                ]),
            ]),
        ),
    ))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}
pub(super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Grant::reference(),
            WorthQueryApplicationEntityKey::new("predecessor-inspection-grant").unwrap(),
        )
        .field(Action::at("Action"), "inspect".to_owned())
        .field(Purpose::at("Purpose"), "audit".to_owned())
        .field(Status::at("Status"), "active".to_owned())
        .field(Workflow::at("Workflow"), "a".to_owned())
        .field(Before::at("Before"), 0_u64)
        .field(After::at("After"), 4_000_000_000_u64)
        .field(Depth::at("Depth"), 0_u64),
    )?;
    for (name, grantor) in [
        ("predecessor-inspection-grantor", true),
        ("predecessor-inspection-grantee", false),
    ] {
        let actor = WorthQueryApplicationEntityKey::new("operator").unwrap();
        let grant = WorthQueryApplicationEntityKey::new("predecessor-inspection-grant").unwrap();
        if grantor {
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                Grantor::reference(),
                name,
                actor,
                grant,
            ))?;
        } else {
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                Grantee::reference(),
                name,
                actor,
                grant,
            ))?;
        }
    }
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        Resource::reference(),
        "predecessor-inspection-resource",
        WorthQueryApplicationEntityKey::new("predecessor-inspection-grant").unwrap(),
        WorthQueryApplicationEntityKey::new("a").unwrap(),
    ))
}
