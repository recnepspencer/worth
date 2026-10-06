use crate::declaration::UiIntentCatalogResolvedRoute;
use crate::runtime::interaction::UiPresentedInteractionTargetView;

#[path = "standing_observation/inspection.rs"]
mod inspection;

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

impl UiIntentStandingOperabilityUnavailable {
    /// Whether a publication still in flight withheld the observation. The
    /// target's standing is unknown until that publication settles, which is
    /// not the same as knowing it cannot be operated.
    pub(crate) const fn is_withheld_by_publication(&self) -> bool {
        matches!(
            self,
            Self::Target(super::super::payload::UiIntentPayloadStop::PublicationTransitionInFlight)
        )
    }
}

/// The declaration and execution owners that decide operability: the
/// generation's catalog, definitions and execution support, and the session's
/// occupancy.
#[derive(Clone, Copy)]
pub(crate) struct UiIntentOperabilityAuthority<'owners> {
    pub(crate) catalog: &'owners crate::declaration::UiIntentCatalog,
    pub(crate) definitions: &'owners crate::capability::FrozenIntentDefinitionCapabilities,
    pub(crate) execution_bindings:
        &'owners crate::runtime::intent_execution::FrozenIntentExecutionBindings,
    pub(crate) occupancy: &'owners super::UiIntentOccupancyState,
}

/// The owners an activation observation reads, all at one generation.
#[derive(Clone, Copy)]
pub(crate) struct UiIntentOperabilityReadOwners<'owners> {
    pub(crate) authority: UiIntentOperabilityAuthority<'owners>,
    pub(crate) generation: &'owners crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    pub(crate) inputs: super::super::payload::UiIntentInputOwners<'owners>,
}

pub(crate) fn observe_activation_operability(
    target: UiPresentedInteractionTargetView,
    owners: UiIntentOperabilityReadOwners<'_>,
    confirmation: &super::super::UiIntentConfirmationState,
    host_time: Option<worth_ui_host_contract::UiHostObservationTimeBasis>,
) -> Result<UiIntentStandingOperabilityObservation, UiIntentStandingOperabilityUnavailable> {
    owners
        .inputs
        .mounted
        .current_semantic_surface_for_presentation(target.presentation())
        .map_err(UiIntentStandingOperabilityUnavailable::Presentation)?;
    let activation = observe_activation(target, owners, super::UiIntentAffinityPosture::Current)?;
    let decision = match activation.route {
        UiIntentActivationRoute::Product(decision) => UiIntentStandingDecision::Product(decision),
        UiIntentActivationRoute::Confirmation => {
            let time = host_time
                .ok_or(UiIntentStandingOperabilityUnavailable::ConfirmationTimeUnavailable)?;
            UiIntentStandingDecision::Confirmation(super::super::observe_confirmation(
                confirmation,
                target,
                time,
                super::super::UiIntentConfirmationReadContext {
                    catalog: owners.authority.catalog,
                    definitions: owners.authority.definitions,
                    generation: owners.generation,
                    mounted: owners.inputs.mounted,
                    application_facts: owners.inputs.application_facts,
                    occupancy: owners.authority.occupancy,
                    expressions: owners.inputs.expressions,
                },
            ))
        }
    };
    Ok(UiIntentStandingOperabilityObservation {
        generation: owners.generation.clone(),
        graph_node: activation.graph_node,
        target,
        route: activation.declaration,
        decision,
    })
}

/// The product decision a standing fact would record now, or `None` when the
/// fact must be left to its lifecycle: its target no longer admits, or no
/// longer resolves to the fact's own node and route. The fact's affinity is
/// kept, because it is relative to the candidate the fact was recorded from.
pub(crate) fn reobserve_standing_fact(
    fact: &super::UiIntentOperabilityStandingFact,
    owners: UiIntentOperabilityReadOwners<'_>,
) -> Option<super::UiIntentOperabilityDecision> {
    let activation = observe_activation(fact.target(), owners, fact.decision().affinity()).ok()?;
    if activation.graph_node != fact.graph_node() || *activation.declaration != *fact.route() {
        return None;
    }
    match activation.route {
        UiIntentActivationRoute::Product(decision) => Some(decision),
        // Conditions feed product operability only; a confirmation route has
        // no condition consumer to refresh.
        UiIntentActivationRoute::Confirmation => None,
    }
}

/// Target admission, activation route and product decision: the one
/// composition standing observation and condition re-observation share.
struct UiIntentActivation {
    graph_node: crate::graph::UiGraphNodeIdentity,
    declaration: Box<str>,
    route: UiIntentActivationRoute,
}

enum UiIntentActivationRoute {
    Product(super::UiIntentOperabilityDecision),
    Confirmation,
}

fn observe_activation(
    target: UiPresentedInteractionTargetView,
    owners: UiIntentOperabilityReadOwners<'_>,
    affinity: super::UiIntentAffinityPosture,
) -> Result<UiIntentActivation, UiIntentStandingOperabilityUnavailable> {
    super::super::payload::UiIntentInputBasisView::observe_target_with(
        target,
        owners.generation,
        owners.inputs,
        |view| {
            let (route, _) = owners
                .authority
                .catalog
                .lookup(
                    view.graph_node(),
                    crate::capability::UiSemanticInteractionFamily::Activate,
                )
                .ok_or(UiIntentStandingOperabilityUnavailable::MissingActivationRoute)?;
            let (declaration, route) = match route {
                UiIntentCatalogResolvedRoute::Product { declaration, .. } => {
                    let basis = super::observe_operability_basis(
                        view,
                        &declaration,
                        owners
                            .authority
                            .definitions
                            .definition_at(declaration.definition()),
                        owners
                            .authority
                            .execution_bindings
                            .support_at(declaration.definition()),
                        owners.authority.occupancy,
                    );
                    (
                        declaration.identity().as_str().into(),
                        UiIntentActivationRoute::Product(basis.decision(affinity)),
                    )
                }
                UiIntentCatalogResolvedRoute::Confirmation { declaration, .. } => (
                    declaration.identity().as_str().into(),
                    UiIntentActivationRoute::Confirmation,
                ),
            };
            Ok(UiIntentActivation {
                graph_node: view.graph_node(),
                declaration,
                route,
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
