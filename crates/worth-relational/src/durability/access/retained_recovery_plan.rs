use crate::capabilities::{DurabilityRead, RuntimeConfigSource};
use crate::durability::access::{
    authority_continuity_for_envelopes, descriptor_semantics_version_for_envelopes,
};
use crate::durability::data::{RecoveryPlan, RecoveryVerificationMode};
use crate::runtime::RelationalRuntime;

pub(super) fn retained_recovery_plan(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    verification_mode: RecoveryVerificationMode,
) -> Result<RecoveryPlan, crate::durability::data::RelationalRecoveryStop> {
    let checkpoint = runtime
        .durable_checkpoints()
        .last()
        .map(|checkpoint| checkpoint.as_ref().clone());
    let tail_log = tail_log_after_in_memory_checkpoint(runtime, execution, checkpoint.as_ref())?;
    let covered = checkpoint.as_ref().map_or(
        crate::branch::RelationalBranchTransitionSequence(0),
        |checkpoint| checkpoint.coverage.up_to_branch_transition,
    );
    let branch_transitions = crate::durability::data::RecoveryBranchTransitions::Contiguous(
        runtime
            .durability_access()
            .reconstruct_branch_transition_history(execution, covered)?,
    );
    let descriptor_semantics_version = descriptor_semantics_version_for_envelopes(
        checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.envelopes.as_slice())
            .unwrap_or(&[]),
        &tail_log,
    );
    let recovery_authority_continuity = authority_continuity_for_envelopes(
        runtime,
        checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.envelopes.as_slice())
            .unwrap_or(&[]),
        &tail_log,
    );
    let restore_authoritative_envelope_commit_ids = tail_log
        .iter()
        .map(|entry| entry.envelope().commit.commit_id)
        .collect();

    Ok(RecoveryPlan::new(
        runtime.runtime_config().clone(),
        checkpoint,
        tail_log,
        branch_transitions,
        recovery_authority_continuity,
        verification_mode,
        descriptor_semantics_version,
        restore_authoritative_envelope_commit_ids,
        runtime.durability.cold_source(),
    )
    .with_commit_strategy_executors(runtime.commit_strategy_executor_registry().clone()))
}

fn tail_log_after_in_memory_checkpoint(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    checkpoint: Option<&crate::durability::data::DurableCheckpoint>,
) -> Result<
    Vec<crate::durability::migration::ReadmittedCanonicalCommit>,
    crate::durability::data::RelationalRecoveryStop,
> {
    let after = checkpoint.and_then(|checkpoint| {
        checkpoint
            .envelopes
            .iter()
            .map(crate::history::data::PositionedCanonicalCommit::position)
            .max()
    });
    Ok(runtime
        .durability
        .demand_log_after(execution, after)?
        .into_iter()
        .map(crate::durability::migration::ReadmittedCanonicalCommit::exact)
        .collect())
}
