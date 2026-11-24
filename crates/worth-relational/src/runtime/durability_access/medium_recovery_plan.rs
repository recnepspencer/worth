use crate::capabilities::RuntimeConfigSource;
use crate::durability::data::{RecoveryPlan, RecoveryVerificationMode, RelationalRecoveryStop};
use crate::durability::published_medium::RelationalDurableRecoverySource;
use crate::runtime::durability_access::{
    authority_continuity_for_envelopes, descriptor_semantics_version_for_envelopes,
};
use crate::runtime::RelationalRuntime;

pub(super) fn medium_recovery_plan(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: RelationalDurableRecoverySource,
    verification_mode: RecoveryVerificationMode,
) -> Result<RecoveryPlan, RelationalRecoveryStop> {
    let crate::durability::medium_recovery_readmission::ReadmittedMediumRecovery {
        checkpoint,
        tail,
        branch_transitions,
        original_source,
        cold_source,
        source_memory: _source_memory,
    } = crate::durability::medium_recovery_readmission::readmit_medium_recovery(execution, source)?;
    let checkpoint_envelopes = checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.envelopes.as_slice())
        .unwrap_or(&[]);
    let descriptor_semantics_version =
        descriptor_semantics_version_for_envelopes(checkpoint_envelopes, &tail);
    let mut continuity = authority_continuity_for_envelopes(runtime, checkpoint_envelopes, &tail);
    if let Some(checkpoint) = &checkpoint {
        super::native_checkpoint_recovery_plan::bind_source_runtime_identity(
            runtime,
            checkpoint,
            &mut continuity,
        );
    }
    let restore_authoritative_envelope_commit_ids = tail
        .iter()
        .map(|entry| entry.envelope().commit.commit_id)
        .collect();
    let mut plan = RecoveryPlan::new(
        runtime.runtime_config().clone(),
        checkpoint,
        tail,
        branch_transitions,
        continuity,
        verification_mode,
        descriptor_semantics_version,
        restore_authoritative_envelope_commit_ids,
        Some(cold_source),
    )
    .with_commit_strategy_executors(runtime.commit_strategy_executor_registry().clone());
    plan.original_source = original_source;
    Ok(plan)
}
