use std::sync::Arc;

use crate::capability::{UiIntentPayload, UiIntentProductOutcome};

use super::candidate::{
    UiCurrentIntentAdmissionCandidate, UiIntentAdmissionCandidateOrigin,
    UiPreparedIntentAdmissionCandidate,
};
use super::{UiIntentAdmissionCost, UiIntentAdmissionStopReason};

pub(crate) struct UiIntentAdmissionCurrentnessContext<'state> {
    pub(crate) catalog: &'state crate::declaration::UiIntentCatalog,
    pub(crate) definitions: &'state crate::capability::FrozenIntentDefinitionCapabilities,
    pub(crate) generation: &'state crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    pub(crate) mounted: &'state crate::mounting::WorthUiMountedSessionState,
    pub(crate) application_facts: &'state super::super::payload::UiIntentApplicationFactState,
    pub(crate) expressions: &'state crate::runtime::expression::UiExpressionRuntimeState,
    pub(crate) command_contexts: Box<[crate::runtime::command_routing::UiCommandRoutingContext]>,
}

pub(crate) struct UiIntentAdmissionPreparationFailure {
    reason: UiIntentAdmissionStopReason,
    cost: UiIntentAdmissionCost,
}

pub(crate) struct UiIntentExecutionCurrentnessAdmission {
    checks: usize,
    target: crate::runtime::interaction::targeting::UiIntentExecutionTargetAffinity,
}

enum UiIntentCandidateCurrentnessViolation {
    ApplicationWorldChanged,
    ApplicationGenerationChanged,
    PresentationInFlight,
    TargetChanged(crate::runtime::interaction::UiInteractionTargetingDenial),
    ProductRouteChanged,
    CommandContextChanged,
    PayloadInputChanged,
    OperabilityDependencyChanged,
    PolicyChanged,
    ConfirmationPolicyChanged,
}

pub(crate) fn prepare_typed_candidate<I, D>(
    definition: crate::capability::UiIntentDefinition<I, D>,
    candidate: UiPreparedIntentAdmissionCandidate,
    context: UiIntentAdmissionCurrentnessContext<'_>,
) -> Result<UiCurrentIntentAdmissionCandidate, UiIntentAdmissionPreparationFailure>
where
    I: crate::capability::UiIntent,
    D: crate::capability::UiIntentDefinitionDestination,
{
    let operability_dependencies = candidate.decision().cost().selected_dependencies_visited();
    validate_typed_definition(definition, candidate.payload(), context.definitions).map_err(
        |reason| {
            UiIntentAdmissionPreparationFailure::new(
                reason,
                UiIntentAdmissionCost::prepared(
                    candidate.route_resolution_cost(),
                    candidate.payload_projection_cost(),
                    operability_dependencies,
                    1,
                ),
            )
        },
    )?;
    let currentness =
        validate_currentness(&candidate, &context).map_err(|(violation, currentness_checks)| {
            UiIntentAdmissionPreparationFailure::new(
                violation.into_admission_stop(),
                UiIntentAdmissionCost::prepared(
                    candidate.route_resolution_cost(),
                    candidate.payload_projection_cost(),
                    operability_dependencies,
                    currentness_checks,
                ),
            )
        })?;
    Ok(candidate.seal_current(currentness.checks()))
}

pub(crate) fn revalidate_typed_candidate_for_execution<I: crate::capability::UiIntent>(
    candidate: &UiCurrentIntentAdmissionCandidate,
    context: UiIntentAdmissionCurrentnessContext<'_>,
) -> Result<
    UiIntentExecutionCurrentnessAdmission,
    crate::runtime::intent_execution::UiIntentExecutionCurrentnessStop,
> {
    validate_typed_intent::<I>(candidate.prepared().payload(), context.definitions)?;
    validate_currentness(candidate.prepared(), &context)
        .map_err(|(violation, _)| violation.into_execution_stop())
}

pub(crate) fn validate_typed_inoperable<I, D>(
    definition: crate::capability::UiIntentDefinition<I, D>,
    candidate: &super::super::operability::UiInoperableIntentCandidate,
    definitions: &crate::capability::FrozenIntentDefinitionCapabilities,
) -> Result<UiIntentAdmissionCost, UiIntentAdmissionPreparationFailure>
where
    I: crate::capability::UiIntent,
    D: crate::capability::UiIntentDefinitionDestination,
{
    let cost = UiIntentAdmissionCost::prepared(
        candidate.candidate().input_basis().route_resolution_cost(),
        candidate.candidate().input_basis().cost(),
        candidate.decision().cost().selected_dependencies_visited(),
        1,
    );
    validate_typed_definition(definition, candidate.candidate(), definitions)
        .map(|()| cost)
        .map_err(|reason| UiIntentAdmissionPreparationFailure::new(reason, cost))
}

