//! A selected ordinary occurrence may recover itself, never replace a different pending owner.
use super::*;
use crate::domain_computation::primary_graph::{
    workflow::instance::SelectedWorkflowTransition, WorthQueryPrimaryGraphIntegrationHandle,
};
use worth_relational::facade::snapshots::SnapshotHandle;
impl ObservedWorkflowInstance {
    /// Retains absence too: a custody commit after preparation must stale this candidate.
    /// Foreign custody pays existing bounded history reconstruction only when present.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph::application_attempt) fn fence_other_owner(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &WorthQueryPrimaryGraphIntegrationHandle,
        snapshot: &SnapshotHandle,
        layout: &WorthQueryWorkflowLayout,
        instance: EntityId,
        compiled: &CompiledWorkflowDefinition,
        selected: Option<&SelectedWorkflowTransition>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let (custody, fact) = handle.with_runtime(execution, |runtime| {
            instance_binding::owner_custody(execution, runtime, snapshot, layout, instance)
        })?;
        if let Some(custody) = custody {
            let same_occurrence=selected.is_some_and(|selected| selected.identity()==custody
                && matches!(selected.kind(),crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowTransitionKind::Operation(_)));
            if !same_occurrence {
                handle.with_runtime(execution, |runtime| {
                    self.ensure_history(
                        execution,
                        handle,
                        runtime,
                        snapshot,
                        layout,
                        instance,
                        self.maximum_transitions,
                        compiled,
                    )
                })?;
                if !self
                    .transitions
                    .iter()
                    .any(|settled| settled.identity == custody)
                {
                    return Err(WorthQueryApplicationAttemptDenial::new(
                        WorthQueryApplicationAttemptDenialKind::WorkflowOperationInOwnerCustody,
                        "workflow frontier cannot bypass a different performed pending operation",
                    ));
                }
            }
        }
        self.facts.push(fact);
        Ok(())
    }
}
