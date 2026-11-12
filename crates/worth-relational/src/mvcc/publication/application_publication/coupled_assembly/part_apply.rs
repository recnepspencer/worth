//! Complete original correspondence before a coupled apply can join assembly.
use super::*;
use crate::authority::commit::phases::prepare::prepare_authoritative_working_state_scope_for_base;
use crate::transactions::data::{AuthoritativeApplyPlan, CoupledOriginalRelationCreates};

pub(super) struct CompletedCoupledApply {
    pub(super) commands: crate::mvcc::OwnedNormalizedPlan,
    pub(super) state: crate::storage::overlay::WorkingState,
    pub(super) outcome: crate::authority::mutation::MutationApplyOutcome,
}

struct PendingCoupledApply {
    commands: crate::mvcc::OwnedNormalizedPlan,
    originals: CoupledOriginalRelationCreates,
}

pub(super) fn apply(
    candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    execution: &ExecutionRequest<'_, '_>,
    before: usize,
    predecessor_origin: &crate::history::data::CanonicalRequiredOutputOrigin,
    content: &crate::content::RelationalContentAttachments,
) -> Result<CompletedCoupledApply, RelationalCoupledAssemblyFailure> {
    let runtime = candidate
        .runtime
        .with_candidate_symbols(candidate.mutation.candidate_symbols());
    let intents = candidate
        .transaction
        .normalized_intents_for_merge(&runtime, execution)?;
    let originals = CoupledOriginalRelationCreates::capture(
        execution,
        intents.plan().merged_intents.get(before..).ok_or_else(|| {
            crate::mvcc::origin_denial("coupled suffix starts beyond its normalized source")
        })?,
    )?;
    let intents = intents.into_coupled_suffix(
        before,
        &candidate.mutation.outcome.created_entities,
        execution,
    )?;
    let read = candidate.pin_source(execution)?;
    let root = read.resident_basis().root();
    let plan = candidate.transaction.build_merged_plan_for_state(
        &runtime,
        root.as_ref(),
        intents,
        execution,
    )?;
    PendingCoupledApply {
        commands: plan,
        originals,
    }
    .apply(
        candidate,
        execution,
        &runtime,
        root.as_ref(),
        predecessor_origin,
        content,
    )
}

impl PendingCoupledApply {
    fn apply(
        self,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalPreparationRuntime,
        root: &crate::branch::RelationalBranchRoot,
        predecessor_origin: &crate::history::data::CanonicalRequiredOutputOrigin,
        content: &crate::content::RelationalContentAttachments,
    ) -> Result<CompletedCoupledApply, RelationalCoupledAssemblyFailure> {
        let deferred_edges = super::writer_custody::require(
            execution,
            &candidate.binding.owners,
            Some(predecessor_origin),
            self.commands.plan(),
        )?;
        let (_, mut state, _) = prepare_authoritative_working_state_scope_for_base(
            runtime,
            root,
            self.commands.plan(),
            0,
            None,
        );
        crate::aggregate_identity::guard::ordinary_plan(
            execution,
            runtime.commit_work_budget.as_ref(),
            candidate.prepared.schema_authority.registry(),
            &crate::validation::engine::state_view::InvariantStateView::new(
                root,
                candidate.identity.proposed_version_id(),
            ),
            &self.commands.plan().merged_intents,
            None,
        )?;
        let config = crate::config::data::MutationConfig {
            cascade_delete_policy: runtime.config.storage.cascade_delete_policy,
            adjacency_policy: runtime.config.storage.adjacency_policy.clone(),
            cross_context_policy: runtime.config.storage.cross_context_policy,
        };
        let _apply_memory = crate::mvcc::plan_copy_admission::reserve_copy(
            execution,
            crate::mvcc::plan_copy_admission::intent_bytes(&self.commands.plan().merged_intents),
        )?;
        let apply = AuthoritativeApplyPlan {
            transaction_id: self.commands.plan().transaction_id,
            version_id: candidate.identity.proposed_version_id(),
            merged_intents: self.commands.plan().merged_intents.clone(),
        };
        let mut outcome = crate::authority::mutation::apply_plan_with_publication_edges(
            execution,
            &mut state,
            &apply,
            &config,
            candidate.prepared.schema_authority.registry(),
            candidate.prepared.schema_authority.aspect_plans(),
            &mut candidate.mutation.identities.symbols,
            crate::authority::mutation::BranchLocalDeleteAllowance::default(),
            &mut candidate.mutation.identities.records,
            crate::authority::commit::pipeline::request_lease(execution),
            runtime.commit_work_budget.clone(),
            Some(&deferred_edges),
        )
        .map_err(RelationalCoupledAssemblyFailure::from_apply)?;
        outcome.created_relations = self.originals.complete(
            execution,
            outcome.created_relations,
            &candidate.mutation.outcome.created_entities,
            candidate.mutation.candidate_symbols().interner(),
        )?;
        let source = candidate
            .identity
            .source_interpretation()
            .expect("owner-issued writer identity");
        state.content = crate::authority::commit::phases::applied_mutation::content::prepare(
            execution,
            runtime,
            root,
            &candidate.prepared.schema_authority,
            content,
            &state,
            &outcome.effect.publication.changed_records,
            candidate.identity.proposed_version_id(),
            source.restoration_content(),
        )
        .map_err(RelationalCoupledAssemblyFailure::from_apply)?;
        crate::authority::mutation::apply_adjacency_deltas_for_request(
            execution,
            &mut state,
            &outcome.effect.adjacency.deltas,
            candidate.identity.proposed_version_id(),
        )?;
        Ok(CompletedCoupledApply {
            commands: self.commands,
            state,
            outcome,
        })
    }
}
