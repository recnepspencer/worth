use std::sync::Arc;

use worth_query_installation::facade::WorthQueryInstalledBoundedStepContract;

use crate::domain_computation::provider_session::graph_provider::bounded_step::{
    provider_anchor::WorthQueryGraphProviderAnchor, WorthQueryGraphProviderMemoryArena,
    WorthQueryGraphProviderMemorySnapshot, WorthQueryGraphProviderStepArtifactContext,
    WorthQueryRetainedGraphProviderCheckpoint,
};
use crate::domain_computation::{
    WorthQueryGraphProviderCall, WorthQueryGraphReadStreamAccumulator,
    WorthQueryProviderCheckpointEvidence, WorthQueryProviderCheckpointReleaseEvidence,
    WorthQueryProviderCheckpointRetentionFailure,
};

pub(super) struct WorthQueryRetainedManagedGraphExecution {
    pub(super) call: WorthQueryGraphProviderCall,
    pub(super) checkpoint: WorthQueryRetainedGraphProviderCheckpoint,
    pub(super) contract: WorthQueryInstalledBoundedStepContract,
    pub(super) memory: WorthQueryGraphProviderMemoryArena,
    pub(super) completed_work_units: u64,
    pub(super) applied_effect_count: u64,
    pub(super) peak_scratch_bytes: u64,
    pub(super) retained_bytes: u64,
    pub(super) projection: Option<WorthQueryGraphReadStreamAccumulator>,
    pub(super) artifact_context: Option<WorthQueryGraphProviderStepArtifactContext>,
    pub(super) produced_artifact_count: usize,
    pub(super) retained_artifact_count: usize,
    pub(super) disposed_artifact_count: usize,
}

pub(super) struct WorthQueryRetainedManagedGraphExecutionParts {
    pub(super) call: WorthQueryGraphProviderCall,
    pub(super) anchor: Arc<WorthQueryGraphProviderAnchor>,
    pub(super) contract: WorthQueryInstalledBoundedStepContract,
    pub(super) memory: WorthQueryGraphProviderMemoryArena,
    pub(super) completed_work_units: u64,
    pub(super) applied_effect_count: u64,
    pub(super) peak_scratch_bytes: u64,
    pub(super) retained_bytes: u64,
    pub(super) projection: Option<WorthQueryGraphReadStreamAccumulator>,
    pub(super) artifact_context: Option<WorthQueryGraphProviderStepArtifactContext>,
    pub(super) produced_artifact_count: usize,
    pub(super) retained_artifact_count: usize,
    pub(super) disposed_artifact_count: usize,
}

impl WorthQueryRetainedManagedGraphExecution {
    pub(super) fn new(
        parts: WorthQueryRetainedManagedGraphExecutionParts,
        checkpoint: Box<dyn crate::domain_computation::WorthQueryGraphProviderCheckpoint>,
    ) -> Result<Self, WorthQueryProviderCheckpointRetentionFailure> {
        let checkpoint = WorthQueryRetainedGraphProviderCheckpoint::retain(
            parts.anchor,
            &parts.call,
            checkpoint,
        )?;
        debug_assert!(checkpoint.provider_generation_matches_anchor());
        Ok(Self {
            call: parts.call,
            checkpoint,
            contract: parts.contract,
            memory: parts.memory,
            completed_work_units: parts.completed_work_units,
            applied_effect_count: parts.applied_effect_count,
            peak_scratch_bytes: parts.peak_scratch_bytes,
            retained_bytes: parts.retained_bytes,
            projection: parts.projection,
            artifact_context: parts.artifact_context,
            produced_artifact_count: parts.produced_artifact_count,
            retained_artifact_count: parts.retained_artifact_count,
            disposed_artifact_count: parts.disposed_artifact_count,
        })
    }

    pub(super) fn checkpoint_evidence(&self) -> &WorthQueryProviderCheckpointEvidence {
        self.checkpoint.evidence()
    }

    pub(super) fn provider_memory_snapshot(&self) -> WorthQueryGraphProviderMemorySnapshot {
        self.memory.snapshot()
    }

    pub(super) fn contract(&self) -> &WorthQueryInstalledBoundedStepContract {
        &self.contract
    }

    pub(super) fn provider_generation_matches_anchor(&self) -> bool {
        self.checkpoint.provider_generation_matches_anchor()
    }

    pub(super) fn release(self) -> WorthQueryProviderCheckpointReleaseEvidence {
        let Self {
            call,
            checkpoint,
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
        } = self;
        let checkpoint_release = checkpoint.release();
        drop((
            call,
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
        ));
        checkpoint_release
    }
}
