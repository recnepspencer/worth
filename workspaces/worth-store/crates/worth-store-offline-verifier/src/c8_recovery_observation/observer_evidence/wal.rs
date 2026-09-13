#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::c8_recovery_observation) struct RecoveryObserverWalPrefixEvidence {
    segment_count: u64,
    valid_prefix_bytes: u64,
    observed_bytes: u64,
    frame_count: u64,
    first_lsn: Option<u64>,
    last_lsn: Option<u64>,
    digest: [u8; 32],
}

impl RecoveryObserverWalPrefixEvidence {
    pub(in crate::c8_recovery_observation) const fn segment_count(self) -> u64 {
        self.segment_count
    }

    pub(in crate::c8_recovery_observation) const fn valid_prefix_bytes(self) -> u64 {
        self.valid_prefix_bytes
    }

    pub(in crate::c8_recovery_observation) const fn observed_bytes(self) -> u64 {
        self.observed_bytes
    }

    pub(in crate::c8_recovery_observation) const fn frame_count(self) -> u64 {
        self.frame_count
    }

    pub(in crate::c8_recovery_observation) const fn first_lsn(self) -> Option<u64> {
        self.first_lsn
    }

    pub(in crate::c8_recovery_observation) const fn last_lsn(self) -> Option<u64> {
        self.last_lsn
    }

    pub(in crate::c8_recovery_observation) const fn digest(self) -> [u8; 32] {
        self.digest
    }

    pub(in crate::c8_recovery_observation) const fn from_parts(
        segment_count: u64,
        valid_prefix_bytes: u64,
        observed_bytes: u64,
        frame_count: u64,
        first_lsn: Option<u64>,
        last_lsn: Option<u64>,
        digest: [u8; 32],
    ) -> Self {
        Self {
            segment_count,
            valid_prefix_bytes,
            observed_bytes,
            frame_count,
            first_lsn,
            last_lsn,
            digest,
        }
    }
}
