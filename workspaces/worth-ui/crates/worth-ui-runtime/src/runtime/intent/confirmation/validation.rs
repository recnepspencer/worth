use std::sync::Arc;

use super::{
    UiIntentConfirmationChallenge, UiIntentConfirmationStopReason,
    UiIntentConfirmationTimeBasisKind,
};

pub(crate) struct UiIntentConfirmationReadContext<'state> {
    pub(crate) catalog: &'state crate::declaration::UiIntentCatalog,
    pub(crate) definitions: &'state crate::capability::FrozenIntentDefinitionCapabilities,
    pub(crate) generation: &'state crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    pub(crate) mounted: &'state crate::mounting::WorthUiMountedSessionState,
    pub(crate) application_facts: &'state super::super::payload::UiIntentApplicationFactState,
    pub(crate) occupancy: &'state super::super::operability::UiIntentOccupancyState,
}

pub(super) struct UiIntentConfirmationSourceBasis<'source> {
    pub(super) generation: &'source crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    pub(super) target: crate::runtime::interaction::UiPresentedInteractionTargetView,
    pub(super) time_basis: worth_ui_host_contract::UiHostObservationTimeBasis,
}

pub(super) fn validate_challenge(
    challenge: &UiIntentConfirmationChallenge,
    route_definition: crate::capability::UiIntentId,
    route_declaration: &Arc<crate::declaration::UiCanonicalIntentDeclaration>,
    source: UiIntentConfirmationSourceBasis<'_>,
    context: &UiIntentConfirmationReadContext<'_>,
) -> Option<UiIntentConfirmationStopReason> {
    let candidate = &challenge.candidate;
    let candidate_generation = candidate.input_basis().generation();
    if candidate_generation.session_identity() != context.generation.session_identity() {
        return Some(UiIntentConfirmationStopReason::ApplicationWorldChanged);
    }
    if candidate_generation.prepared_generation() != context.generation.prepared_generation() {
        return Some(UiIntentConfirmationStopReason::ApplicationGenerationChanged);
    }
    if source.generation.session_identity() != context.generation.session_identity() {
        return Some(UiIntentConfirmationStopReason::ApplicationWorldChanged);
    }
    if source.generation.prepared_generation() != context.generation.prepared_generation() {
        return Some(UiIntentConfirmationStopReason::ApplicationGenerationChanged);
    }
    if route_definition != candidate.definition_id()
        || !Arc::ptr_eq(route_declaration, candidate.declaration_reference())
    {
        return Some(UiIntentConfirmationStopReason::ConfirmationRouteChanged);
    }
    let observed = match source.time_basis {
        worth_ui_host_contract::UiHostObservationTimeBasis::HostMonotonicMillis(millis) => millis,
        worth_ui_host_contract::UiHostObservationTimeBasis::HostWallClockMicros(_) => {
            return Some(UiIntentConfirmationStopReason::MonotonicTimeRequired {
                observed: UiIntentConfirmationTimeBasisKind::HostWallClock,
            })
        }
        worth_ui_host_contract::UiHostObservationTimeBasis::PresentationRelativeTick(_) => {
            return Some(UiIntentConfirmationStopReason::MonotonicTimeRequired {
                observed: UiIntentConfirmationTimeBasisKind::PresentationRelative,
            })
        }
    };
    if observed < challenge.issued_at_millis {
        return Some(UiIntentConfirmationStopReason::MonotonicTimeRegressed {
            issued_at_millis: challenge.issued_at_millis,
            observed_millis: observed,
        });
    }
    if observed > challenge.expires_at_millis {
        return Some(UiIntentConfirmationStopReason::Expired {
            expires_at_millis: challenge.expires_at_millis,
            observed_millis: observed,
        });
    }
    let current_frame = context.mounted.view().current_frame();
    if source.target.frame_relation()
        != crate::runtime::interaction::UiPresentedTargetFrameRelation::Current
        || current_frame != Some(source.target.presentation().frame())
    {
        return Some(UiIntentConfirmationStopReason::ConfirmationPresentationStale);
    }
    if let Err(denial) =
        crate::runtime::interaction::targeting::admit_current_target(context.mounted, source.target)
    {
        return Some(UiIntentConfirmationStopReason::ConfirmationTargetChanged(
            denial,
        ));
    }
    if source.target.presentation().frame() == candidate.input_basis().publication_frame() {
        return Some(UiIntentConfirmationStopReason::ConfirmationNotPresented);
    }
    let affinity = match crate::runtime::interaction::targeting::admit_current_target_incarnation(
        context.mounted,
        candidate.input_basis().target(),
    ) {
        Ok(affinity) => affinity,
        Err(denial) => return Some(UiIntentConfirmationStopReason::TargetChanged(denial)),
    };
    if affinity.graph_node() != candidate.graph_node()
        || !product_route_is_current(candidate, context.catalog, context.definitions)
    {
        return Some(UiIntentConfirmationStopReason::ProductRouteChanged);
    }
    if !candidate.payload_inputs_are_current(
        context.mounted,
        context.application_facts,
        context.generation,
    ) {
        return Some(UiIntentConfirmationStopReason::PayloadInputChanged);
    }
    match candidate.operability_dependencies_are_current(
        context.mounted,
        context.application_facts,
        context.generation,
    ) {
        Ok(()) => {}
        Err(super::super::operability::UiIntentOperabilityDependencyDrift::DeclaredDependency) => {
            return Some(UiIntentConfirmationStopReason::OperabilityDependencyChanged)
        }
        Err(super::super::operability::UiIntentOperabilityDependencyDrift::Policy) => {
            return Some(UiIntentConfirmationStopReason::PolicyChanged)
        }
        Err(super::super::operability::UiIntentOperabilityDependencyDrift::Confirmation) => {
            return Some(UiIntentConfirmationStopReason::ConfirmationPolicyChanged)
        }
    }
    if challenge.decision.confirmation().required_policy_identity()
        != Some(challenge.policy_identity.as_ref())
    {
        return Some(UiIntentConfirmationStopReason::ConfirmationPolicyChanged);
    }
    if !context
        .occupancy
        .is_current_observation(candidate.operability_basis().occupancy())
    {
        return Some(UiIntentConfirmationStopReason::OccupancyChanged);
    }
    None
}

fn product_route_is_current(
    candidate: &super::super::payload::UiPreparedIntentPayload,
    catalog: &crate::declaration::UiIntentCatalog,
    definitions: &crate::capability::FrozenIntentDefinitionCapabilities,
) -> bool {
    let Some((crate::declaration::UiIntentCatalogResolvedRoute::Product { declaration, .. }, _)) =
        catalog.lookup(candidate.graph_node(), candidate.interaction_family())
    else {
        return false;
    };
    Arc::ptr_eq(&declaration, candidate.declaration_reference())
        && definitions.definition_at(declaration.definition()).id() == candidate.definition_id()
}
