//! Complete reversible Native aggregate writes before judging the proposed state.
use super::AppliedMutationDraft;
use crate::authority::mutation::{apply_plan_to_working_state, BranchLocalDeleteAllowance};
use crate::transactions::data::{CommitConflict, ConflictClass, TransactionCommitError};

impl AppliedMutationDraft {
    pub(crate) fn complete_aggregate_draft(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalPreparationRuntime,
        selected: &crate::branch::SelectedRelationalBranchRead<'_>,
        business: crate::authority::commit::semantic_plan::RelationalSemanticPlan<'_>,
        validation: &crate::mvcc::RelationalTransactionValidationInput,
        identity: &crate::mvcc::RelationalMutationProposalIdentity,
    ) -> Result<Option<crate::aggregate_identity::AggregateAdmission>, TransactionCommitError> {
        let mut admission = None;
        let version_id = identity.proposed_version_id();
        let occurrence = identity.commit_id();
        let working = &mut self.state;
        let aggregate_source = self
            .aggregate
            .as_ref()
            .map(|owner| owner.borrow(working, *selected));
        let authority = validation.schema_authority();
        let current_ownership = authority.registry().current_publication_storage.is_some();
        if validation.intent().materialization_mode().is_none() {
            let (prepared, reads) = match aggregate_source {
                Some(source) => source.take_for_commit(
                    execution,
                    runtime.commit_work_budget.as_ref(),
                    business.commands(),
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
            if let Some(prepared) = prepared {
                let mut prepared = prepared;
                prepared.materialize(
                    execution,
                    runtime.commit_work_budget.as_ref(),
                    selected.selected(),
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
                    &mut self.identities.symbols,
                    BranchLocalDeleteAllowance::default(),
                    &mut self.identities.records,
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
                admission = Some(prepared.admission);
            }
            // Metadata-dependent recipes see actual Native membership/lineage
            // writes. They cannot be checked early against a degree-zero birth,
            // and still require validation when no consequence was necessary.
            let final_reads = crate::aggregate_identity::validate_final_derived_laws(
                execution,
                runtime.commit_work_budget.as_ref(),
                authority,
                selected.selected(),
                working,
                &self.outcome.effect,
                version_id,
                &runtime.services.symbols,
                validation.basis(),
            )?;
            performance.count_aggregate_maintenance_reads(final_reads);
        }
        if current_ownership {
            // This publication's Native current-ownership consequences share
            // the remaining replay allocation stream and finish it at completion.
            return Ok(admission);
        }
        self.identities
            .records
            .finish_mutation()
            .map_err(|denial| {
                TransactionCommitError::conflict(CommitConflict::new(
                    ConflictClass::RecordAllocationDenied { denial },
                ))
            })?;
        Ok(admission)
    }
}
