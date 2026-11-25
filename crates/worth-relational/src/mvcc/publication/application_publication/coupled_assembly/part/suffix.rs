//! Apply one writer's suffix over the current candidate pin. Producers and
//! authored stages share this one apply owner; only their custody differs.

use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::assembly_failure::RelationalCoupledAssemblyFailureOwnerPort as _;
use worth_relational_publication::internal_port::visibility::materialization::read_records::reader::pinned_read::RelationalPinnedReadOwnerPort as _;

use super::super::suffix_scope::{self, AppliedSuffix, SuffixWriter};
use super::*;
use crate::storage::overlay::WorkingState;
use worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentityOwnerPort as _;

impl RelationalAuthoredCoupledCandidate<'_> {
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) fn apply_suffix(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        writer: SuffixWriter<'_>,
        content: &crate::content::RelationalContentAttachments,
        mut batches: Vec<WorkerIntentBatch>,
        mut naming: Option<&mut crate::lineage::data::CommitNamingPlan>,
    ) -> Result<AppliedSuffix, RelationalCoupledAssemblyFailure> {
        let successor = matches!(writer, SuffixWriter::Successor { .. });
        let (creator, source, custody, local_keys) = suffix_scope::scope(
            execution,
            writer,
            self.applied_parts(),
            &self.mutation.outcome.created_entities,
            self.mutation.candidate_symbols().interner(),
            &mut batches,
            naming.as_deref_mut(),
        )?;
        self.transaction
            .attach_content(execution, content)
            .map_err(crate::content::publication::commit_denial)?;
        for batch in &mut batches {
            super::super::fresh_references::retain(
                execution,
                &self.mutation.outcome.created_entities,
                &mut batch.intents,
            )?;
        }
        if let Some(plan) = naming {
            super::super::fresh_references::retain_naming(
                execution,
                &self.mutation.outcome.created_entities,
                self.mutation.candidate_symbols().interner(),
                plan,
            )?;
        }
        // Reserve and normalize only this incoming suffix, before it enters the
        // overlay's index and footprint. The final C validator still sees all
        // commands once; no prefix conflict or intent validation is relaxed.
        self.transaction
            .reserve_coupled_batch_keys(
                execution,
                &mut self.mutation.identities.symbols,
                &batches,
                self.runtime.config.identity.client_key_symbol_policy,
                execution.allocation_policy(),
            )
            .map_err(RelationalCoupledAssemblyFailure::from_apply)?;
        let runtime = self
            .runtime
            .with_candidate_symbols(self.mutation.candidate_symbols());
        let intents = self
            .transaction
            .push_normalized_coupled_batches(execution, &runtime, batches)?;
        // Preserve master's original relation correspondence before prefix-created
        // endpoints are lowered, for every writer's suffix.
        let (intents, originals) =
            suffix_scope::normalize(execution, intents, &self.mutation.outcome.created_entities)?;
        let read = if successor {
            self.retracted_pin(execution)?
        } else {
            self.pin_source(execution)?
        };
        let root = read.resident_basis().root();
        let plan = self.transaction.build_merged_plan_for_state(
            &runtime,
            root.as_ref(),
            intents,
            execution,
        )?;
        self.fresh_records
            .require(execution, creator, source, plan.plan())?;
        let deferred_edges = super::super::writer_custody::require(
            execution,
            &self.binding.owners,
            custody,
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
        let mut outcome = crate::authority::mutation::apply_plan_with_publication_edges(
            execution,
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
        outcome.created_relations = originals.complete(
            execution,
            outcome.created_relations,
            &self.mutation.outcome.created_entities,
            self.mutation.candidate_symbols().interner(),
        )?;
        let interpretation = self
            .identity
            .source_interpretation()
            .expect("owner-issued writer identity");
        let content = crate::authority::commit::phases::applied_mutation::content::prepare(
            execution,
            &runtime,
            root,
            &self.prepared.schema_authority,
            content,
            &state,
            &outcome.effect.publication.changed_records,
            self.identity.proposed_version_id(),
            interpretation.restoration_content(),
        )
        .map_err(RelationalCoupledAssemblyFailure::from_apply)?;
        crate::authority::mutation::apply_adjacency_deltas_for_request(
            execution,
            &mut state,
            &outcome.effect.adjacency.deltas,
            self.identity.proposed_version_id(),
        )?;
        self.fresh_records
            .retain(execution, creator, source, &outcome)?;
        let has_entity_deletions = outcome.effect.publication.has_entity_deletions;
        let (records, creates, delta_state) = suffix_scope::adopt(
            execution,
            &mut self.mutation.outcome,
            &mut self.mutation.state,
            outcome,
            state,
            &self.mutation.identities,
        )?;
        self.mutation.aggregate = crate::aggregate_identity::AggregateProposal::capture(
            &self.prepared.schema_authority,
            &self.prepared.selected_branch_state,
            self.identity,
            &self.mutation.outcome.effect.publication.changed_records,
            &self.mutation.outcome.effect.adjacency.deltas,
            self.mutation
                .outcome
                .effect
                .publication
                .has_entity_deletions,
            None,
        );
        Ok(AppliedSuffix {
            read,
            plan,
            records,
            delta_state,
            content,
            local_keys,
            creates,
            has_entity_deletions,
        })
    }

    /// Replace the candidate pin with the successor that includes the part
    /// just pushed, numbered after it in the one applied-part sequence.
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) fn advance_pin(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        delta_state: &WorkingState,
        content: &crate::content::publication::ContentPublication,
        records: &[RecordRef],
        input: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        let successor = RelationalPinnedRead::capture_applied(
            execution,
            self.runtime,
            self.binding.owners.before(),
            self.identity,
            self.applied_parts() as u64,
            delta_state,
            content,
            self.mutation.candidate_symbols(),
            records,
            self.aggregate_read(execution)?,
            Some(input),
        )?;
        self.read.replace(Some(successor));
        Ok(())
    }
}
