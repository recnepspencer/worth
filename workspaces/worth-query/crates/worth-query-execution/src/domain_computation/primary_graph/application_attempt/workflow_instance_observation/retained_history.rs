use super::*;

impl ObservedWorkflowInstance {
    /// Every step the lineage has taken up to this instance's next one: its
    /// sources' and its own. Migration and fork continuation carry it, so a
    /// loop through a successor spends the same budget.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn lineage_steps(
        &self,
    ) -> u64 {
        self.inherited_steps
            .saturating_add(self.progress_basis.progress().next_occurrence())
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn ensure_history(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        layout: &WorthQueryWorkflowLayout,
        instance: EntityId,
        maximum_transitions: usize,
        compiled: &CompiledWorkflowDefinition,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if self.history_materialized {
            return Ok(());
        }
        let mut history = history::observe(
            execution,
            runtime,
            snapshot,
            layout,
            instance,
            history_basis(
                self.live_membership.is_some(),
                self.purpose,
                self.inherited_steps,
            ),
            maximum_transitions,
            history::HistoryNodes::Compiled(compiled),
            self.history_budget,
        )?;
        let transition_visits = history.transition_visits;
        self.transitions = history.transitions;
        let (key, _) = self.progress_basis.replay_retention();
        handle.with_workflow_instance_progress_mut(key, |retention| {
            retention.observe_warm_history(transition_visits);
            retention.observe_history_reconstruction_charge(history.charge_bytes);
        });
        self.facts.append(&mut history.facts);
        self.history_materialized = true;
        Ok(())
    }
}
