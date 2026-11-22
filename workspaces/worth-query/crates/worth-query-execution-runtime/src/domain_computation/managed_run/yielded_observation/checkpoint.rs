use std::sync::Arc;

use crate::domain_computation::WorthQueryProviderCheckpointEvidence;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryYieldedCheckpointInspection {
    identity: Arc<str>,
    provider_generation: u64,
    retained_bytes: u64,
}

impl WorthQueryYieldedCheckpointInspection {
    pub(super) fn capture(checkpoint: &WorthQueryProviderCheckpointEvidence) -> Self {
        Self {
            identity: Arc::from(checkpoint.identity()),
            provider_generation: checkpoint.provider_generation(),
            retained_bytes: checkpoint.retained_bytes(),
        }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn provider_generation(&self) -> u64 {
        self.provider_generation
    }

    pub const fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }
}
