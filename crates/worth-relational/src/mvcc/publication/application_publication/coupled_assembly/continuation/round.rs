//! Judge one continued round by what it would actually apply, before it can publish.
use super::super::{
    custody, writer_custody, RelationalCoupledAssemblyFailure, RequiredOutputPublication,
};
use super::{
    checkpoint, part, Cause, ExecutionRequest, RelationalAuthoredCoupledCandidate,
    RelationalCoupledProducerContinuation, TransactionCommitError,
};
use crate::authority::commit::phases::prepare::prepare_authoritative_working_state_scope_for_base;
use crate::publication::patch::data::preserves_application;
use crate::transactions::data::{
    AuthoritativeApplyPlan, EntityMutationIntent, MutationIntent, RelationMutationIntent,
    WorkerIntentBatch,
};
use std::sync::Arc;

/// Publishing a second producing round needs multi-input provenance (public #119).
const PROVENANCE_ISSUE: u16 = 119;

/// Only these commands can leave a record structurally as it was. Creation,
/// deletion, replacement and endpoint changes are effects by their application.
pub(super) fn patches_in_place(intent: &MutationIntent) -> bool {
    matches!(
        intent,
        MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(_) | EntityMutationIntent::ApplyAspectPatch(_)
        ) | MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(_))
    )
}

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
            return Err(part::reject(
                Cause::CoupledContinuationProvenancePostponed {
                    issue: PROVENANCE_ISSUE,
                },
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
        let allocation = execution.allocation_policy();
        let mut intents = Vec::new();
        let in_place = execution
            .run_native_with_checkpoint(|context| {
                let mut raw_keys = std::collections::BTreeSet::new();
                for intent in batches.iter().flat_map(|batch| &batch.intents) {
                    checkpoint(
                        context,
                        intent.owned_allocation_capacity_bytes().saturating_add(1),
                    )
                    .map_err(custody::execution_stop)?;
                    intent
                        .collect_raw_client_keys(&mut raw_keys, allocation)
                        .map_err(|cause| {
                            TransactionCommitError::conflict(
                                crate::mvcc::transaction::RelationalTransactionStagingDenial::from(
                                    cause,
                                )
                                .into_conflict(),
                            )
                        })?;
                    if !patches_in_place(intent) || !raw_keys.is_empty() {
                        return Ok(false);
                    }
                    intents.try_reserve(1).map_err(|_| {
                        custody::execution_stop(
                            crate::execution::RelationalExecutionDenialCause::AllocationFailed,
                        )
                    })?;
                    intents.push(intent.clone());
                }
                Ok::<_, TransactionCommitError>(true)
            })
            .map_err(custody::execution_stop)?
            .0?;
        if !in_place {
            return Ok(false);
        }
        if intents.is_empty() {
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
        let plan = self
            .transaction
            .build_merged_plan_for_state(&runtime, root.as_ref(), intents)
            .map_err(TransactionCommitError::conflict)?;
        let deferred_edges = writer_custody::require(
            execution,
            &self.binding.owners,
            writer_custody::Writer::Producer(predecessor.as_deref()),
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
            transaction_id: plan.transaction_id,
            version_id: self.identity.proposed_version_id(),
            merged_intents: plan.merged_intents,
        };
        let outcome = crate::authority::mutation::apply_plan_with_publication_edges(
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
        if !outcome.effect.adjacency.deltas.is_empty()
            || outcome
                .created_entities
                .allocated_identities()
                .next()
                .is_some()
            || outcome.created_relations.iter().next().is_some()
            || symbols.reserved_meaning_count() != reserved
        {
            return Ok(false);
        }
        let publication = &outcome.effect.publication;
        Ok(execution
            .run_native_with_checkpoint(|context| {
                for fragment in &publication.patch_fragments {
                    checkpoint(context, 1).map_err(custody::execution_stop)?;
                    if !preserves_application(
                        fragment.structural_change,
                        fragment.contains_opaque_aspect,
                        &fragment.semantic_changes,
                        &fragment.published_patch,
                        fragment.relation_birth.as_ref(),
                    ) {
                        return Ok(false);
                    }
                }
                for record in &publication.changed_records {
                    checkpoint(context, publication.patch_fragments.len() as u64 + 1)
                        .map_err(custody::execution_stop)?;
                    if !publication
                        .patch_fragments
                        .iter()
                        .any(|fragment| &fragment.target == record)
                    {
                        return Ok(false);
                    }
                }
                Ok::<_, TransactionCommitError>(true)
            })
            .map_err(custody::execution_stop)?
            .0?)
    }
}
