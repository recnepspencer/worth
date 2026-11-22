use super::{WorthQueryPausedWorkflowGraphExecution, WorthQueryWorkflowYieldDenialKind};

pub(super) fn classify_workflow_yield_denial(
    paused: &WorthQueryPausedWorkflowGraphExecution,
) -> Option<(WorthQueryWorkflowYieldDenialKind, &'static str)> {
    let running = &paused.active.running;
    if !running.installation_is_current() {
        return Some((
            WorthQueryWorkflowYieldDenialKind::InstallationGenerationStale,
            "the installed workflow generation changed after the run was admitted",
        ));
    }
    if !running.yield_is_installed() {
        return Some((
            WorthQueryWorkflowYieldDenialKind::YieldNotInstalled,
            "the admitted resource envelope does not install provider checkpoint yield",
        ));
    }
    if !paused.safe_point.checkpoint_available() {
        return Some((
            WorthQueryWorkflowYieldDenialKind::CheckpointUnavailable,
            "the consumed provider safe point reported no checkpoint availability",
        ));
    }
    None
}

pub(super) fn classify_workflow_retained_bytes_denial(
    provider_retained_bytes: u64,
    artifact_retained_bytes: usize,
    retained_bytes_ceiling: u64,
) -> Option<(WorthQueryWorkflowYieldDenialKind, &'static str)> {
    let retained_total = provider_retained_bytes
        .saturating_add(u64::try_from(artifact_retained_bytes).unwrap_or(u64::MAX));
    if retained_total > retained_bytes_ceiling {
        return Some((
            WorthQueryWorkflowYieldDenialKind::RetainedBytesExceeded,
            "provider and workflow artifacts exceed the installed retained-byte ceiling",
        ));
    }
    None
}
