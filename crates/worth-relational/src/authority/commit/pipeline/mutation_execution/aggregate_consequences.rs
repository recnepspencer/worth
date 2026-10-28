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
        let selected = self.prepared.selected_branch_state().clone();
        let version_id = self.prepared.version_id();
        let admitted = &mut self.prepared.admitted;
        let working = &mut self.working_state;
        let aggregate_source = self.aggregate.as_ref().map(|owner| {
            owner.borrow(
                working,
                crate::branch::SelectedRelationalBranchRead::in_round(&selected, &selected),
            )
        });
        let (_, validation, business, _, _, _) = admitted.phase_view().into_parts();
        let authority = validation.schema_authority();
        if validation.intent().materialization_mode().is_none() {
            let (prepared, reads) = match aggregate_source {
                Some(source) => source.take_for_commit(
                    execution,
                    runtime.commit_work_budget.as_ref(),
                    business,
                    occurrence,
                )?,
                None => (
                    None,
                    crate::aggregate_identity::AggregateReadObservations::default(),
                ),
            };
            let performance = crate::performance::PerformanceAccess::from_instrumentation(
                &runtime.services.instrumentation,
            );
            // Consumption is single-use even when no consequence was needed.
            // The report includes the one preparation already shared with rules.
            performance.count_aggregate_maintenance_reads(reads);
            let mut consequences_applied = false;
            if let Some(prepared) = prepared {
                let mut prepared = prepared;
                prepared.materialize(
                    execution,
                    runtime.commit_work_budget.as_ref(),
                    &selected,
                    working,
                    &runtime.services.symbols,
                    validation.basis(),
                )?;
                let config = crate::config::data::MutationConfig {
                    cascade_delete_policy: runtime.config.storage.cascade_delete_policy,
                    adjacency_policy: runtime.config.storage.adjacency_policy.clone(),
                    cross_context_policy: runtime.config.storage.cross_context_policy,
                };
                let applied = apply_plan_to_working_state(
                    working,
                    &prepared.apply,
                    &config,
                    authority.registry(),
                    authority.aspect_plans(),
                    &mut self.mutation_identities.symbols,
                    BranchLocalDeleteAllowance::default(),
                    &mut self.mutation_identities.records,
                    crate::authority::commit::pipeline::request_lease(execution),
                    runtime.commit_work_budget.clone(),
                )
                .map_err(|denial| match denial {
                    crate::authority::mutation::MutationApplyDenial::Commit(error) => error,
                    crate::authority::mutation::MutationApplyDenial::Symbols(cause) => {
                        crate::transactions::data::TransactionCommitError::publication(
                            crate::publication::data::PublicationError::new(
                                crate::publication::bundle::PublicationStage::BundleAssembly,
                                format!("aggregate consequence symbol admission: {cause:?}"),
                            ),
                        )
                    }
                })?;
                crate::authority::mutation::apply_adjacency_deltas(
                    working,
                    &applied.effect.adjacency.deltas,
                    version_id,
                );
                self.outcome
                    .created_entities
                    .append_owned(applied.created_entities)?;
                self.outcome
                    .created_relations
                    .append_owned(applied.created_relations)?;
                self.outcome.effect.accumulate(applied.effect);
                self.aggregate_admission = Some(prepared.admission);
                consequences_applied = true;
            }
            // Metadata-dependent recipes see actual Native membership/lineage
            // writes. They cannot be checked early against a degree-zero birth,
            // and still require validation when no consequence was necessary.
            let final_reads = crate::aggregate_identity::validate_final_derived_laws(
                execution,
                runtime.commit_work_budget.as_ref(),
                authority,
                &selected,
                working,
                &self.outcome.effect,
                version_id,
                &runtime.services.symbols,
                validation.basis(),
            )?;
            performance.count_aggregate_maintenance_reads(final_reads);
            if consequences_applied {
                // Earlier publication evidence described only the business graph.
                admitted.take_prevalidated_snapshot_publication();
            }
        }
        self.mutation_identities
            .records
            .finish_mutation()
            .map_err(|denial| {
                TransactionCommitError::conflict(CommitConflict::new(
                    ConflictClass::RecordAllocationDenied { denial },
                ))
            })
    }
}