fn validate_typed_definition<I, D>(
    definition: crate::capability::UiIntentDefinition<I, D>,
    candidate: &super::super::payload::UiPreparedIntentPayload,
    definitions: &crate::capability::FrozenIntentDefinitionCapabilities,
) -> Result<(), UiIntentAdmissionStopReason>
where
    I: crate::capability::UiIntent,
    D: crate::capability::UiIntentDefinitionDestination,
{
    let registered = definitions.definition_at(candidate.declaration_reference().definition());
    if candidate.definition_id() != definition.id() || registered != &definition.descriptor() {
        return Err(UiIntentAdmissionStopReason::DefinitionContractMismatch {
            candidate: candidate.definition_id(),
            requested: definition.id(),
        });
    }
    if definition.execution_destination()
        == crate::capability::UiIntentExecutionDestination::RuntimeService(
            crate::capability::UiIntentRuntimeServiceDestination::InvokeCommand,
        )
    {
        let receipt = candidate
            .command_route_receipt()
            .ok_or(UiIntentAdmissionStopReason::CommandRouteRequired)?;
        let routed = receipt.destination().intent();
        if routed != definition.id() {
            return Err(UiIntentAdmissionStopReason::CommandDestinationMismatch {
                routed,
                requested: definition.id(),
            });
        }
    }
    Ok(())
}

fn validate_typed_intent<I: crate::capability::UiIntent>(
    candidate: &super::super::payload::UiPreparedIntentPayload,
    definitions: &crate::capability::FrozenIntentDefinitionCapabilities,
) -> Result<(), crate::runtime::intent_execution::UiIntentExecutionCurrentnessStop> {
    let registered = definitions.definition_at(candidate.declaration_reference().definition());
    if candidate.definition_id() != I::ID
        || registered.id() != I::ID
        || registered.payload_schema() != I::Payload::SCHEMA
        || registered.product_outcome_schema() != I::ProductOutcome::SCHEMA
        || registered.accepted_interactions() != I::ACCEPTED_INTERACTIONS.as_slice()
    {
        return Err(
            crate::runtime::intent_execution::UiIntentExecutionCurrentnessStop::DefinitionContractMismatch {
                candidate: candidate.definition_id(),
                requested: I::ID,
            },
        );
    }
    Ok(())
}

fn validate_currentness(
    candidate: &UiPreparedIntentAdmissionCandidate,
    context: &UiIntentAdmissionCurrentnessContext<'_>,
) -> Result<UiIntentExecutionCurrentnessAdmission, (UiIntentCandidateCurrentnessViolation, usize)> {
    let payload = candidate.payload();
    let basis = payload.input_basis();
    if basis.generation().session_identity() != context.generation.session_identity() {
        return Err((
            UiIntentCandidateCurrentnessViolation::ApplicationWorldChanged,
            2,
        ));
    }
    if basis.generation().prepared_generation() != context.generation.prepared_generation() {
        return Err((
            UiIntentCandidateCurrentnessViolation::ApplicationGenerationChanged,
            3,
        ));
    }
    if context.mounted.has_active_presentation_attempt() {
        return Err((
            UiIntentCandidateCurrentnessViolation::PresentationInFlight,
            4,
        ));
    }
    let target = validate_target(candidate.origin(), payload, context.mounted)
        .map_err(|reason| (reason, 5))?;
    if !product_route_is_current(payload, context.catalog, context.definitions) {
        return Err((
            UiIntentCandidateCurrentnessViolation::ProductRouteChanged,
            6,
        ));
    }
    if let Some(receipt) = payload.command_route_receipt() {
        let current = context
            .command_contexts
            .iter()
            .find(|current| current.surface() == receipt.surface());
        if !current.is_some_and(|current| receipt.consumed_context_is_current(current)) {
            return Err((
                UiIntentCandidateCurrentnessViolation::CommandContextChanged,
                7,
            ));
        }
    }
    if !payload.payload_inputs_are_current(
        context.mounted,
        context.application_facts,
        context.generation,
    ) {
        return Err((
            UiIntentCandidateCurrentnessViolation::PayloadInputChanged,
            8,
        ));
    }
    payload
        .operability_dependencies_are_current(
            &super::super::operability::UiIntentOperabilityDependencyReads {
                mounted: context.mounted,
                application_facts: context.application_facts,
                expressions: context.expressions,
                generation: context.generation,
            },
        )
        .map_err(|drift| {
            (
                UiIntentCandidateCurrentnessViolation::from_dependency_drift(drift),
                9,
            )
        })?;
    Ok(UiIntentExecutionCurrentnessAdmission { checks: 9, target })
}

impl UiIntentExecutionCurrentnessAdmission {
    const fn checks(&self) -> usize {
        self.checks
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        usize,
        crate::runtime::interaction::targeting::UiIntentExecutionTargetAffinity,
    ) {
        (self.checks, self.target)
    }
}

impl UiIntentAdmissionPreparationFailure {
    const fn new(reason: UiIntentAdmissionStopReason, cost: UiIntentAdmissionCost) -> Self {
        Self { reason, cost }
    }

    pub(crate) fn into_parts(self) -> (UiIntentAdmissionStopReason, UiIntentAdmissionCost) {
        (self.reason, self.cost)
    }
}

