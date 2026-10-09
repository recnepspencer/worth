#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalRecoveryResidue {
    name: String,
    kind: PhysicalRecoveryResidueKind,
    observed_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PhysicalRecoveryResidueKind {
    NonCanonicalWalArtifact,
    NonRegularWalEntry,
    TrailingEmptyWalSegment,
    InterruptedWalSegmentStart,
    UnreferencedCompactionProduct,
}

impl PhysicalRecoveryResidue {
    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        u64::try_from(self.name.capacity()).ok()
    }

    pub fn new(name: String, kind: PhysicalRecoveryResidueKind) -> Self {
        Self {
            name,
            kind,
            observed_bytes: 0,
        }
    }

    pub fn with_observed_bytes(
        name: String,
        kind: PhysicalRecoveryResidueKind,
        observed_bytes: u64,
    ) -> Self {
        Self {
            name,
            kind,
            observed_bytes,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn kind(&self) -> PhysicalRecoveryResidueKind {
        self.kind
    }

    pub const fn observed_bytes(&self) -> u64 {
        self.observed_bytes
    }
}
