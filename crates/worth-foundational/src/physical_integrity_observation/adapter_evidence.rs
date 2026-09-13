use serde::{Deserialize, Serialize};

use super::{
    PhysicalArtifactFamily, PhysicalArtifactGeneration, PhysicalArtifactIdentity,
    PhysicalAuthorityClass, PhysicalByteRange, PhysicalIntegrityPosture, PhysicalQuarantinePosture,
};

/// Bounded descriptive projection used by runtime and process adapters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicalAdapterEvidence {
    family: PhysicalArtifactFamily,
    identity: PhysicalArtifactIdentity,
    generation: PhysicalArtifactGeneration,
    range: PhysicalByteRange,
    integrity: PhysicalIntegrityPosture,
    authority: Option<PhysicalAuthorityClass>,
    quarantine: PhysicalQuarantinePosture,
}

impl PhysicalAdapterEvidence {
    pub const fn new(
        family: PhysicalArtifactFamily,
        identity: PhysicalArtifactIdentity,
        generation: PhysicalArtifactGeneration,
        range: PhysicalByteRange,
        integrity: PhysicalIntegrityPosture,
    ) -> Self {
        Self {
            family,
            identity,
            generation,
            range,
            integrity,
            authority: None,
            quarantine: PhysicalQuarantinePosture::NotObserved,
        }
    }

    pub const fn with_owner_class(mut self, authority: PhysicalAuthorityClass) -> Self {
        self.authority = Some(authority);
        self
    }

    pub const fn with_quarantine(mut self, quarantine: PhysicalQuarantinePosture) -> Self {
        self.quarantine = quarantine;
        self
    }

    pub const fn family(&self) -> PhysicalArtifactFamily {
        self.family
    }

    pub const fn identity(&self) -> &PhysicalArtifactIdentity {
        &self.identity
    }

    pub const fn generation(&self) -> PhysicalArtifactGeneration {
        self.generation
    }

    pub const fn range(&self) -> PhysicalByteRange {
        self.range
    }

    pub const fn integrity(&self) -> PhysicalIntegrityPosture {
        self.integrity
    }

    pub const fn owner_class(&self) -> Option<PhysicalAuthorityClass> {
        self.authority
    }

    pub const fn quarantine(&self) -> PhysicalQuarantinePosture {
        self.quarantine
    }
}
