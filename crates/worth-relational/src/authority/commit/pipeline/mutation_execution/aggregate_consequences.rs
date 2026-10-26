//! History-bound aggregate effects share the business apply's allocation stream.
use super::MutatedCommitExecution;
use crate::authority::mutation::{apply_plan_to_working_state, BranchLocalDeleteAllowance};
use crate::transactions::data::{CommitConflict, ConflictClass, TransactionCommitError};

impl MutatedCommitExecution {
    pub(in crate::authority::commit::pipeline) fn bind_aggregate_consequences(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalPreparationRuntime,
        occurrence: crate::history::data::CommitId,
    ) -> Result<(), TransactionCommitError> {
        let selected = self
            .validated
            .prepared_mut()
            .selected_branch_state()
            .clone();
        let (admitted, working, aggregate_source) = self.validated.prepared_mut().mutation_parts();
        let (_, validation, business, _, _, _) = admitted.phase_view().into_parts();
        let authority = validation.schema_authority();
        if validation.intent().materialization_mode().is_none() {
            let prepared = match aggregate_source {
                Some(source) => source.take_for_commit(
                    execution,
                    runtime.commit_work_budget.as_ref(),
                    business,
                    occurrence,
                )?,
                None => None,
            };
            if let Some(prepared) = prepared {
                let mut prepared = prepared;
                prepared.materialize(
                    execution,
                    runtime.commit_work_budget.as_ref(),
                    &selected,
                    working,
                )?;
                let config = crate::config::data::MutationConfig {
                    cascade_delete_policy: runtime.config.storage.cascade_delete_policy,
                    adjacency_policy: runtime.config.storage.adjacency_policy.clone(),
                    cross_context_policy: runtime.config.storage.cross_context_policy,
                };
                let applied = runtime.services.symbols.with_write(|symbols| {
                    apply_plan_to_working_state(
                        working,
                        &prepared.apply,
                        &config,
                        authority.registry(),
                        authority.aspect_plans(),
                        symbols,
                        BranchLocalDeleteAllowance::default(),
                        &mut self.record_allocations,
                        crate::authority::commit::pipeline::request_lease(execution),
                        runtime.commit_work_budget.clone(),
                    )
                })?;
                self.created_entities
                    .append_owned(applied.created_entities)?;
                self.created_relations
                    .append_owned(applied.created_relations)?;
                self.effect.accumulate(applied.effect);
                self.aggregate_admission = Some(prepared.admission);
                // Earlier publication evidence described only the business graph.
                admitted.take_prevalidated_snapshot_publication();
            }
        }
        self.record_allocations.finish_mutation().map_err(|denial| {
            TransactionCommitError::conflict(CommitConflict::new(
                ConflictClass::RecordAllocationDenied { denial },
            ))
        })
    }
}
