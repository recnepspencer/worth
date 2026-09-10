use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::node::EvaluationCondition;
use crate::data::proof::invalidation::revalidation::NodeInvalidationInput;
use crate::logic::evaluation::{ConditionEvaluationContext, EvaluationRequestMode};

use super::{
    InstalledSignalConditionDecision, InstalledSignalConditionResolver,
    SignalConditionalExecutionRequest,
};

pub(super) enum ConditionDisposition {
    Eligible,
    Suppressed,
    Deferred,
    DeferredTemporal,
    DeferredOnDemand,
}

pub(super) fn resolve_condition(
    graph: &SignalGraph,
    request: &SignalConditionalExecutionRequest<'_>,
    resolver: &mut impl InstalledSignalConditionResolver,
    work: &mut crate::data::retained_storage::RetainedStoragePreparation,
) -> Result<ConditionDisposition, SignalError> {
    let node = request.contract.node();
    let external_trigger_requested = request.force_on_demand
        || matches!(
            request.contract.condition(),
            EvaluationCondition::Installed(identity)
                if identity.role() == crate::data::node::InstalledSignalConditionRole::TemporalWake
        );
    if graph.conditional_has_current_unsettled_upstream(node, work)? {
        return Ok(ConditionDisposition::Deferred);
    }
    let dirty_aspects = match graph.node_invalidation_input(node)? {
        NodeInvalidationInput::Pending(_) => return Ok(ConditionDisposition::Deferred),
        NodeInvalidationInput::Resolved(causes) => causes.dirty_aspects(),
        NodeInvalidationInput::ResolvedNoChange(_) if external_trigger_requested => {
            crate::data::aspect::AspectMask::EMPTY
        }
        NodeInvalidationInput::ResolvedNoChange(_) => return Ok(ConditionDisposition::Suppressed),
    };
    let trigger_aspects = request.contract.trigger_aspects();
    let trigger_dirty_aspects = if trigger_aspects.is_empty() {
        dirty_aspects
    } else {
        crate::data::aspect::AspectMask::from_bits(dirty_aspects.bits() & trigger_aspects.bits())
    };
    let context = ConditionEvaluationContext {
        node,
        request_mode: if request.force_on_demand {
            EvaluationRequestMode::ForceOnDemand
        } else {
            EvaluationRequestMode::Default
        },
        dirty_aspects: trigger_dirty_aspects,
        max_dependency_delta: max_dependency_delta(graph, node, work)?,
        required_context: graph.get_contract(node)?.semantics.required_context,
    };
    Ok(match request.contract.condition() {
        EvaluationCondition::Always => ConditionDisposition::Eligible,
        EvaluationCondition::AspectFilter(mask) => {
            if dirty_aspects.is_empty() || dirty_aspects.intersects(*mask) {
                ConditionDisposition::Eligible
            } else {
                ConditionDisposition::Suppressed
            }
        }
        EvaluationCondition::OnDemand if request.force_on_demand => ConditionDisposition::Eligible,
        EvaluationCondition::OnDemand => ConditionDisposition::DeferredOnDemand,
        EvaluationCondition::Installed(identity) => {
            match (identity.role(), resolver.resolve(identity, &context)?) {
                (_, InstalledSignalConditionDecision::Eligible) => ConditionDisposition::Eligible,
                (_, InstalledSignalConditionDecision::Suppressed) => {
                    ConditionDisposition::Suppressed
                }
                (
                    crate::data::node::InstalledSignalConditionRole::TemporalWake,
                    InstalledSignalConditionDecision::Deferred,
                ) => ConditionDisposition::DeferredTemporal,
                (_, InstalledSignalConditionDecision::Deferred) => ConditionDisposition::Deferred,
            }
        }
        EvaluationCondition::Temporal(_) => ConditionDisposition::DeferredTemporal,
        EvaluationCondition::DeltaThreshold(threshold) => {
            if context.max_dependency_delta as f64 > *threshold {
                ConditionDisposition::Eligible
            } else {
                ConditionDisposition::Suppressed
            }
        }
        EvaluationCondition::Custom(_) => {
            return Err(SignalError::invalid_input(
                "portable custom strings are not installed conditional authority",
            ));
        }
    })
}

fn max_dependency_delta(
    graph: &SignalGraph,
    node: crate::data::handle::NodeId,
    work: &mut crate::data::retained_storage::RetainedStoragePreparation,
) -> Result<u64, SignalError> {
    let snapshot = graph.get_dep_snapshot(node)?;
    let mut maximum = 0;
    for entry in snapshot.entries() {
        let current = graph.conditional_node_version_for_scope(
            entry.source,
            entry.aspect,
            entry.scope.as_ref(),
            work,
        )?;
        maximum = maximum.max(current.abs_diff(entry.cached_version));
    }
    Ok(maximum)
}
