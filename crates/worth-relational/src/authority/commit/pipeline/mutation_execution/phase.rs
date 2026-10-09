use super::super::rejection::{attach_rejection, elapsed_micros};
use crate::authority::commit::phases::mutation::branch_local_delete_allowance_for_plan;
use crate::authority::mutation::{apply_plan_to_working_state, MutationApplyOutcome};
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::{
    AuthoritativeApplyPlan, CommitLog, CommitPhase, CommitPhaseTiming, MergedCommitPlan,
    TransactionCommitError, TransactionId,
};

pub(super) struct MutationPhaseOutput {
    version_id: crate::identity::data::VersionId,
    effect: crate::authority::mutation::MutationEffect,
    invariant_results: crate::validation::engine::InvariantExecutionResult,
    created_entities: crate::transactions::data::CommitCreatedEntityBindings,
    created_relations: crate::transactions::data::CommitCreatedRelationBindings,
    record_allocations: crate::runtime::PendingRecordAllocations,
}

impl MutationPhaseOutput {
    pub(super) fn invariant_results(&self) -> &crate::validation::engine::InvariantExecutionResult {
        &self.invariant_results
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        crate::identity::data::VersionId,
        crate::authority::mutation::MutationEffect,
        crate::validation::engine::InvariantExecutionResult,
        crate::transactions::data::CommitCreatedEntityBindings,
        crate::transactions::data::CommitCreatedRelationBindings,
        crate::runtime::PendingRecordAllocations,
    ) {
        (
            self.version_id,
            self.effect,
            self.invariant_results,
            self.created_entities,
            self.created_relations,
            self.record_allocations,
        )
    }
}

pub(super) struct MutationPhaseInput<'a, 'lease> {
    pub(super) commit_log: &'a mut CommitLog,
    pub(super) phase_timing: &'a mut CommitPhaseTiming,
    pub(super) transaction_id: TransactionId,
    pub(super) working_state: &'a mut crate::storage::overlay::WorkingState,
    pub(super) merged_plan: &'a MergedCommitPlan,
    pub(super) schema_authority: &'a crate::branch::RelationalBranchRootSchemaAuthority,
    pub(super) selected_branch_state: &'a crate::branch::SelectedRelationalBranchState,
    pub(super) proposed_version_id: crate::identity::data::VersionId,
    pub(super) proposal_identity: &'a crate::mvcc::RelationalMutationProposalIdentity,
    pub(super) prevalidated_mutation_sensitive:
        Option<crate::validation::engine::InvariantExecutionResult>,
    pub(super) lease: Option<&'a worth_execution::ExecutionResourceLease<'lease>>,
}

pub(super) fn run_authoritative_mutation_phase(
    runtime: &RelationalPreparationRuntime,
    input: MutationPhaseInput<'_, '_>,
    control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
) -> Result<MutationPhaseOutput, TransactionCommitError> {
    let MutationPhaseInput {
        commit_log,
        phase_timing,
        transaction_id,
        working_state,
        merged_plan,
        schema_authority,
        selected_branch_state,
        proposed_version_id,
        proposal_identity,
        prevalidated_mutation_sensitive,
        lease,
    } = input;
    commit_log.begin_phase(CommitPhase::AuthoritativeMutation);
    let phase_started = std::time::Instant::now();
    let mutation = run_authoritative_mutation_for_runtime(
        runtime,
        transaction_id,
        working_state,
        merged_plan,
        schema_authority,
        selected_branch_state,
        proposed_version_id,
        proposal_identity,
        prevalidated_mutation_sensitive,
        lease,
        control,
    )
    .map_err(|error| attach_rejection(commit_log, CommitPhase::AuthoritativeMutation, error))?;
    commit_log.record_invariant_outcomes(mutation.invariant_results());
    commit_log.complete_phase(CommitPhase::AuthoritativeMutation);
    phase_timing.authoritative_mutation_micros = elapsed_micros(phase_started);
    Ok(mutation)
}

fn run_authoritative_mutation_for_runtime(
    runtime: &RelationalPreparationRuntime,
    transaction_id: TransactionId,
    working_state: &mut crate::runtime::WorkingState,
    merged_plan: &MergedCommitPlan,
    schema_authority: &crate::branch::RelationalBranchRootSchemaAuthority,
    selected_branch_state: &crate::branch::SelectedRelationalBranchState,
    proposed_version_id: crate::identity::data::VersionId,
    proposal_identity: &crate::mvcc::RelationalMutationProposalIdentity,
    prevalidated_mutation_sensitive: Option<crate::validation::engine::InvariantExecutionResult>,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
) -> Result<MutationPhaseOutput, TransactionCommitError> {
    let version_id = proposed_version_id;
    let apply_plan = AuthoritativeApplyPlan {
        transaction_id,
        version_id,
        merged_intents: merged_plan.merged_intents.clone(),
    };
    let mutation_config = crate::config::data::MutationConfig {
        cascade_delete_policy: runtime.config.storage.cascade_delete_policy,
        adjacency_policy: runtime.config.storage.adjacency_policy.clone(),
        cross_context_policy: runtime.config.storage.cross_context_policy,
    };
    let branch_local_delete_allowance =
        branch_local_delete_allowance_for_plan(selected_branch_state, working_state, merged_plan);
    let mut record_allocations = runtime.record_identity.begin_allocations();
    let schema_registry = schema_authority.registry();
    let aspect_plans = schema_authority.aspect_plans();
    let MutationApplyOutcome {
        effect,
        preparation_telemetry,
        created_entities,
        created_relations,
    } = runtime.services.symbols.with_write(|symbols| {
        apply_plan_to_working_state(
            working_state,
            &apply_plan,
            &mutation_config,
            schema_registry,
            aspect_plans,
            symbols,
            branch_local_delete_allowance,
            &mut record_allocations,
            lease,
            runtime.commit_work_budget.clone(),
        )
    })?;
    record_preparation_telemetry(runtime, preparation_telemetry);
    crate::authority::commit::phases::prepare::record_mutation_counters(runtime, working_state);

    let invariant_results = match prevalidated_mutation_sensitive {
        Some(result) => result,
        None => runtime
            .invariant_authority()
            .enforce_mutation_sensitive_for_working_state(
                selected_branch_state,
                working_state,
                version_id,
                merged_plan,
                Some(proposal_identity),
                lease,
                control,
            )?,
    };

    Ok(MutationPhaseOutput {
        version_id,
        effect,
        invariant_results,
        created_entities,
        created_relations,
        record_allocations,
    })
}

fn record_preparation_telemetry(
    runtime: &RelationalPreparationRuntime,
    telemetry: crate::authority::mutation::MutationPreparationTelemetry,
) {
    runtime.performance_access().count_preparation_packet_shape(
        telemetry.packet_count,
        telemetry.packet_item_count,
        telemetry.packet_peak_width_total,
        telemetry.scope_unit_count,
    );
    for _ in 0..telemetry.parallel_legal_count {
        runtime
            .performance_access()
            .count_preparation_parallel_legal();
    }
    for _ in 0..telemetry.parallel_profitable_count {
        runtime
            .performance_access()
            .count_preparation_parallel_profitable();
    }
    for _ in 0..telemetry.serial_strategy_count {
        runtime
            .performance_access()
            .count_preparation_serial_strategy();
    }
    for _ in 0..telemetry.staged_parallel_strategy_count {
        runtime
            .performance_access()
            .count_preparation_staged_parallel_strategy();
    }
}
