use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::identity::EntityId;

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
    retry_attempts: usize,
) -> Result<&'compiled CompiledWorkflowNode, WorthQueryApplicationAttemptDenial> {
    let mut retries = compiled.retry_successors(source, outcome);
    let retry = retries.next();
    if retries.next().is_some() {
        return Err(denial(
            "settled workflow transition has ambiguous retry successors",
        ));
    }
    if let Some((target, maximum_attempts)) = retry {
        if retry_attempts <= usize::from(maximum_attempts) {
            Ok(target)
        } else {
            unique_successor_node(
                compiled
                    .control_successors(source, ApplicationWorkflowControlOutcome::RetryExhausted),
            )
        }
    } else {
        unique_successor_node(compiled.control_successors(source, outcome))
    }
}

fn unique_successor_node<'compiled>(
    targets: impl IntoIterator<Item = &'compiled CompiledWorkflowNode>,
) -> Result<&'compiled CompiledWorkflowNode, WorthQueryApplicationAttemptDenial> {
    let mut targets = targets.into_iter();
    let Some(target) = targets.next() else {
        return Err(denial(
            "settled workflow transition has no compiled successor",
        ));
    };
    if targets.next().is_some() {
        return Err(denial(
            "settled workflow transition has ambiguous compiled successors",
        ));
    }
    Ok(target)
}

#[cfg(test)]
fn unique_successor_entity(
    targets: impl IntoIterator<Item = EntityId>,
) -> Result<EntityId, WorthQueryApplicationAttemptDenial> {
    let mut targets = targets.into_iter();
    let Some(target) = targets.next() else {
        return Err(denial(
            "settled workflow transition has no compiled successor",
        ));
    };
    if targets.next().is_some() {
        return Err(denial(
            "settled workflow transition has ambiguous compiled successors",
        ));
    }
    Ok(target)
}

fn denial(subject: &'static str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_relational::facade::identity::PartitionId;

    fn entity(slot: u64) -> EntityId {
        EntityId::new(PartitionId::new(7), slot, 1)
    }

    #[test]
    fn missing_and_ambiguous_successors_fail_closed() {
        assert!(unique_successor_entity([]).is_err());
        assert!(unique_successor_entity([entity(31), entity(32)]).is_err());
        assert_eq!(
            unique_successor_entity([entity(33)]).expect("one successor is exact"),
            entity(33)
        );
    }
}
