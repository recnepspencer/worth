//! Apply one writer's suffix over the current candidate pin. Producers and
//! authored stages share this one apply owner; only their custody differs.
use super::*;
use crate::history::data::CanonicalRequiredOutputOrigin;
use crate::storage::overlay::WorkingState;

/// Who writes this suffix, and under which custody.
pub(in crate::mvcc::publication::application_publication::coupled_assembly) enum SuffixWriter<'a> {
    /// An admitted producer route, `index` in the producer roster.
    Producer {
        index: usize,
        source: crate::identity::data::EntityId,
        roles: &'a mut [RequiredOutputRoleTarget],
        predecessor: Option<&'a CanonicalRequiredOutputOrigin>,
    },
    /// The authored writer: one later stage editing an admitted stage source.
    Authored {
        source: crate::identity::data::EntityId,
        roles: &'a mut [RequiredOutputRoleTarget],
    },
}

pub(in crate::mvcc::publication::application_publication::coupled_assembly) struct AppliedSuffix {
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) read:
        RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) plan:
        crate::mvcc::OwnedNormalizedPlan,
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) records:
        Vec<RecordRef>,
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) delta_state:
        WorkingState,
}

impl RelationalAuthoredCoupledCandidate<'_> {
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) fn apply_suffix(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        writer: SuffixWriter<'_>,
        content: &crate::content::RelationalContentAttachments,
        mut batches: Vec<WorkerIntentBatch>,
    ) -> Result<AppliedSuffix, RelationalCoupledAssemblyFailure> {
        let (creator, source, custody) = match writer {
            SuffixWriter::Producer {
                index,
                source,
                roles,
                predecessor,
            } => {
                super::super::local_keys::scope(
                    execution,
                    self.applied_parts(),
                    &self.mutation.outcome.created_entities,
                    self.mutation.candidate_symbols().interner(),
                    &mut batches,
                    roles,
                )?;
                (
                    crate::publication::data::CoupledFreshWriter::Producer(index),
                    source,
                    super::super::writer_custody::Writer::Producer(predecessor),
                )
            }
            SuffixWriter::Authored { source, roles } => {
                super::super::local_keys::scope(
                    execution,
                    self.applied_parts(),
                    &self.mutation.outcome.created_entities,
                    self.mutation.candidate_symbols().interner(),
                    &mut batches,
                    roles,
                )?;
                (
                    crate::publication::data::CoupledFreshWriter::Authored,
                    source,
                    super::super::writer_custody::Writer::AuthoredPrefix,
                )
            }
        };
        self.transaction
            .attach_content(execution, content)
            .map_err(crate::content::commit_denial)?;
        for batch in &mut batches {
            super::super::fresh_references::retain(
                execution,
                &self.mutation.outcome.created_entities,
                &mut batch.intents,
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
            .push_normalized_coupled_batches(execution, &runtime, batches)?
            .into_coupled_suffix(0, &self.mutation.outcome.created_entities, execution)?;
        let read = self.pin_source(execution)?;
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
            root.as_ref(),
            plan.plan(),
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
        let outcome = crate::authority::mutation::apply_plan_with_publication_edges(
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
        let interpretation = self
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
        let records = outcome.effect.publication.changed_records.clone();
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
        // Retain only this part's actual sparse state for the next physical pin.
        // Applying it over the previous D reuses the ordinary root-region owner.
        let delta_state = state.clone();
        self.mutation
            .state
            .adopt_sequential_apply(state, execution)
            .map_err(custody::execution_stop)?;
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
        })
    }

    /// Replace the candidate pin with the successor that includes the part
    /// just pushed, numbered after it in the one applied-part sequence.
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) fn advance_pin(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        delta_state: &WorkingState,
        records: &[RecordRef],
        input: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        let successor = RelationalPinnedRead::capture_applied(
            execution,
            self.runtime,
            &self.binding.owners.before,
            self.identity,
            self.applied_parts() as u64,
            delta_state,
            self.mutation.candidate_symbols(),
            records,
            self.aggregate_read(execution)?,
            Some(input),
        )?;
        self.read.replace(Some(successor));
        Ok(())
    }
}
