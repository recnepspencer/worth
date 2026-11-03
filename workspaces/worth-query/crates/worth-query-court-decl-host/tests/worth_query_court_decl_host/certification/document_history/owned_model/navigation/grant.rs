//! Ordinary recorded grant and actor path for generic model snapshot navigation.
use super::*;
worth_query_entity!(pub Grant for Schema);
worth_query_aspect!(pub Facts for Schema, Grant; identity = AspectIdentity(0x91820305), revision = AspectContractRevision(1));
worth_query_field!(pub Action for Schema, Grant, Facts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub Purpose for Schema, Grant, Facts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub Status for Schema, Grant, Facts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub Workflow for Schema, Grant, Facts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub Before for Schema, Grant, Facts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub After for Schema, Grant, Facts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub Depth for Schema, Grant, Facts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_relation!(pub Owner in Schema, Principal => Model; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub Resource in Schema, Grant => Model; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub Parent in Schema, Grant => Grant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub Grantor in Schema, Principal => Grant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub Grantee in Schema, Principal => Grant; integrity = same_context_unbounded_retain_dangling);
pub fn text(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value.to_owned()).unwrap()
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .entity(Grant::reference())
        .aspect(Grant::reference(), Facts::reference())
        .field(Grant::reference(), Action::reference())
        .field(Grant::reference(), Purpose::reference())
        .field(Grant::reference(), Status::reference())
        .field(Grant::reference(), Workflow::reference())
        .field(Grant::reference(), Before::reference())
        .field(Grant::reference(), After::reference())
        .field(Grant::reference(), Depth::reference())
        .relation(
            Owner::reference(),
            Principal::reference(),
            Model::reference(),
        )
        .relation(
            Resource::reference(),
            Grant::reference(),
            Model::reference(),
        )
        .relation(Parent::reference(), Grant::reference(), Grant::reference())
        .relation(
            Grantor::reference(),
            Principal::reference(),
            Grant::reference(),
        )
        .relation(
            Grantee::reference(),
            Principal::reference(),
            Grant::reference(),
        )
}
pub fn contract(
) -> ApplicationCapabilityContract<Schema, Navigate, SelectSnapshot, query::model::Read> {
    let allow = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(Owner::reference())
        .allow(Model::reference());
    ApplicationCapabilityContractBuilder::new(
        Navigate::reference(),
        SelectSnapshot::reference(),
        Grant::reference(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(Action::reference(), text("navigate")),
        ApplicationCapabilityRelationBinding::from_reference(Resource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(Purpose::reference(), text("history")),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(Status::reference(), text("active")),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(Workflow::reference()),
                ApplicationCapabilityFieldBinding::from_reference(ModelKey::reference()),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(Before::reference()),
                ApplicationCapabilityFieldBinding::from_reference(After::reference()),
            ),
        ),
        Context::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(Parent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(Grantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(Grantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(Depth::reference()),
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
            ApplicationCapabilityDisclosureRule::not_applicable(),
        ),
    ))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}
pub fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Grant::reference(),
            WorthQueryApplicationEntityKey::new("navigation-grant").unwrap(),
        )
        .field(Action::reference(), "navigate".to_owned())
        .field(Purpose::reference(), "history".to_owned())
        .field(Status::reference(), "active".to_owned())
        .field(Workflow::reference(), "a".to_owned())
        .field(Before::reference(), 0)
        .field(After::reference(), 4_000_000_000)
        .field(Depth::reference(), 0),
    )?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        Owner::reference(),
        "navigation-owner",
        WorthQueryApplicationEntityKey::new("operator").unwrap(),
        WorthQueryApplicationEntityKey::new("a").unwrap(),
    ))?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        Resource::reference(),
        "navigation-resource",
        WorthQueryApplicationEntityKey::new("navigation-grant").unwrap(),
        WorthQueryApplicationEntityKey::new("a").unwrap(),
    ))?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        Grantor::reference(),
        "navigation-grantor",
        WorthQueryApplicationEntityKey::new("operator").unwrap(),
        WorthQueryApplicationEntityKey::new("navigation-grant").unwrap(),
    ))?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        Grantee::reference(),
        "navigation-grantee",
        WorthQueryApplicationEntityKey::new("operator").unwrap(),
        WorthQueryApplicationEntityKey::new("navigation-grant").unwrap(),
    ))?;
    Ok(())
}
