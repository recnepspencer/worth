use std::sync::Arc;

use crate::domain_computation::WorthQueryGraphProviderCheckpoint;
use {
    crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryOwnedGraphProviderExecution,
    crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryProviderCheckpointReleaseEvidence,
    crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryProviderCheckpointRetentionFailure,
    crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryProviderCheckpointRetentionFailureKind,
    crate::domain_computation::provider_session::graph_provider::bounded_step::WorthQueryProviderExecutionInvocation,
    crate::domain_computation::vocabulary::provider_session::provider_execution_release::WorthQueryProviderExecutionReleaseEvidence,
};

use super::managed_graph_execution::WorthQueryManagedGraphExecution;
use super::retained_graph_execution::{
    WorthQueryRetainedManagedGraphExecution, WorthQueryRetainedManagedGraphExecutionParts,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderCheckpointSuspensionFailureKind {
    ProviderRejected,
    ProviderPanicked,
    ProviderExecutionReleaseRecoveryRequired,
    CheckpointMemoryMismatch,
    CheckpointRetention(WorthQueryProviderCheckpointRetentionFailureKind),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProviderCheckpointSuspensionFailureEvidence {
    kind: WorthQueryProviderCheckpointSuspensionFailureKind,
    detail: Arc<str>,
    provider_execution_release: WorthQueryProviderExecutionReleaseEvidence,
    checkpoint_retention_failure: Option<WorthQueryProviderCheckpointRetentionFailure>,
    checkpoint_release: Option<WorthQueryProviderCheckpointReleaseEvidence>,
    checkpoint_retained_byte_probe_count: usize,
}

impl WorthQueryProviderCheckpointSuspensionFailureEvidence {
    pub const fn kind(&self) -> WorthQueryProviderCheckpointSuspensionFailureKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub const fn provider_execution_release(&self) -> &WorthQueryProviderExecutionReleaseEvidence {
        &self.provider_execution_release
    }

    pub fn checkpoint_retention_failure(
        &self,
    ) -> Option<&WorthQueryProviderCheckpointRetentionFailure> {
        self.checkpoint_retention_failure.as_ref()
    }

    pub fn checkpoint_release(&self) -> Option<&WorthQueryProviderCheckpointReleaseEvidence> {
        self.checkpoint_release.as_ref()
    }

    pub const fn checkpoint_retained_byte_probe_count(&self) -> usize {
        self.checkpoint_retained_byte_probe_count
    }
}

pub(super) struct WorthQueryManagedGraphSuspension {
    pub(super) retained: WorthQueryRetainedManagedGraphExecution,
    pub(super) provider_execution_release: WorthQueryProviderExecutionReleaseEvidence,
}

impl WorthQueryManagedGraphExecution {
    pub(super) fn suspend(
        self,
    ) -> Result<
        WorthQueryManagedGraphSuspension,
        WorthQueryProviderCheckpointSuspensionFailureEvidence,
    > {
        let (mut execution, parts) = split_managed_execution(self);
        let suspension = execution.suspend();
        let provider_execution_release = execution.release();
        let checkpoint = classify_suspension_invocation(suspension, &provider_execution_release)?;
        retain_suspended_checkpoint(parts, checkpoint, provider_execution_release)
    }
}

fn split_managed_execution(
    execution: WorthQueryManagedGraphExecution,
) -> (
    WorthQueryOwnedGraphProviderExecution,
    WorthQueryRetainedManagedGraphExecutionParts,
) {
    let WorthQueryManagedGraphExecution {
        call,
        execution,
        anchor,
        contract,
        memory,
        completed_work_units,
        applied_effect_count,
        peak_scratch_bytes,
        retained_bytes,
        projection,
        artifact_context,
        produced_artifact_count,
        retained_artifact_count,
        disposed_artifact_count,
        ..
    } = execution;
    (
        execution,
        WorthQueryRetainedManagedGraphExecutionParts {
            call,
            anchor,
            contract: contract.into_installed(),
            memory,
            completed_work_units,
            applied_effect_count,
            peak_scratch_bytes,
            retained_bytes,
            projection,
            artifact_context,
            produced_artifact_count,
            retained_artifact_count,
            disposed_artifact_count,
        },
    )
}

fn classify_suspension_invocation(
    suspension: WorthQueryProviderExecutionInvocation<Box<dyn WorthQueryGraphProviderCheckpoint>>,
    provider_execution_release: &WorthQueryProviderExecutionReleaseEvidence,
) -> Result<
    Box<dyn WorthQueryGraphProviderCheckpoint>,
    WorthQueryProviderCheckpointSuspensionFailureEvidence,
> {
    match suspension {
        WorthQueryProviderExecutionInvocation::Returned(Ok(checkpoint)) => Ok(checkpoint),
        WorthQueryProviderExecutionInvocation::Returned(Err(failure)) => {
            Err(suspension_failure_without_checkpoint(
                WorthQueryProviderCheckpointSuspensionFailureKind::ProviderRejected,
                Arc::from(failure.detail()),
                provider_execution_release.clone(),
            ))
        }
        WorthQueryProviderExecutionInvocation::Panicked => {
            Err(suspension_failure_without_checkpoint(
                WorthQueryProviderCheckpointSuspensionFailureKind::ProviderPanicked,
                Arc::from("provider checkpoint suspension panicked"),
                provider_execution_release.clone(),
            ))
        }
    }
}

fn retain_suspended_checkpoint(
    parts: WorthQueryRetainedManagedGraphExecutionParts,
    checkpoint: Box<dyn WorthQueryGraphProviderCheckpoint>,
    provider_execution_release: WorthQueryProviderExecutionReleaseEvidence,
) -> Result<WorthQueryManagedGraphSuspension, WorthQueryProviderCheckpointSuspensionFailureEvidence>
{
    let retained = WorthQueryRetainedManagedGraphExecution::new(parts, checkpoint);
    match retained {
        Ok(retained)
            if retained.provider_memory_snapshot().retained_bytes()
                != retained.checkpoint_evidence().retained_bytes() =>
        {
            Err(checkpoint_memory_mismatch(
                retained.release(),
                provider_execution_release,
            ))
        }
        Ok(retained) if !provider_execution_release.recovery_required() => {
            Ok(WorthQueryManagedGraphSuspension {
                retained,
                provider_execution_release,
            })
        }
        Ok(retained) => Err(execution_release_failure(
            retained.release(),
            provider_execution_release,
        )),
        Err(checkpoint_retention_failure) => Err(checkpoint_retention_failure_evidence(
            checkpoint_retention_failure,
            provider_execution_release,
        )),
    }
}

fn checkpoint_memory_mismatch(
    checkpoint_release: WorthQueryProviderCheckpointReleaseEvidence,
    provider_execution_release: WorthQueryProviderExecutionReleaseEvidence,
) -> WorthQueryProviderCheckpointSuspensionFailureEvidence {
    WorthQueryProviderCheckpointSuspensionFailureEvidence {
        kind: WorthQueryProviderCheckpointSuspensionFailureKind::CheckpointMemoryMismatch,
        detail: Arc::from(
            "provider checkpoint retained bytes differ from the governed execution-memory arena",
        ),
        provider_execution_release,
        checkpoint_retention_failure: None,
        checkpoint_release: Some(checkpoint_release),
        checkpoint_retained_byte_probe_count: 1,
    }
}

fn execution_release_failure(
    checkpoint_release: WorthQueryProviderCheckpointReleaseEvidence,
    provider_execution_release: WorthQueryProviderExecutionReleaseEvidence,
) -> WorthQueryProviderCheckpointSuspensionFailureEvidence {
    WorthQueryProviderCheckpointSuspensionFailureEvidence {
        kind:
            WorthQueryProviderCheckpointSuspensionFailureKind::ProviderExecutionReleaseRecoveryRequired,
        detail: Arc::from(
            "provider execution required physical-release recovery after checkpoint suspension",
        ),
        provider_execution_release,
        checkpoint_retention_failure: None,
        checkpoint_release: Some(checkpoint_release),
        checkpoint_retained_byte_probe_count: 1,
    }
}

fn checkpoint_retention_failure_evidence(
    checkpoint_retention_failure: WorthQueryProviderCheckpointRetentionFailure,
    provider_execution_release: WorthQueryProviderExecutionReleaseEvidence,
) -> WorthQueryProviderCheckpointSuspensionFailureEvidence {
    let kind = if provider_execution_release.recovery_required() {
        WorthQueryProviderCheckpointSuspensionFailureKind::ProviderExecutionReleaseRecoveryRequired
    } else {
        WorthQueryProviderCheckpointSuspensionFailureKind::CheckpointRetention(
            checkpoint_retention_failure.kind(),
        )
    };
    WorthQueryProviderCheckpointSuspensionFailureEvidence {
        kind,
        detail: Arc::from("provider checkpoint retained-byte probe or physical release failed"),
        provider_execution_release,
        checkpoint_retention_failure: Some(checkpoint_retention_failure),
        checkpoint_release: None,
        checkpoint_retained_byte_probe_count: 1,
    }
}

fn suspension_failure_without_checkpoint(
    kind: WorthQueryProviderCheckpointSuspensionFailureKind,
    detail: Arc<str>,
    provider_execution_release: WorthQueryProviderExecutionReleaseEvidence,
) -> WorthQueryProviderCheckpointSuspensionFailureEvidence {
    WorthQueryProviderCheckpointSuspensionFailureEvidence {
        kind,
        detail,
        provider_execution_release,
        checkpoint_retention_failure: None,
        checkpoint_release: None,
        checkpoint_retained_byte_probe_count: 0,
    }
}
