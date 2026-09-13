#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::c8_recovery_observation) struct RecoveryObserverResidueEvidence {
    artifact_count: u64,
    bytes: u64,
    digest: [u8; 32],
}

impl RecoveryObserverResidueEvidence {
    pub(in crate::c8_recovery_observation) const fn artifact_count(self) -> u64 {
        self.artifact_count
    }

    pub(in crate::c8_recovery_observation) const fn bytes(self) -> u64 {
        self.bytes
    }

    pub(in crate::c8_recovery_observation) const fn digest(self) -> [u8; 32] {
        self.digest
    }

    pub(in crate::c8_recovery_observation) const fn from_parts(
        artifact_count: u64,
        bytes: u64,
        digest: [u8; 32],
    ) -> Self {
        Self {
            artifact_count,
            bytes,
            digest,
        }
    }
}
