use serde::{Deserialize, Serialize};

use super::{PhysicalArtifactFamily, PhysicalArtifactIdentity, PhysicalIntegrityPosture};

/// Preserves two observations without declaring either one authoritative.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicalIntegrityDisagreement {
    family: PhysicalArtifactFamily,
    identity: PhysicalArtifactIdentity,
    runtime: PhysicalIntegrityPosture,
    offline: PhysicalIntegrityPosture,
}

impl PhysicalIntegrityDisagreement {
    pub fn new(
        family: PhysicalArtifactFamily,
        identity: PhysicalArtifactIdentity,
        runtime: PhysicalIntegrityPosture,
        offline: PhysicalIntegrityPosture,
    ) -> Option<Self> {
        (runtime != offline).then_some(Self {
            family,
            identity,
            runtime,
            offline,
        })
    }

    pub const fn family(&self) -> PhysicalArtifactFamily {
        self.family
    }

    pub const fn runtime(&self) -> PhysicalIntegrityPosture {
        self.runtime
    }

    pub const fn offline(&self) -> PhysicalIntegrityPosture {
        self.offline
    }

    pub const fn identity(&self) -> &PhysicalArtifactIdentity {
        &self.identity
    }
}
