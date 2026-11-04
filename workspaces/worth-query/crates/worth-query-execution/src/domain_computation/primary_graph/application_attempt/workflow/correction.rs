//! A correction preserves its source workflow's current continuation and custody.
use super::{instance::WorkflowInstanceState, operation_output::WorkflowOperationOrigin};
use crate::domain_computation::primary_graph::application_attempt::{
    observe_field_value, WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use worth_foundational::facade::AspectValue;
mod settlement;

/// Descriptive disposition of one exact target, not authority to advance a workflow.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct PreparedWorkflowCorrection {
    origin: WorkflowOperationOrigin,
    state: WorkflowInstanceState,
    owner_custody_transition: Option<String>,
}

impl PreparedWorkflowCorrection {
    /// The inverse may change authored subject facts, but cannot rewrite the
    /// source instance's current node, completion state or external custody.
    pub(in crate::domain_computation::primary_graph) fn validate_candidate(
        &self,
        batch: &worth_relational::facade::transactions::WorkerIntentBatch,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        preserve_instance(batch, self.origin.instance())
    }
}

fn preserve_instance(
    batch: &worth_relational::facade::transactions::WorkerIntentBatch,
    instance: worth_relational::facade::identity::EntityId,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    use worth_relational::facade::transactions::{EntityMutationIntent, MutationIntent};
    for intent in &batch.intents {
        let MutationIntent::Entity(intent) = intent else {
            continue;
        };
        let entity = match intent {
            EntityMutationIntent::UpdateFields(intent) => intent.entity_id,
            EntityMutationIntent::ApplyAspectPatch(intent) => intent.entity_id,
            EntityMutationIntent::Replace(intent) => intent.entity_id,
            EntityMutationIntent::Delete(intent) => intent.entity_id,
            EntityMutationIntent::Revalidate(_) => continue,
        };
        if entity == instance {
            return Err(unavailable(
                "correction must preserve its source workflow continuation",
            ));
        }
    }
    Ok(())
}

/// Constant work in the selected instance. Existing custody stays with its owner.
pub(in crate::domain_computation::primary_graph) fn retain_current_continuation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    origin: &WorkflowOperationOrigin,
) -> Result<PreparedWorkflowCorrection, WorthQueryApplicationAttemptDenial> {
    let layout = origin.layout();
    let state = observe_field_value(
        runtime,
        execution,
        snapshot,
        origin.instance(),
        layout.instance.entity_kind,
        &layout.instance.state,
    )
    .map_err(|cause| {
        WorthQueryApplicationAttemptDenial::relational_read_stopped(
            cause,
            "correction workflow continuation",
        )
    })?;
    let Some(AspectValue::UInt64(state)) = state else {
        return Err(unavailable("source workflow instance is unavailable"));
    };
    let state = WorkflowInstanceState::from_persisted_tag(state)
        .ok_or_else(|| unavailable("source workflow state is malformed"))?;
    let (owner_custody_transition, _) =
        crate::domain_computation::primary_graph::application_attempt::workflow_instance_observation::instance_binding::owner_custody(
            execution, runtime, snapshot, layout, origin.instance(),
        )?;
    settlement::require_settled_origin_and_custody(
        execution,
        runtime,
        snapshot,
        origin,
        owner_custody_transition.as_deref(),
    )?;
    Ok(PreparedWorkflowCorrection {
        origin: origin.clone(),
        state,
        owner_custody_transition,
    })
}

pub(in crate::domain_computation::primary_graph) fn unavailable(
    subject: &str,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}

#[cfg(test)]
mod tests;
