use std::sync::Arc;

use super::{
    WorthQueryGraphProviderMemorySnapshot, WorthQueryGraphProviderStepArtifactEvidence,
    WorthQueryGraphProviderStepDispositionKind, WorthQueryGraphProviderStepFailureEvidence,
};
use crate::domain_computation::WorthQueryGraphReadMaterial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryGraphProviderStepCompletion {
    Continue,
    Complete,
    Failed,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryGraphProviderStepRetainedEvidence {
    provider_memory_arena_identity: u64,
    provider_allocation_count: u64,
    provider_bytes: u64,
    projection_bytes: u64,
    output_bytes: u64,
    artifact_bytes: u64,
    total_bytes: u64,
}

impl WorthQueryGraphProviderStepRetainedEvidence {
    pub(super) fn new(
        provider_memory: WorthQueryGraphProviderMemorySnapshot,
        projection_bytes: usize,
        artifact_bytes: usize,
    ) -> Self {
        let projection_bytes = u64::try_from(projection_bytes).unwrap_or(u64::MAX);
        let artifact_bytes = u64::try_from(artifact_bytes).unwrap_or(u64::MAX);
        let provider_bytes = provider_memory.retained_bytes();
        Self {
            provider_memory_arena_identity: provider_memory.arena_identity(),
            provider_allocation_count: provider_memory.retained_allocation_count(),
            provider_bytes,
            projection_bytes,
            output_bytes: 0,
            artifact_bytes,
            total_bytes: provider_bytes
                .saturating_add(projection_bytes)
                .saturating_add(artifact_bytes),
        }
    }

    pub const fn provider_memory_arena_identity(self) -> u64 {
        self.provider_memory_arena_identity
    }

    pub const fn provider_allocation_count(self) -> u64 {
        self.provider_allocation_count
    }

    pub const fn provider_bytes(self) -> u64 {
        self.provider_bytes
    }

    pub const fn projection_bytes(self) -> u64 {
        self.projection_bytes
    }

    pub const fn artifact_bytes(self) -> u64 {
        self.artifact_bytes
    }

    pub const fn output_bytes(self) -> u64 {
        self.output_bytes
    }

    pub const fn total_bytes(self) -> u64 {
        self.total_bytes
    }

    pub(crate) fn release_projection_bytes(&mut self, released_bytes: u64) -> bool {
        let Some(projection_bytes) = self.projection_bytes.checked_sub(released_bytes) else {
            return false;
        };
        let Some(total_bytes) = self.total_bytes.checked_sub(released_bytes) else {
            return false;
        };
        self.projection_bytes = projection_bytes;
        self.total_bytes = total_bytes;
        true
    }

    pub(crate) fn retain_prior_output_bytes(&mut self, retained_bytes: u64) {
        self.output_bytes = self.output_bytes.saturating_add(retained_bytes);
        self.total_bytes = self.total_bytes.saturating_add(retained_bytes);
    }

    pub(crate) fn transfer_projection_to_output(
        &mut self,
        projection_bytes: u64,
        additional_bytes: u64,
    ) -> bool {
        let Some(remaining_projection) = self.projection_bytes.checked_sub(projection_bytes) else {
            return false;
        };
        self.projection_bytes = remaining_projection;
        self.output_bytes = self
            .output_bytes
            .saturating_add(projection_bytes)
            .saturating_add(additional_bytes);
        self.total_bytes = self.total_bytes.saturating_add(additional_bytes);
        true
    }
}

#[derive(Debug, PartialEq)]
pub struct WorthQueryGraphProviderStepReport {
    completion: WorthQueryGraphProviderStepCompletion,
    provider_receipt: Option<Arc<str>>,
    completed_work_units: u64,
    attempted_effect_count: u64,
    applied_effect_count: u64,
    peak_scratch_bytes: u64,
    retained: WorthQueryGraphProviderStepRetainedEvidence,
    projection: Option<WorthQueryGraphReadMaterial>,
    artifacts: WorthQueryGraphProviderStepArtifactEvidence,
    checkpoint_available: bool,
    failure: Option<WorthQueryGraphProviderStepFailureEvidence>,
}

pub(crate) struct WorthQueryGraphProviderStepReportParts {
    pub(crate) completed_work_units: u64,
    pub(crate) attempted_effect_count: u64,
    pub(crate) applied_effect_count: u64,
    pub(crate) peak_scratch_bytes: u64,
    pub(crate) retained: WorthQueryGraphProviderStepRetainedEvidence,
    pub(crate) projection: Option<WorthQueryGraphReadMaterial>,
    pub(crate) artifacts: WorthQueryGraphProviderStepArtifactEvidence,
    pub(crate) checkpoint_available: bool,
}

impl WorthQueryGraphProviderStepReport {
    pub(super) fn from_disposition(
        disposition: super::WorthQueryGraphProviderStepDisposition,
        parts: WorthQueryGraphProviderStepReportParts,
    ) -> Self {
        let completion = match disposition.kind() {
            WorthQueryGraphProviderStepDispositionKind::Continue => {
                WorthQueryGraphProviderStepCompletion::Continue
            }
            WorthQueryGraphProviderStepDispositionKind::Complete => {
                WorthQueryGraphProviderStepCompletion::Complete
            }
        };
        Self::from_parts(completion, disposition.into_provider_receipt(), parts, None)
    }

    pub(super) fn failed(
        parts: WorthQueryGraphProviderStepReportParts,
        failure: WorthQueryGraphProviderStepFailureEvidence,
    ) -> Self {
        Self::from_parts(
            WorthQueryGraphProviderStepCompletion::Failed,
            None,
            parts,
            Some(failure),
        )
    }

    fn from_parts(
        completion: WorthQueryGraphProviderStepCompletion,
        provider_receipt: Option<Arc<str>>,
        parts: WorthQueryGraphProviderStepReportParts,
        failure: Option<WorthQueryGraphProviderStepFailureEvidence>,
    ) -> Self {
        Self {
            completion,
            provider_receipt,
            completed_work_units: parts.completed_work_units,
            attempted_effect_count: parts.attempted_effect_count,
            applied_effect_count: parts.applied_effect_count,
            peak_scratch_bytes: parts.peak_scratch_bytes,
            retained: parts.retained,
            projection: parts.projection,
            artifacts: parts.artifacts,
            checkpoint_available: parts.checkpoint_available,
            failure,
        }
    }

    pub const fn completed_work_units(&self) -> u64 {
        self.completed_work_units
    }

    pub const fn attempted_effect_count(&self) -> u64 {
        self.attempted_effect_count
    }

    pub const fn applied_effect_count(&self) -> u64 {
        self.applied_effect_count
    }

    pub const fn peak_scratch_bytes(&self) -> u64 {
        self.peak_scratch_bytes
    }

    pub const fn retained_bytes(&self) -> u64 {
        self.retained.total_bytes()
    }

    pub const fn retained_evidence(&self) -> WorthQueryGraphProviderStepRetainedEvidence {
        self.retained
    }

    pub const fn has_projection_chunk(&self) -> bool {
        self.projection.is_some()
    }

    pub const fn artifact_evidence(&self) -> WorthQueryGraphProviderStepArtifactEvidence {
        self.artifacts
    }

    pub const fn checkpoint_available(&self) -> bool {
        self.checkpoint_available
    }

    pub const fn failure(&self) -> Option<&WorthQueryGraphProviderStepFailureEvidence> {
        self.failure.as_ref()
    }

    pub(crate) const fn completion(&self) -> WorthQueryGraphProviderStepCompletion {
        self.completion
    }

    pub(crate) fn provider_receipt(&self) -> Option<&str> {
        self.provider_receipt.as_deref()
    }

    pub(crate) fn take_projection(&mut self) -> Option<WorthQueryGraphReadMaterial> {
        self.projection.take()
    }
}
