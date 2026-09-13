#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::c8_recovery_observation) struct RecoveryObserverEvidenceDigest {
    observations: u64,
    digest: [u8; 32],
}

impl RecoveryObserverEvidenceDigest {
    pub(in crate::c8_recovery_observation) const fn observations(self) -> u64 {
        self.observations
    }

    pub(in crate::c8_recovery_observation) const fn digest(self) -> [u8; 32] {
        self.digest
    }

    pub(in crate::c8_recovery_observation) const fn empty() -> Self {
        Self {
            observations: 0,
            digest: [0; 32],
        }
    }

    pub(in crate::c8_recovery_observation) const fn from_parts(
        observations: u64,
        digest: [u8; 32],
    ) -> Self {
        Self {
            observations,
            digest,
        }
    }
}
