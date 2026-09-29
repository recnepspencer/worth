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
    pub(crate) expressions: &'state crate::runtime::expression::UiExpressionRuntimeState,
}

impl<'state> UiIntentConfirmationReadContext<'state> {
    /// The owners a prepared payload read its inputs from, as they are now.
    fn payload_input_owners(&self) -> super::super::payload::UiIntentInputOwners<'state> {
        super::super::payload::UiIntentInputOwners {
            mounted: self.mounted,
            application_facts: self.application_facts,
            expressions: self.expressions,
        }
    }
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
    if !candidate.payload_inputs_are_current(context.payload_input_owners(), context.generation) {
        return Some(UiIntentConfirmationStopReason::PayloadInputChanged);
    }
    if let Err(drift) = candidate.operability_dependencies_are_current(
        &super::super::operability::UiIntentOperabilityDependencyReads {
            mounted: context.mounted,
            application_facts: context.application_facts,
            expressions: context.expressions,
            generation: context.generation,
        },
    ) {
        return Some(dependency_stop(drift));
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

/// The confirmation stop for an operability dependency that drifted. A
/// condition result reports under the stop of the axis it feeds.
const fn dependency_stop(
    drift: super::super::operability::UiIntentOperabilityDependencyDrift,
) -> UiIntentConfirmationStopReason {
    use super::super::operability::UiIntentOperabilityDependencyDrift as Drift;
    use crate::declaration::UiIntentOperabilityDependencyAxis as Axis;
    match drift {
        Drift::DeclaredDependency
        | Drift::ExpressionResult {
            axis: Axis::Mutability | Axis::Readiness,
        } => UiIntentConfirmationStopReason::OperabilityDependencyChanged,
        Drift::Policy | Drift::ExpressionResult { axis: Axis::Policy } => {
            UiIntentConfirmationStopReason::PolicyChanged
        }
        Drift::Confirmation => UiIntentConfirmationStopReason::ConfirmationPolicyChanged,
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::operability::UiIntentOperabilityDependencyDrift as Drift;
    use super::{dependency_stop, UiIntentConfirmationStopReason as Stop};
    use crate::declaration::UiIntentOperabilityDependencyAxis as Axis;

    #[test]
    fn a_drifted_condition_stops_confirmation_under_the_axis_it_feeds() {
        for (drift, expected) in [
            (
                Drift::DeclaredDependency,
                Stop::OperabilityDependencyChanged,
            ),
            (
                Drift::ExpressionResult {
                    axis: Axis::Mutability,
                },
                Stop::OperabilityDependencyChanged,
            ),
            (
                Drift::ExpressionResult {
                    axis: Axis::Readiness,
                },
                Stop::OperabilityDependencyChanged,
            ),
            (Drift::Policy, Stop::PolicyChanged),
            (
                Drift::ExpressionResult { axis: Axis::Policy },
                Stop::PolicyChanged,
            ),
            (Drift::Confirmation, Stop::ConfirmationPolicyChanged),
        ] {
            assert_eq!(dependency_stop(drift), expected, "{drift:?}");
        }
    }
}
