pub use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityActorComposition, ApplicationCapabilityAllowRule,
        ApplicationCapabilityCardinalityDimension, ApplicationCapabilityComposition,
        ApplicationCapabilityConflictRule, ApplicationCapabilityConstraintDefinition,
        ApplicationCapabilityContract, ApplicationCapabilityContractBuilder,
        ApplicationCapabilityCurrentnessDefinition, ApplicationCapabilityDecisionComposition,
        ApplicationCapabilityDelegationDefinition, ApplicationCapabilityDelegationRule,
        ApplicationCapabilityDenyRule, ApplicationCapabilityDisclosureRule,
        ApplicationCapabilityDistinctActorRule, ApplicationCapabilityElevationRule,
        ApplicationCapabilityFieldBinding, ApplicationCapabilityFieldDimension,
        ApplicationCapabilityGraphClause, ApplicationCapabilityGraphRule,
        ApplicationCapabilityMagnitudeDimension, ApplicationCapabilityPathContextAnchor,
        ApplicationCapabilityPropagationComposition, ApplicationCapabilityRelationBinding,
        ApplicationCapabilityRelationDimension, ApplicationCapabilitySeparationOfDutyRule,
        ApplicationCapabilityTargetDefinition, ApplicationCapabilityValidityDefinition,
        ApplicationCapabilityValidityTimeline, ApplicationCapabilityValueBinding,
        ApplicationCapabilityWorkflowDefinition,
    },
    application_schema::{
        ApplicationAuthorizationPathBuilder, ApplicationEncodedScalarValue,
        ApplicationSchemaDeclarationBuilder, StringApplicationValueBinding,
    },
};

pub use super::declaration::*;

pub fn install(
    schema: ApplicationSchemaDeclarationBuilder<PublicationAuthorizationSchema>,
) -> ApplicationSchemaDeclarationBuilder<PublicationAuthorizationSchema> {
    schema.capability(publication_capability_contract())
}

pub fn publication_capability_contract() -> ApplicationCapabilityContract<
    PublicationAuthorizationSchema,
    PublicationCapability,
    PublicationOperation,
    PublicationInput,
> {
    ApplicationCapabilityContractBuilder::new(
        PublicationCapability::reference(),
        PublicationOperation::reference(),
        CapabilityGrant::reference(),
    )
    .target(target())
    .constraints(constraints())
    .delegation(delegation())
    .composition(composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

pub fn target() -> ApplicationCapabilityTargetDefinition {
    ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            GrantActionField::reference(),
            encoded_string("inspect"),
        ),
        ApplicationCapabilityRelationBinding::from_reference(GrantResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            GrantPurposeField::reference(),
            encoded_string("publication-proof"),
        ),
    )
}

pub fn constraints() -> ApplicationCapabilityConstraintDefinition {
    ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                GrantStatusField::reference(),
                encoded_string("active"),
            ),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(GrantWorkflowField::reference()),
                ApplicationCapabilityFieldBinding::from_reference(
                    ResourceWorkflowField::reference(),
                ),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(GrantNotBeforeField::reference()),
                ApplicationCapabilityFieldBinding::from_reference(GrantNotAfterField::reference()),
            ),
        ),
        PublicationRequestContext::reference(),
    )
}

pub fn encoded_string(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value.to_owned())
        .expect("publication capability declaration value must encode")
}

pub fn delegation() -> ApplicationCapabilityDelegationDefinition {
    ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(GrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(GrantGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(GrantGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(GrantDelegationLimitField::reference()),
        PublicationCapabilityProvenance::reference(),
    )
}

pub fn composition() -> ApplicationCapabilityComposition {
    let allow = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(ResourceOwner::reference())
        .allow(Resource::reference());
    let deny = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(ExplicitDeny::reference())
        .deny(Resource::reference());
    let conflict = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(ConflictingActor::reference())
        .deny(Resource::reference());
    let request_actor =
        anchored_actor_clause(RequestActor::reference(), RequestActorSlot::reference());
    let prior_actor = anchored_actor_clause(PriorActor::reference(), PriorActorSlot::reference());
    ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(graph(allow)),
            ApplicationCapabilityDenyRule::when(graph(deny)),
            ApplicationCapabilityConflictRule::when(graph(conflict)),
        ),
        ApplicationCapabilityActorComposition::new(
            ApplicationCapabilitySeparationOfDutyRule::when(anchored_graph(request_actor)),
            ApplicationCapabilityDistinctActorRule::when(anchored_graph(prior_actor)), worth_query_declaration::facade::application_capability::ApplicationCapabilityRecordedActorRule::not_applicable()),
        ApplicationCapabilityPropagationComposition::new(
            ApplicationCapabilityDelegationRule::narrow_all_dimensions(
                worth_query_declaration::facade::application_capability::ApplicationCapabilityDelegationDepth::new(1)
                    .unwrap(),
            ),
            ApplicationCapabilityDisclosureRule::not_applicable(),
        ),
    )
}

pub fn anchored_actor_clause<Relation, Slot>(
    relation: worth_query_declaration::facade::application_schema::ApplicationRelationRef<
        PublicationAuthorizationSchema,
        Relation,
        Principal,
        ActionRecord,
    >,
    slot: worth_query_declaration::facade::application_capability::ApplicationCapabilityContextEntitySlotRef<
        PublicationAuthorizationSchema,
        PublicationRequestContext,
        Slot,
        ActionRecord,
    >,
) -> ApplicationCapabilityGraphClause {
    ApplicationCapabilityGraphClause::new(
        ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
            .forward(relation)
            .forward(ActionResource::reference())
            .deny(Resource::reference()),
    )
    .anchored([ApplicationCapabilityPathContextAnchor::after_forward(
        relation, slot,
    )])
}

pub fn graph(
    path: worth_query_declaration::facade::application_schema::ApplicationAuthorizationPath,
) -> ApplicationCapabilityGraphRule {
    ApplicationCapabilityGraphRule::any([ApplicationCapabilityGraphClause::new(path)])
}

pub fn anchored_graph(clause: ApplicationCapabilityGraphClause) -> ApplicationCapabilityGraphRule {
    ApplicationCapabilityGraphRule::any([clause])
}
