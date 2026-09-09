use crate::declaration::UiIntentCatalogResolvedRoute;
use crate::runtime::interaction::UiPresentedInteractionTargetView;

/// An observation of declared operability, never permission to execute an intent.
pub(crate) struct UiIntentStandingOperabilityObservation {
    generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    graph_node: crate::graph::UiGraphNodeIdentity,
    target: UiPresentedInteractionTargetView,
    route: Box<str>,
    decision: UiIntentStandingDecision,
}

enum UiIntentStandingDecision {
    Product(super::UiIntentOperabilityDecision),
    Confirmation(super::super::UiIntentConfirmationObservation),
}

#[derive(Debug)]
pub(crate) enum UiIntentStandingOperabilityUnavailable {
    Target(super::super::payload::UiIntentPayloadStop),
    Presentation(crate::mounting::UiPresentedFrameBasisDenial),
    MissingActivationRoute,
    ConfirmationTimeUnavailable,
}

#[allow(
    clippy::too_many_arguments,
    reason = "read-only observation explicitly names the independent target and Intent owners"
)]
pub(crate) fn observe_activation_operability(
    target: UiPresentedInteractionTargetView,
    catalog: &crate::declaration::UiIntentCatalog,
    definitions: &crate::capability::FrozenIntentDefinitionCapabilities,
    execution_bindings: &crate::runtime::intent_execution::FrozenIntentExecutionBindings,
    generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    mounted: &crate::mounting::WorthUiMountedSessionState,
    application_facts: &super::super::payload::UiIntentApplicationFactState,
    occupancy: &super::UiIntentOccupancyState,
    confirmation: &super::super::UiIntentConfirmationState,
    host_time: Option<worth_ui_host_contract::UiHostObservationTimeBasis>,
) -> Result<UiIntentStandingOperabilityObservation, UiIntentStandingOperabilityUnavailable> {
    mounted
        .current_semantic_surface_for_presentation(target.presentation())
        .map_err(UiIntentStandingOperabilityUnavailable::Presentation)?;
    super::super::payload::UiIntentInputBasisView::observe_target_with(
        target,
        generation,
        mounted,
        application_facts,
        |view| {
            let graph_node = mounted
                .current_mounted_identity_basis(target.mounted_instance())
                .expect("current target admission preserves mounted identity")
                .graph_node_identity();
            let (route, _) = catalog
                .lookup(
                    graph_node,
                    crate::capability::UiSemanticInteractionFamily::Activate,
                )
                .ok_or(UiIntentStandingOperabilityUnavailable::MissingActivationRoute)?;
            let declaration = match route {
                UiIntentCatalogResolvedRoute::Product { declaration, .. } => declaration,
                UiIntentCatalogResolvedRoute::Confirmation { declaration, .. } => {
                    let time = host_time.ok_or(
                        UiIntentStandingOperabilityUnavailable::ConfirmationTimeUnavailable,
                    )?;
                    let observation = super::super::observe_confirmation(
                        confirmation,
                        target,
                        time,
                        super::super::UiIntentConfirmationReadContext {
                            catalog,
                            definitions,
                            generation,
                            mounted,
                            application_facts,
                            occupancy,
                        },
                    );
                    return Ok(UiIntentStandingOperabilityObservation {
                        generation: generation.clone(),
                        graph_node,
                        target,
                        route: declaration.identity().as_str().into(),
                        decision: UiIntentStandingDecision::Confirmation(observation),
                    });
                }
            };
            let basis = super::observe_operability_basis(
                view,
                &declaration,
                definitions.definition_at(declaration.definition()),
                execution_bindings.support_at(declaration.definition()),
                occupancy,
            );
            Ok(UiIntentStandingOperabilityObservation {
                generation: generation.clone(),
                graph_node,
                target,
                route: declaration.identity().as_str().into(),
                decision: UiIntentStandingDecision::Product(
                    basis.decision(super::UiIntentAffinityPosture::Current),
                ),
            })
        },
    )
    .map_err(UiIntentStandingOperabilityUnavailable::Target)?
}

impl UiIntentStandingOperabilityObservation {
    pub(crate) fn generation(&self) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.generation
    }

    pub(crate) const fn graph_node(&self) -> crate::graph::UiGraphNodeIdentity {
        self.graph_node
    }

    pub(crate) const fn target(&self) -> UiPresentedInteractionTargetView {
        self.target
    }

    pub(crate) fn route(&self) -> &str {
        &self.route
    }

    pub(crate) fn product_decision(&self) -> Option<&super::UiIntentOperabilityDecision> {
        match &self.decision {
            UiIntentStandingDecision::Product(decision) => Some(decision),
            UiIntentStandingDecision::Confirmation(_) => None,
        }
    }

    pub(crate) fn is_operable(&self) -> bool {
        match &self.decision {
            UiIntentStandingDecision::Product(decision) => decision.is_operable(),
            UiIntentStandingDecision::Confirmation(observation) => observation.is_eligible(),
        }
    }

    pub(crate) fn confirmation_deadline(&self) -> Option<u64> {
        match &self.decision {
            UiIntentStandingDecision::Product(_) => None,
            UiIntentStandingDecision::Confirmation(observation) => observation.expiry_wake_millis(),
        }
    }
}
