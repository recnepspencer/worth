#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::c8_recovery_observation) struct RecoveryObserverPageLsnEvidence {
    observation_count: u64,
    minimum: Option<u64>,
    maximum: Option<u64>,
    digest: [u8; 32],
}

impl RecoveryObserverPageLsnEvidence {
    pub(in crate::c8_recovery_observation) const fn observation_count(self) -> u64 {
        self.observation_count
    }

    pub(in crate::c8_recovery_observation) const fn minimum(self) -> Option<u64> {
        self.minimum
    }

    pub(in crate::c8_recovery_observation) const fn maximum(self) -> Option<u64> {
        self.maximum
    }

    pub(in crate::c8_recovery_observation) const fn digest(self) -> [u8; 32] {
        self.digest
    }

    pub(in crate::c8_recovery_observation) const fn from_parts(
        observation_count: u64,
        minimum: Option<u64>,
        maximum: Option<u64>,
        digest: [u8; 32],
    ) -> Self {
        Self {
            observation_count,
            minimum,
            maximum,
            digest,
        }
    }
}
