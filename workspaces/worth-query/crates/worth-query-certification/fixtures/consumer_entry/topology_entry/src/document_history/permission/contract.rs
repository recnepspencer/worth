//! Current navigation admits an actual grant on the same full model.
use super::*;
pub(in crate::document_history) fn contract<S: TopologySchemaBinding, C: HistoryPermission<S>, O>(
) -> ApplicationCapabilityContract<S, C, O, PlanarOutputRead>
where
    O: ApplicationOperationMarkerIdentity<S, InputBinding = PlanarOutputReadInputBinding> + 'static,
{
    let path = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference::<S>())
        .forward(PlanarHistoryGrantee::reference::<S>())
        .forward(PlanarHistoryResource::reference::<S>())
        .allow(Body::reference::<S>());
    ApplicationCapabilityContractBuilder::new(
        ApplicationCapabilityRef::<S, C>::from_declaration(),
        ApplicationOperationRef::<S, O, PlanarOutputRead>::from_declaration(),
        PlanarHistoryGrant::reference::<S>(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            PlanarHistoryAction::at("PlanarHistoryAction"),
            encoded(C::ACTION.to_owned()),
        ).unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(
            PlanarHistoryResource::reference::<S>(),
        ),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            PlanarHistoryPurpose::at("PlanarHistoryPurpose"),
            encoded("model-history".to_owned()),
        ).unwrap(),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                PlanarHistoryStatus::at("PlanarHistoryStatus"),
                encoded("active".to_owned()),
            ).unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(
                    PlanarHistoryScopeKey::at("PlanarHistoryScopeKey"),
                ).unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(
                    BodyKey::at("BodyKey"),
                ).unwrap(),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(
                    PlanarHistoryNotBefore::at("PlanarHistoryNotBefore"),
                ).unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(
                    PlanarHistoryNotAfter::at("PlanarHistoryNotAfter"),
                ).unwrap(),
            ),
        ),
        PlanarHistoryContext::<S>::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(
            PlanarHistoryParent::reference::<S>(),
        ),
        ApplicationCapabilityRelationBinding::from_reference(
            PlanarHistoryGrantor::reference::<S>(),
        ),
        ApplicationCapabilityRelationBinding::from_reference(
            PlanarHistoryGrantee::reference::<S>(),
        ),
        ApplicationCapabilityFieldBinding::from_reference(PlanarHistoryDepth::at("PlanarHistoryDepth")).unwrap(),
        PlanarHistoryProvenance::<S>::reference(),
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
fn bind_permission<S: TopologySchemaBinding, C: HistoryPermission<S>>(
    graph: &mut worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphBootstrap<S>,
    principal: &str,
    model: &str,
) -> Result<(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    use worth_query_host::facade::primary_graph::{
        WorthQueryApplicationEntityKey as Key, WorthQueryApplicationEntitySeed as Entity,
        WorthQueryApplicationRelationSeed as Relation,
    };
    let grant = format!("history-{}:{principal}:{model}", C::GRANT_KIND);
    graph.bind_entity(
        Entity::new(
            PlanarHistoryGrant::reference::<S>(),
            Key::new(grant.clone()).unwrap(),
        )
        .field(
            PlanarHistoryGrantIdentity::at("PlanarHistoryGrantIdentity"),
            grant.clone(),
        )
        .field(
            PlanarHistoryScopeKey::at("PlanarHistoryScopeKey"),
            model.to_owned(),
        )
        .field(
            PlanarHistoryAction::at("PlanarHistoryAction"),
            C::ACTION.to_owned(),
        )
        .field(
            PlanarHistoryPurpose::at("PlanarHistoryPurpose"),
            "model-history".to_owned(),
        )
        .field(
            PlanarHistoryStatus::at("PlanarHistoryStatus"),
            "active".to_owned(),
        )
        .field(PlanarHistoryNotBefore::at("PlanarHistoryNotBefore"), 0)
        .field(PlanarHistoryNotAfter::at("PlanarHistoryNotAfter"), u64::MAX)
        .field(PlanarHistoryDepth::at("PlanarHistoryDepth"), 0),
    )?;
    graph.bind_relation(Relation::new(
        PlanarHistoryResource::reference::<S>(),
        format!("{grant}:resource"),
        Key::new(grant.clone()).unwrap(),
        Key::new(model.to_owned()).unwrap(),
    ))?;
    graph.bind_relation(Relation::new(
        PlanarHistoryGrantor::reference::<S>(),
        format!("{grant}:grantor"),
        Key::new(principal.to_owned()).unwrap(),
        Key::new(grant.clone()).unwrap(),
    ))?;
    graph.bind_relation(Relation::new(
        PlanarHistoryGrantee::reference::<S>(),
        format!("{grant}:grantee"),
        Key::new(principal.to_owned()).unwrap(),
        Key::new(grant).unwrap(),
    ))?;
    Ok(())
}

pub fn bind_navigation_grant<S: TopologySchemaBinding>(
    graph: &mut worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphBootstrap<S>,
    principal: &str,
    model: &str,
) -> Result<(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    bind_permission::<S, super::super::navigation::NavigatePlanar<S>>(graph, principal, model)
}

pub fn bind_retention_grants<S: TopologySchemaBinding>(
    graph: &mut worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphBootstrap<S>,
    principal: &str,
    model: &str,
) -> Result<(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    bind_permission::<S, super::super::retention_policy::ChangePlanarHistoryWindow<S>>(
        graph, principal, model,
    )?;
    bind_permission::<S, super::super::retention_policy::ChangePlanarHistoryBookmark<S>>(
        graph, principal, model,
    )
}
