#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::c8_recovery_observation) struct RecoveryObserverCheckpointCoverageEvidence {
    checkpoint_count: u64,
    page_count: u64,
    covered_lsn_start: Option<u64>,
    covered_lsn_end: Option<u64>,
    redo_lsn: Option<u64>,
    durable_checkpoint_lsn: Option<u64>,
    digest: [u8; 32],
}

impl RecoveryObserverCheckpointCoverageEvidence {
    pub(in crate::c8_recovery_observation) const fn checkpoint_count(self) -> u64 {
        self.checkpoint_count
    }

    pub(in crate::c8_recovery_observation) const fn page_count(self) -> u64 {
        self.page_count
    }

    pub(in crate::c8_recovery_observation) const fn covered_lsn_start(self) -> Option<u64> {
        self.covered_lsn_start
    }

    pub(in crate::c8_recovery_observation) const fn covered_lsn_end(self) -> Option<u64> {
        self.covered_lsn_end
    }

    pub(in crate::c8_recovery_observation) const fn redo_lsn(self) -> Option<u64> {
        self.redo_lsn
    }

    pub(in crate::c8_recovery_observation) const fn durable_checkpoint_lsn(self) -> Option<u64> {
        self.durable_checkpoint_lsn
    }

    pub(in crate::c8_recovery_observation) const fn digest(self) -> [u8; 32] {
        self.digest
    }

    pub(in crate::c8_recovery_observation) const fn from_parts(
        checkpoint_count: u64,
        page_count: u64,
        covered_lsn_start: Option<u64>,
        covered_lsn_end: Option<u64>,
        redo_lsn: Option<u64>,
        durable_checkpoint_lsn: Option<u64>,
        digest: [u8; 32],
    ) -> Self {
        Self {
            checkpoint_count,
            page_count,
            covered_lsn_start,
            covered_lsn_end,
            redo_lsn,
            durable_checkpoint_lsn,
            digest,
        }
    }
}
