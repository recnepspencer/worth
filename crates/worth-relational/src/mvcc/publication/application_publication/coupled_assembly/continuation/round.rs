//! Judge one continued round by what it would actually apply, before it can publish.
use super::super::{writer_custody, RelationalCoupledAssemblyFailure, RequiredOutputPublication};
use super::{
    application, postponed, ExecutionRequest, RelationalAuthoredCoupledCandidate,
    RelationalCoupledProducerContinuation,
};
use crate::authority::commit::phases::prepare::prepare_authoritative_working_state_scope_for_base;
use crate::transactions::data::outcomes::plan_artifacts::AuthoritativeApplyPlan;
use crate::transactions::data::primitives::WorkerIntentBatch;
use std::sync::Arc;

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Consume next-round custody and judge the continued round's body on a
    /// discarded image of D. An unchanged application leaves this candidate
    /// exactly as it was. Any effect refuses before publication: it would be
    /// this route's second producing round under one single-input origin.
    pub fn verify_producer_round(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        token: RelationalCoupledProducerContinuation,
        content: &crate::content::RelationalContentAttachments,
        batches: Vec<WorkerIntentBatch>,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        let index = self
            .continuation
            .consume(token, self.identity, self.applied_parts())?;
        let source = self.parts[index]
            .publication
            .producing_meaning()
            .expect("admitted producer route")
            .source_scope;
        if !self.round_preserves_application(execution, index, content, batches)? {
            return Err(postponed(
                crate::publication::data::CoupledProvenancePostponement::ContinuedRoundEffect,
                source,
            )
            .into());
        }
        Ok(())
    }

    /// O(round commands + records they touch). Nothing here writes this
    /// candidate: commands apply to a fresh working state over the current
    /// pin and reserve symbols only in a discarded copy of its dictionary.
    fn round_preserves_application(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        index: usize,
        content: &crate::content::RelationalContentAttachments,
        batches: Vec<WorkerIntentBatch>,
    ) -> Result<bool, RelationalCoupledAssemblyFailure> {
        if !content.is_empty() {
            return Ok(false);
        }
        if !application::commands_are_in_place(execution, &batches)? {
            return Ok(false);
        }
        if batches.iter().all(|batch| batch.intents.is_empty()) {
            return Ok(true);
        }
        let predecessor = match self.parts[index].publication.owned() {
            Some(RequiredOutputPublication::Preserve { predecessor, .. }) => {
                Some(Arc::clone(&predecessor.origin))
            }
            _ => None,
        };
        let read = self.pin_source(execution)?;
        let root = read.resident_basis().root();
        let runtime = self
            .runtime
            .with_candidate_symbols(self.mutation.candidate_symbols());
        let intents = crate::mvcc::transaction::normalized_plan::OwnedNormalizedPlan::copy_batches(
            self.transaction.transaction_id(),
            &batches,
            execution,
        )?;
        let plan = self.transaction.build_merged_plan_for_state(
            &runtime,
            root.as_ref(),
            intents,
            execution,
        )?;
        let deferred_edges = writer_custody::require(
            execution,
            &self.binding.owners,
            writer_custody::Writer::Producer(predecessor.as_deref()),
            plan.plan(),
        )?;
        let (_, mut state, _) = prepare_authoritative_working_state_scope_for_base(
            &runtime,
            root,
            plan.plan(),
            0,
            None,
        );
        crate::aggregate_identity::guard::ordinary_plan(
            execution,
            runtime.commit_work_budget.as_ref(),
            self.prepared.schema_authority.registry(),
            &crate::validation::engine::state_view::InvariantStateView::new(
                root,
                self.identity.proposed_version_id(),
            ),
            &plan.plan().merged_intents,
            None,
        )?;
        let mut symbols = self
            .mutation
            .candidate_symbols()
            .discardable_copy()
            .map_err(|cause| RelationalCoupledAssemblyFailure::from_apply(cause.into()))?;
        let reserved = symbols.reserved_meaning_count();
        let config = crate::config::data::MutationConfig {
            cascade_delete_policy: runtime.config.storage.cascade_delete_policy,
            adjacency_policy: runtime.config.storage.adjacency_policy.clone(),
            cross_context_policy: runtime.config.storage.cross_context_policy,
        };
        let apply = AuthoritativeApplyPlan {
            transaction_id: plan.plan().transaction_id,
            version_id: self.identity.proposed_version_id(),
            merged_intents: plan.plan().merged_intents.clone(),
        };
        let outcome = crate::authority::mutation::apply_plan_with_publication_edges(
            execution,
            &mut state,
            &apply,
            &config,
            self.prepared.schema_authority.registry(),
            self.prepared.schema_authority.aspect_plans(),
            &mut symbols,
            crate::authority::mutation::BranchLocalDeleteAllowance::default(),
            &mut self.mutation.identities.records,
            crate::authority::commit::pipeline::request_lease(execution),
            runtime.commit_work_budget.clone(),
            Some(&deferred_edges),
        )
        .map_err(RelationalCoupledAssemblyFailure::from_apply)?;
        Ok(application::preserves_applied_records(
            execution,
            &outcome,
            reserved,
            symbols.reserved_meaning_count(),
        )?)
    }
}
