use super::{
    UiIntentRouteResolution, UiIntentRouteResolutionStop, UiResolvedConfirmationIntentRoute,
    UiResolvedProductIntentRoute,
};

pub(crate) fn resolve_intent_route(
    catalog: &crate::declaration::UiIntentCatalog,
    definitions: &crate::capability::FrozenIntentDefinitionCapabilities,
    generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    mounted: &crate::mounting::WorthUiMountedSessionState,
    source: crate::runtime::interaction::UiIntentRouteSource,
) -> Result<UiIntentRouteResolution, UiIntentRouteResolutionStop> {
    match source.into_kind() {
        crate::runtime::interaction::UiIntentRouteSourceMaterial::MountedInteraction(
            interaction,
        ) => resolve_mounted(catalog, definitions, generation, mounted, interaction),
        crate::runtime::interaction::UiIntentRouteSourceMaterial::CommandRoute(receipt) => {
            resolve_command(catalog, definitions, generation, mounted, receipt)
        }
    }
}

fn resolve_mounted(
    catalog: &crate::declaration::UiIntentCatalog,
    definitions: &crate::capability::FrozenIntentDefinitionCapabilities,
    generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    mounted: &crate::mounting::WorthUiMountedSessionState,
    interaction: crate::runtime::interaction::UiSemanticInteraction,
) -> Result<UiIntentRouteResolution, UiIntentRouteResolutionStop> {
    if interaction.generation() != generation {
        return Err(UiIntentRouteResolutionStop::ApplicationGenerationChanged);
    }
    let family = interaction.family();
    let affinity =
        crate::runtime::interaction::targeting::admit_current_target(mounted, interaction.target())
            .map_err(UiIntentRouteResolutionStop::Targeting)?;
    let graph_node = affinity.graph_node();
    let (route, cost) =
        catalog
            .lookup(graph_node, family)
            .ok_or(UiIntentRouteResolutionStop::Unrouted {
                graph_node,
                interaction: family,
            })?;
    Ok(match route {
        crate::declaration::UiIntentCatalogResolvedRoute::Product { route, declaration } => {
            UiIntentRouteResolution::Product(UiResolvedProductIntentRoute::new(
                super::UiResolvedProductIntentRouteInput {
                    graph_node: route.graph_node(),
                    interaction: route.interaction(),
                    portal_declaration: route.portal_declaration(),
                    definition_id: definitions.definition_at(declaration.definition()).id(),
                    declaration,
                    source: super::UiIntentProductInputSource::mounted(interaction),
                    cost,
                },
            ))
        }
        crate::declaration::UiIntentCatalogResolvedRoute::Confirmation { route, declaration } => {
            UiIntentRouteResolution::Confirmation(UiResolvedConfirmationIntentRoute::new(
                super::UiResolvedConfirmationIntentRouteInput {
                    graph_node: route.graph_node(),
                    definition_id: definitions.definition_at(declaration.definition()).id(),
                    declaration,
                    source: interaction,
                    cost,
                },
            ))
        }
    })
}

fn resolve_command(
    catalog: &crate::declaration::UiIntentCatalog,
    definitions: &crate::capability::FrozenIntentDefinitionCapabilities,
    generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    mounted: &crate::mounting::WorthUiMountedSessionState,
    receipt: crate::runtime::UiCommandRouteReceipt,
) -> Result<UiIntentRouteResolution, UiIntentRouteResolutionStop> {
    if receipt.application() != generation {
        return Err(UiIntentRouteResolutionStop::ApplicationGenerationChanged);
    }
    let presentation = receipt
        .presentation()
        .ok_or(UiIntentRouteResolutionStop::CommandInvocationBasisMissing)?;
    if receipt.sequence().is_none() || receipt.time_basis().is_none() {
        return Err(UiIntentRouteResolutionStop::CommandInvocationBasisMissing);
    }
    let intent = receipt.destination().intent();
    let (resolved, cost) = catalog
        .lookup_command(intent)
        .ok_or(UiIntentRouteResolutionStop::CommandDestinationUnrouted { intent })?;
    let declaration = match resolved {
        crate::declaration::UiIntentCatalogCommandRoute::Resolved { declaration } => declaration,
        crate::declaration::UiIntentCatalogCommandRoute::Ambiguous { candidates } => {
            return Err(UiIntentRouteResolutionStop::CommandDestinationAmbiguous {
                intent,
                candidates,
            })
        }
    };
    let target = crate::runtime::interaction::targeting::resolve_presented_command_target(
        mounted,
        presentation,
        &receipt,
    )
    .map_err(UiIntentRouteResolutionStop::Targeting)?;
    if receipt.evidence_reference().is_none() {
        return Err(UiIntentRouteResolutionStop::CommandEvidenceMissing);
    }
    let graph_node = crate::runtime::interaction::targeting::admit_current_target(mounted, target)
        .map_err(UiIntentRouteResolutionStop::Targeting)?
        .graph_node();
    Ok(UiIntentRouteResolution::Product(
        UiResolvedProductIntentRoute::new(super::UiResolvedProductIntentRouteInput {
            graph_node,
            interaction: declaration.interaction(),
            portal_declaration: None,
            definition_id: definitions.definition_at(declaration.definition()).id(),
            declaration,
            source: super::UiIntentProductInputSource::command(receipt, target),
            cost,
        }),
    ))
}
