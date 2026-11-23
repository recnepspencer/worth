//! Complete reversible Native aggregate writes before judging the proposed state.
use super::aggregate_inputs::{AggregateDraftAdmission, AggregateDraftInputs};
use super::aggregate_metrics::count_aggregate_maintenance_reads;
use super::AppliedMutationDraft;
use crate::authority::mutation::{apply_plan_to_working_state, BranchLocalDeleteAllowance};
use crate::transactions::data::outcomes::commit_conflict::CommitConflict;
use crate::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;

impl AppliedMutationDraft {
    pub(crate) fn complete_aggregate_draft<'a>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        inputs: impl Into<AggregateDraftInputs<'a>>,
        selected: &crate::branch::selected_read::SelectedRelationalBranchRead<'_>,
        business: crate::authority::commit::semantic_plan::RelationalSemanticPlan<'_>,
        admitted: impl Into<AggregateDraftAdmission<'a>>,
        identity: &crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
    ) -> Result<Option<crate::aggregate_identity::AggregateAdmission>, TransactionCommitError> {
        let inputs = inputs.into();
        let admitted = admitted.into();
        let mut admission = None;
        let version_id = identity.proposed_version_id();
        let occurrence = identity.commit_id();
        let working = &mut self.state;
        let aggregate_source = self
            .aggregate
            .as_ref()
            .map(|owner| owner.borrow(working, *selected));
        let authority = admitted.schema;
        let current_ownership = authority.registry().current_publication_storage.is_some();
        if !admitted.materialization_only {
            let (prepared, reads) = match aggregate_source {
                Some(source) => source.take_for_commit(
                    execution,
                    inputs.work,
                    business.commands(),
                    occurrence,
                )?,
                None => (
                    None,
                    crate::aggregate_identity::AggregateReadObservations::default(),
                ),
            };
            // Consumption is single-use even when no consequence was needed.
            // The report includes the one preparation already shared with rules.
            count_aggregate_maintenance_reads(inputs.instrumentation, reads);
            if let Some(prepared) = prepared {
                let mut prepared = prepared;
                prepared.materialize(
                    execution,
                    inputs.work,
                    selected.selected(),
                    working,
                    inputs.symbols,
                    admitted.basis,
                )?;
                let config = crate::config::data::MutationConfig {
                    cascade_delete_policy: inputs.storage.cascade_delete_policy,
                    adjacency_policy: inputs.storage.adjacency_policy.clone(),
                    cross_context_policy: inputs.storage.cross_context_policy,
                };
                let applied = apply_plan_to_working_state(
                    execution,
                    working,
                    &prepared.apply,
                    &config,
                    authority.registry(),
                    authority.aspect_plans(),
                    &mut self.identities.symbols,
                    BranchLocalDeleteAllowance::default(),
                    &mut self.identities.records,
                    crate::authority::commit::pipeline::request_lease(execution),
                    inputs.work.cloned(),
                )
                .map_err(|denial| match denial {
                    crate::authority::mutation::MutationApplyDenial::Commit(error) => error,
                    crate::authority::mutation::MutationApplyDenial::Symbols(cause) => {
                        crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::publication(
                            crate::publication::data::PublicationError::new(
                                crate::publication::bundle::PublicationStage::BundleAssembly,
                                format!("aggregate consequence symbol admission: {cause:?}"),
                            ),
                        )
                    }
                })?;
                crate::authority::mutation::apply_adjacency_deltas_for_request(
                    execution,
                    working,
                    &applied.effect.adjacency.deltas,
                    version_id,
                )?;
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
                inputs.work,
                authority,
                selected.selected(),
                working,
                &self.outcome.effect,
                version_id,
                inputs.symbols,
                admitted.basis,
            )?;
            count_aggregate_maintenance_reads(inputs.instrumentation, final_reads);
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
