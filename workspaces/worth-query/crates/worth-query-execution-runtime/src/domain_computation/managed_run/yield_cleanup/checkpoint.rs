use std::sync::Arc;

use crate::domain_computation::{
    WorthQueryProviderCheckpointReleaseDisposition, WorthQueryProviderCheckpointReleaseEvidence,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryYieldCleanupCheckpointInspection {
    identity: Arc<str>,
    provider_generation: u64,
    retained_bytes: u64,
    release_disposition: WorthQueryProviderCheckpointReleaseDisposition,
}

impl WorthQueryYieldCleanupCheckpointInspection {
    pub fn capture(
        release: &WorthQueryProviderCheckpointReleaseEvidence,
    ) -> Self {
        Self {
            identity: Arc::from(release.checkpoint().identity()),
            provider_generation: release.checkpoint().provider_generation(),
            retained_bytes: release.checkpoint().retained_bytes(),
            release_disposition: release.disposition(),
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

    pub const fn release_disposition(&self) -> WorthQueryProviderCheckpointReleaseDisposition {
        self.release_disposition
    }
}
