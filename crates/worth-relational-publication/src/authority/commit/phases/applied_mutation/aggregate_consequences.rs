//! Complete reversible Native aggregate writes before judging the proposed state.
use super::aggregate_inputs::{AggregateDraftAdmission, AggregateDraftInputs};
use super::aggregate_metrics::count_aggregate_maintenance_reads;
use super::AppliedMutationDraft;
use contracts::schema::data::RelationalSchemaRegistryFieldsPort as _;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflict;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::authority::mutation::execution::apply_plan_to_working_state;
use worth_relational_mutation::internal_port::authority::mutation::workspace::BranchLocalDeleteAllowance;
use worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentityOwnerPort as _;

impl AppliedMutationDraft {
    pub fn complete_aggregate_draft<'a>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        inputs: impl Into<AggregateDraftInputs<'a>>,
        selected: &worth_relational_branch::internal_port::branch::selected_read::SelectedRelationalBranchRead<'_>,
        business: worth_relational_mutation::internal_port::authority::commit::semantic_plan::meaning::RelationalSemanticPlan<'_>,
        admitted: impl Into<AggregateDraftAdmission<'a>>,
        identity: &worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
    ) -> Result<Option<worth_relational_mutation::internal_port::aggregate_identity::preparation::AggregateAdmission>, TransactionCommitError>{
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
        let current_ownership =
            (*authority.registry().owner_current_publication_storage()).is_some();
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
                    worth_relational_mutation::internal_port::aggregate_identity::observations::AggregateReadObservations::default(),
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
                let config = contracts::config::data::policies::MutationConfig {
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
                    contracts::execution::request_custody::request_lease(execution),
                    inputs.work.cloned(),
                )
                .map_err(|denial| match denial {
                    worth_relational_mutation::internal_port::authority::mutation::round_denial::MutationApplyDenial::Commit(error) => error,
                    worth_relational_mutation::internal_port::authority::mutation::round_denial::MutationApplyDenial::Symbols(cause) => {
                        contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::publication(
                            contracts::publication::data::publication_error::PublicationError::new(
                                contracts::publication::bundle::publication_status::PublicationStage::BundleAssembly,
                                format!("aggregate consequence symbol admission: {cause:?}"),
                            ),
                        )
                    }
                })?;
                worth_relational_mutation::internal_port::authority::mutation::record_changes::adjacency::apply_adjacency_deltas_for_request(
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
            let final_reads = worth_relational_mutation::internal_port::aggregate_identity::final_laws::validate_final_derived_laws(
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