fn validate_target(
    origin: UiIntentAdmissionCandidateOrigin,
    payload: &super::super::payload::UiPreparedIntentPayload,
    mounted: &crate::mounting::WorthUiMountedSessionState,
) -> Result<
    crate::runtime::interaction::targeting::UiIntentExecutionTargetAffinity,
    UiIntentCandidateCurrentnessViolation,
> {
    let target = payload.input_basis().target();
    let affinity = match origin {
        UiIntentAdmissionCandidateOrigin::Direct => {
            crate::runtime::interaction::targeting::admit_presented_intent_execution_affinity(
                target, mounted,
            )
        }
        UiIntentAdmissionCandidateOrigin::Confirmed => {
            crate::runtime::interaction::targeting::admit_continued_intent_execution_affinity(
                target, mounted,
            )
        }
    };
    let affinity = affinity.map_err(UiIntentCandidateCurrentnessViolation::TargetChanged)?;
    if affinity.graph_node() == payload.graph_node() {
        Ok(affinity)
    } else {
        Err(UiIntentCandidateCurrentnessViolation::ProductRouteChanged)
    }
}

impl UiIntentCandidateCurrentnessViolation {
    fn from_dependency_drift(
        drift: super::super::operability::UiIntentOperabilityDependencyDrift,
    ) -> Self {
        use super::super::operability::UiIntentOperabilityDependencyDrift as Drift;
        use crate::declaration::UiIntentOperabilityDependencyAxis as Axis;
        match drift {
            Drift::DeclaredDependency
            | Drift::ExpressionResult {
                axis: Axis::Mutability | Axis::Readiness,
            } => Self::OperabilityDependencyChanged,
            Drift::Policy | Drift::ExpressionResult { axis: Axis::Policy } => Self::PolicyChanged,
            Drift::Confirmation => Self::ConfirmationPolicyChanged,
        }
    }

    fn into_admission_stop(self) -> UiIntentAdmissionStopReason {
        match self {
            Self::ApplicationWorldChanged => UiIntentAdmissionStopReason::ApplicationWorldChanged,
            Self::ApplicationGenerationChanged => {
                UiIntentAdmissionStopReason::ApplicationGenerationChanged
            }
            Self::PresentationInFlight => UiIntentAdmissionStopReason::PresentationInFlight,
            Self::TargetChanged(reason) => UiIntentAdmissionStopReason::TargetChanged(reason),
            Self::ProductRouteChanged => UiIntentAdmissionStopReason::ProductRouteChanged,
            Self::CommandContextChanged => UiIntentAdmissionStopReason::CommandContextChanged,
            Self::PayloadInputChanged => UiIntentAdmissionStopReason::PayloadInputChanged,
            Self::OperabilityDependencyChanged => {
                UiIntentAdmissionStopReason::OperabilityDependencyChanged
            }
            Self::PolicyChanged => UiIntentAdmissionStopReason::PolicyChanged,
            Self::ConfirmationPolicyChanged => {
                UiIntentAdmissionStopReason::ConfirmationPolicyChanged
            }
        }
    }

    fn into_execution_stop(
        self,
    ) -> crate::runtime::intent_execution::UiIntentExecutionCurrentnessStop {
        use crate::runtime::intent_execution::UiIntentExecutionCurrentnessStop as Stop;
        match self {
            Self::ApplicationWorldChanged => Stop::ApplicationWorldChanged,
            Self::ApplicationGenerationChanged => Stop::ApplicationGenerationChanged,
            Self::PresentationInFlight => Stop::PresentationInFlight,
            Self::TargetChanged(reason) => Stop::TargetChanged(reason),
            Self::ProductRouteChanged => Stop::ProductRouteChanged,
            Self::CommandContextChanged => Stop::CommandContextChanged,
            Self::PayloadInputChanged => Stop::PayloadInputChanged,
            Self::OperabilityDependencyChanged => Stop::OperabilityDependencyChanged,
            Self::PolicyChanged => Stop::PolicyChanged,
            Self::ConfirmationPolicyChanged => Stop::ConfirmationPolicyChanged,
        }
    }
}

fn product_route_is_current(
    candidate: &super::super::payload::UiPreparedIntentPayload,
    catalog: &crate::declaration::UiIntentCatalog,
    definitions: &crate::capability::FrozenIntentDefinitionCapabilities,
) -> bool {
    if candidate.command_route_receipt().is_some() {
        let Some((crate::declaration::UiIntentCatalogCommandRoute::Resolved { declaration }, _)) =
            catalog.lookup_command(candidate.definition_id())
        else {
            return false;
        };
        return Arc::ptr_eq(&declaration, candidate.declaration_reference())
            && definitions.definition_at(declaration.definition()).id()
                == candidate.definition_id();
    }
    let Some((crate::declaration::UiIntentCatalogResolvedRoute::Product { declaration, .. }, _)) =
        catalog.lookup(candidate.graph_node(), candidate.interaction_family())
    else {
        return false;
    };
    Arc::ptr_eq(&declaration, candidate.declaration_reference())
        && definitions.definition_at(declaration.definition()).id() == candidate.definition_id()
}
