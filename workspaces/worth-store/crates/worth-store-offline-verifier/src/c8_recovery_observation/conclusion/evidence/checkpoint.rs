use super::super::super::observer_evidence::RecoveryObserverCheckpointCoverageEvidence;
use super::super::super::observer_evidence_accumulation::{
    EvidenceDigestBuilder, RecoveryObserverCheckpointObservation,
};

pub(crate) struct CheckpointEvidenceAccumulator {
    count: u64,
    pages: u64,
    covered_start: Option<u64>,
    covered_end: Option<u64>,
    redo_lsn: Option<u64>,
    durable_checkpoint_lsn: Option<u64>,
    digest: EvidenceDigestBuilder,
}

impl CheckpointEvidenceAccumulator {
    pub(crate) fn new() -> Self {
        Self {
            count: 0,
            pages: 0,
            covered_start: None,
            covered_end: None,
            redo_lsn: None,
            durable_checkpoint_lsn: None,
            digest: EvidenceDigestBuilder::new(
                b"worth.store.recovery-observer.checkpoint-coverage.v1",
            ),
        }
    }

    pub(crate) fn observe(&mut self, checkpoint: RecoveryObserverCheckpointObservation) {
        self.count = self.count.saturating_add(1);
        self.pages = self.pages.saturating_add(checkpoint.page_count);
        self.covered_start = min_option(self.covered_start, checkpoint.covered_lsn.0);
        self.covered_end = max_option(self.covered_end, checkpoint.covered_lsn.1);
        self.redo_lsn = min_option(self.redo_lsn, checkpoint.redo_lsn);
        self.durable_checkpoint_lsn = max_option(
            self.durable_checkpoint_lsn,
            checkpoint.durable_checkpoint_lsn,
        );
        self.digest.record(&checkpoint.digest);
    }

    pub(crate) fn finish(self) -> RecoveryObserverCheckpointCoverageEvidence {
        RecoveryObserverCheckpointCoverageEvidence::from_parts(
            self.count,
            self.pages,
            self.covered_start,
            self.covered_end,
            self.redo_lsn,
            self.durable_checkpoint_lsn,
            self.digest.finish().digest(),
        )
    }
}

fn min_option(current: Option<u64>, value: u64) -> Option<u64> {
    Some(current.map_or(value, |current| current.min(value)))
}

fn max_option(current: Option<u64>, value: u64) -> Option<u64> {
    Some(current.map_or(value, |current| current.max(value)))
}
