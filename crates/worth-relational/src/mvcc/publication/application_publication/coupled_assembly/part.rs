//! Admit and apply one authenticated surviving writer's own successor commands.
use super::*;
use crate::execution::request_stage::checkpoint;
use crate::{
    authority::commit::phases::prepare::prepare_authoritative_working_state_scope_for_base,
    publication::data::AuthoredRetirementRefusal as Cause,
    transactions::data::{AuthoritativeApplyPlan, RecordRef, WorkerIntentBatch},
};

pub(super) fn reject(
    cause: Cause,
    subject: crate::identity::data::EntityId,
) -> TransactionCommitError {
    TransactionCommitError::publication(
        crate::publication::data::PublicationError::authored_retirement(
            cause,
            RecordRef::Entity(subject),
        ),
    )
}

pub(super) fn require_unseen_route(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    parts: &[CoupledPublicationPart],
    meaning: &RequiredOutputPublicationMeaning,
) -> Result<(), TransactionCommitError> {
    for part in parts {
        let prior = part
            .publication
            .producing_meaning()
            .expect("admitted producer route");
        checkpoint(
            context,
            (meaning.family.len() + prior.family.len() + 65) as u64,
        )
        .map_err(custody::execution_stop)?;
        if prior.same_route(meaning) {
            return Err(reject(Cause::RepeatedCoupledOwner, meaning.source_scope));
        }
    }
    Ok(())
}

impl RelationalAuthoredCoupledCandidate<'_> {
    pub fn append_publication(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        mut publication: RequiredOutputPublication,
        content: &crate::content::RelationalContentAttachments,
        mut batches: Vec<WorkerIntentBatch>,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        let predecessor_origin =
            super::participant_admission::admit(self, execution, &publication)?;
        let roles = match &mut publication {
            RequiredOutputPublication::Birth { roles, .. }
            | RequiredOutputPublication::Preserve { roles, .. } => roles,
            _ => unreachable!("admitted producer"),
        };
        super::local_keys::scope(
            execution,
            self.parts.len(),
            &self.mutation.outcome.created_entities,
            self.mutation.candidate_symbols().interner(),
            &mut batches,
            roles,
        )?;
        let before = self
            .transaction
            .batches()
            .iter()
            .map(|batch| batch.intents.len())
            .sum::<usize>();
        self.transaction
            .attach_content(execution, content)
            .map_err(crate::content::commit_denial)?;
        for mut batch in batches {
            super::fresh_references::retain(
                execution,
                &self.mutation.outcome.created_entities,
                &mut batch.intents,
            )?;
            self.transaction
                .push_batch(batch, execution.allocation_policy())
                .map_err(|failure| TransactionCommitError::conflict(failure.into_conflict()))?;
        }
        // Symbols discovered by a part retain the enclosing ordinary symbol
        // round's abandon-and-retry behavior. No Native effect has published.
        self.transaction
            .reserve_preparation_client_keys(
                &mut self.mutation.identities.symbols,
                self.runtime.config.identity.client_key_symbol_policy,
                execution.allocation_policy(),
            )
            .map_err(RelationalCoupledAssemblyFailure::from_apply)?;
        let runtime = self
            .runtime
            .with_candidate_symbols(self.mutation.candidate_symbols());
        let mut intents = self
            .transaction
            .normalized_intents_for_merge(&runtime, execution.allocation_policy())
            .map_err(TransactionCommitError::conflict)?
            .into_iter()
            .skip(before)
            .collect::<Vec<_>>();
        super::fresh_references::resolve(
            execution,
            &self.mutation.outcome.created_entities,
            &mut intents,
        )?;
        let read = self.pin_source(execution)?;
        let root = read.resident_basis().root();
        let plan = self
            .transaction
            .build_merged_plan_for_state(&runtime, root.as_ref(), intents)
            .map_err(TransactionCommitError::conflict)?;
        let deferred_edges = super::writer_custody::require(
            execution,
            &self.binding.owners,
            super::writer_custody::Writer::Producer(predecessor_origin.as_deref()),
            &plan,
        )?;
        let (_, mut state, _) = prepare_authoritative_working_state_scope_for_base(
            &runtime,
            root.as_ref(),
            &plan,
            0,
            None,
        );
        crate::aggregate_identity::guard::ordinary_plan(
            execution,
            runtime.commit_work_budget.as_ref(),
            self.prepared.schema_authority.registry(),
            &crate::validation::engine::state_view::InvariantStateView::new(
                root.as_ref(),
                self.identity.proposed_version_id(),
            ),
            &plan.merged_intents,
            None,
        )?;
        let config = crate::config::data::MutationConfig {
            cascade_delete_policy: runtime.config.storage.cascade_delete_policy,
            adjacency_policy: runtime.config.storage.adjacency_policy.clone(),
            cross_context_policy: runtime.config.storage.cross_context_policy,
        };
        let apply = AuthoritativeApplyPlan {
            transaction_id: plan.transaction_id,
            version_id: self.identity.proposed_version_id(),
            merged_intents: plan.merged_intents.clone(),
        };
        let outcome = crate::authority::mutation::apply_plan_with_publication_edges(
            &mut state,
            &apply,
            &config,
            self.prepared.schema_authority.registry(),
            self.prepared.schema_authority.aspect_plans(),
            &mut self.mutation.identities.symbols,
            crate::authority::mutation::BranchLocalDeleteAllowance::default(),
            &mut self.mutation.identities.records,
            crate::authority::commit::pipeline::request_lease(execution),
            runtime.commit_work_budget.clone(),
            Some(&deferred_edges),
        )
        .map_err(RelationalCoupledAssemblyFailure::from_apply)?;
        let source = self
            .identity
            .source_interpretation()
            .expect("owner-issued writer identity");
        state.content = crate::authority::commit::phases::applied_mutation::content::prepare(
            execution,
            &runtime,
            root.as_ref(),
            &self.prepared.schema_authority,
            content,
            &state,
            &outcome.effect.publication.changed_records,
            self.identity.proposed_version_id(),
            source.restoration_content(),
        )
        .map_err(RelationalCoupledAssemblyFailure::from_apply)?;
        crate::authority::mutation::apply_adjacency_deltas(
            &mut state,
            &outcome.effect.adjacency.deltas,
            self.identity.proposed_version_id(),
        );
        let part = CoupledPublicationPart {
            input: read,
            publication,
            commands: plan.clone(),
            records: outcome.effect.publication.changed_records.clone(),
        };
        self.mutation
            .outcome
            .created_entities
            .append_owned(outcome.created_entities)
            .map_err(TransactionCommitError::conflict)?;
        self.mutation
            .outcome
            .created_relations
            .append_owned(outcome.created_relations)
            .map_err(TransactionCommitError::conflict)?;
        self.mutation
            .outcome
            .preparation_telemetry
            .accumulate(outcome.preparation_telemetry);
        self.mutation.outcome.effect.accumulate(outcome.effect);
        self.mutation.state.adopt_sequential_apply(state);
        self.mutation.aggregate = crate::aggregate_identity::AggregateProposal::capture(
            &self.prepared.schema_authority,
            &self.prepared.selected_branch_state,
            self.identity,
            self.mutation.outcome.effect.clone(),
            None,
        );
        self.parts.push(part);
        Ok(())
    }
}
