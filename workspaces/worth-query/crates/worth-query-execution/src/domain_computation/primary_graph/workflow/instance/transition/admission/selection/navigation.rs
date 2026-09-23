use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::identity::EntityId;

use super::{denial, SettledWorkflowTransition};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use crate::domain_computation::primary_graph::workflow::definition::{
    CompiledWorkflowDefinition, CompiledWorkflowNode,
};

pub(super) fn unique_successor<'compiled>(
    compiled: &'compiled CompiledWorkflowDefinition,
    source: EntityId,
    outcome: ApplicationWorkflowControlOutcome,
    history: &[SettledWorkflowTransition],
) -> Result<&'compiled CompiledWorkflowNode, WorthQueryApplicationAttemptDenial> {
    let mut retries = compiled.retry_successors(source, outcome);
    let retry = retries.next();
    if retries.next().is_some() {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
            "settled workflow transition has ambiguous retry successors",
        ));
    }
    let target = if let Some((target, maximum_attempts)) = retry {
        let attempts = history
            .iter()
            .filter(|transition| transition.node == source && transition.outcome == outcome)
            .count();
        if attempts <= usize::from(maximum_attempts) {
            target.entity()
        } else {
            unique_successor_entity(
                compiled
                    .control_successors(source, ApplicationWorkflowControlOutcome::RetryExhausted)
                    .map(CompiledWorkflowNode::entity),
            )?
        }
    } else {
        unique_successor_entity(
            compiled
                .control_successors(source, outcome)
                .map(CompiledWorkflowNode::entity),
        )?
    };
    compiled
        .nodes()
        .find(|node| node.entity() == target)
        .ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                "settled workflow successor is absent from the compiled node inventory",
            )
        })
}

pub(super) fn unique_successor_entity(
    targets: impl IntoIterator<Item = EntityId>,
) -> Result<EntityId, WorthQueryApplicationAttemptDenial> {
    let mut targets = targets.into_iter();
    let Some(target) = targets.next() else {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
            "settled workflow transition has no compiled successor",
        ));
    };
    if targets.next().is_some() {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
            "settled workflow transition has ambiguous compiled successors",
        ));
    }
    Ok(target)
}
