use crate::domain_computation::{
    WorthQueryProviderCheckpointReleaseEvidence,
    WorthQueryProviderCheckpointSuspensionFailureEvidence,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryYieldRecoveryResourceEvidence {
    provider_checkpoint_suspension: Option<WorthQueryProviderCheckpointSuspensionFailureEvidence>,
    checkpoint_release: Option<WorthQueryProviderCheckpointReleaseEvidence>,
}

impl WorthQueryYieldRecoveryResourceEvidence {
    pub(super) fn provider_checkpoint_suspension(
        failure: WorthQueryProviderCheckpointSuspensionFailureEvidence,
    ) -> Self {
        Self {
            provider_checkpoint_suspension: Some(failure),
            checkpoint_release: None,
        }
    }

    pub(super) fn retained_bytes_exceeded(
        checkpoint_release: WorthQueryProviderCheckpointReleaseEvidence,
    ) -> Self {
        Self {
            provider_checkpoint_suspension: None,
            checkpoint_release: Some(checkpoint_release),
        }
    }

    pub fn provider_checkpoint_failure(
        &self,
    ) -> Option<&WorthQueryProviderCheckpointSuspensionFailureEvidence> {
        self.provider_checkpoint_suspension.as_ref()
    }

    pub fn checkpoint_release(&self) -> Option<&WorthQueryProviderCheckpointReleaseEvidence> {
        self.checkpoint_release.as_ref()
    }
}
