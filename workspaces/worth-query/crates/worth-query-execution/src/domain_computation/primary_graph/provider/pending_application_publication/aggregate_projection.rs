use super::*;

pub(super) fn publish_aggregate_projection(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    pending: &mut WorthQueryPendingApplicationPublication,
    after: &worth_relational::facade::snapshots::SnapshotHandle,
) -> Result<(), WorthQueryProviderSessionFailure> {
    if pending.aggregate_published {
        return Ok(());
    }
    let mut aggregates = provider
        .graph
        .aggregate_projections
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(before) = pending.before.as_ref() {
        aggregates.refresh_after_commit(
            execution,
            runtime,
            before,
            after,
            pending.committed.patch(),
        ).map_err(|cause| WorthQueryProviderSessionFailure::checked_read_stopped(
            super::super::relational_execution_denial::relational_execution_kind(cause,None),
            crate::domain_computation::WorthQueryProviderSessionProtocolStage::Commit,
            "aggregate source read stopped after publication",
        ).with_recovery_posture(crate::domain_computation::WorthQueryProviderSessionRecoveryPosture::RecoveryRequired))?;
    } else {
        aggregates.recover_after_commit(after.version_id());
    }
    pending.aggregate_published = true;
    Ok(())
}
