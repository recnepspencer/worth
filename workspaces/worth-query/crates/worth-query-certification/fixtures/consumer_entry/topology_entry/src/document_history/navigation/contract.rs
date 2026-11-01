//! Current navigation admits an actual grant on the same full model.
use super::*;
pub(super) fn contract<S: TopologySchemaBinding>(
) -> ApplicationCapabilityContract<S, NavigatePlanar<S>, NavigatePlanarOperation, PlanarOutputRead>
{
    let path = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference::<S>())
        .forward(PlanarNavigationGrantee::reference::<S>())
        .forward(PlanarNavigationResource::reference::<S>())
        .allow(Body::reference::<S>());
    ApplicationCapabilityContractBuilder::new(
        NavigatePlanar::<S>::reference(),
        NavigatePlanarOperation::reference::<S>(),
        PlanarNavigationGrant::reference::<S>(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            PlanarNavigationAction::reference::<S>(),
            encoded("navigate".to_owned()),
        ),
        ApplicationCapabilityRelationBinding::from_reference(
            PlanarNavigationResource::reference::<S>(),
        ),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            PlanarNavigationPurpose::reference::<S>(),
            encoded("model-history".to_owned()),
        ),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                PlanarNavigationStatus::reference::<S>(),
                encoded("active".to_owned()),
            ),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(
                    PlanarNavigationScopeKey::reference::<S>(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(
                    BodyKey::reference::<S>(),
                ),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(
                    PlanarNavigationNotBefore::reference::<S>(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(
                    PlanarNavigationNotAfter::reference::<S>(),
                ),
            ),
        ),
        PlanarNavigationContext::<S>::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(
            PlanarNavigationParent::reference::<S>(),
        ),
        ApplicationCapabilityRelationBinding::from_reference(
            PlanarNavigationGrantor::reference::<S>(),
        ),
        ApplicationCapabilityRelationBinding::from_reference(
            PlanarNavigationGrantee::reference::<S>(),
        ),
        ApplicationCapabilityFieldBinding::from_reference(PlanarNavigationDepth::reference::<S>()),
        PlanarNavigationProvenance::<S>::reference(),
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
            ApplicationCapabilityDistinctActorRule::not_applicable(), worth_query_decl::facade::application_capability::ApplicationCapabilityRecordedActorRule::not_applicable()),
        ApplicationCapabilityPropagationComposition::new(
            ApplicationCapabilityDelegationRule::forbidden(),
            ApplicationCapabilityDisclosureRule::not_applicable(),
        ),
    ))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

/// Bind an explicit current grant through the ordinary bootstrap owner.
pub fn bind_navigation_grant<S: TopologySchemaBinding>(
    graph: &mut worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphBootstrap<S>,
    principal: &str,
    model: &str,
) -> Result<(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    use worth_query_host::facade::primary_graph::{
        WorthQueryApplicationEntityKey as Key, WorthQueryApplicationEntitySeed as Entity,
        WorthQueryApplicationRelationSeed as Relation,
    };
    let grant = format!("history-navigation:{principal}:{model}");
    graph.bind_entity(
        Entity::new(
            PlanarNavigationGrant::reference::<S>(),
            Key::new(grant.clone()).unwrap(),
        )
        .field(
            PlanarNavigationGrantIdentity::reference::<S>(),
            grant.clone(),
        )
        .field(PlanarNavigationScopeKey::reference::<S>(), model.to_owned())
        .field(
            PlanarNavigationAction::reference::<S>(),
            "navigate".to_owned(),
        )
        .field(
            PlanarNavigationPurpose::reference::<S>(),
            "model-history".to_owned(),
        )
        .field(
            PlanarNavigationStatus::reference::<S>(),
            "active".to_owned(),
        )
        .field(PlanarNavigationNotBefore::reference::<S>(), 0)
        .field(PlanarNavigationNotAfter::reference::<S>(), u64::MAX)
        .field(PlanarNavigationDepth::reference::<S>(), 0),
    )?;
    graph.bind_relation(Relation::new(
        PlanarNavigationResource::reference::<S>(),
        format!("{grant}:resource"),
        Key::new(grant.clone()).unwrap(),
        Key::new(model.to_owned()).unwrap(),
    ))?;
    graph.bind_relation(Relation::new(
        PlanarNavigationGrantor::reference::<S>(),
        format!("{grant}:grantor"),
        Key::new(principal.to_owned()).unwrap(),
        Key::new(grant.clone()).unwrap(),
    ))?;
    graph.bind_relation(Relation::new(
        PlanarNavigationGrantee::reference::<S>(),
        format!("{grant}:grantee"),
        Key::new(principal.to_owned()).unwrap(),
        Key::new(grant).unwrap(),
    ))?;
    Ok(())
}
