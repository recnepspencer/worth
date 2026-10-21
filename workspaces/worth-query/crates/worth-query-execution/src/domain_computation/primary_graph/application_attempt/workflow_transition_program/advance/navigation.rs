//! Native cursor navigation cannot abandon performed pending business custody.
use super::*;
use crate::domain_computation::primary_graph::{
    workflow::{definition::CompiledWorkflowDefinition, schema::WorthQueryWorkflowLayout},
    WorthQueryApplicationObservedFact,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
{
    #[allow(clippy::too_many_arguments)]
    pub(super) fn materialize_observed_navigation_back(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        layout: &WorthQueryWorkflowLayout,
        compiled: CompiledWorkflowDefinition,
        instance: super::super::super::PublishedWorkflowInstanceRef,
        observed: &mut super::super::super::workflow_instance_observation::ObservedWorkflowInstance,
        live_membership: worth_relational::facade::identity::RelationId,
        mut facts: Vec<WorthQueryApplicationObservedFact>,
        allowance: super::super::super::WorkflowStepAllowance,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        let selected = select_navigation_back_transition(
            &compiled,
            instance.entity_id(),
            &observed.progress_basis,
        )
        .and_then(|selected| {
            observed.fence_other_owner(
                execution,
                self.lease.handle(),
                self.lease.snapshot(),
                layout,
                instance.entity_id(),
                &compiled,
                Some(&selected),
            )?;
            Ok(selected)
        });
        let selected = match selected {
            Ok(selected) => selected,
            Err(denial) => {
                return Ok(self.navigation_replay_denial(
                    layout,
                    instance.entity_id(),
                    std::mem::take(&mut observed.replays),
                    denial,
                ))
            }
        };
        let replay_probe_identity = *selected.identity_bytes();
        let replays = publication::PreparedWorkflowTransitionReplays::retained(std::mem::take(
            &mut observed.replays,
        ))
        .for_navigation_back();
        facts.append(&mut observed.facts);
        self.materialize_navigation_back(
            layout,
            compiled,
            instance,
            selected,
            live_membership,
            facts,
            allowance,
        )
        .map(|prepared| prepared.with_replays(replays.with_probe_identity(replay_probe_identity)))
    }
}
