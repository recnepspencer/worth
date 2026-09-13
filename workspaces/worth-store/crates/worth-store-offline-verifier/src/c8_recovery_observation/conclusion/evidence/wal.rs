use super::super::super::observer_evidence::RecoveryObserverWalPrefixEvidence;
use super::super::super::observer_evidence_accumulation::{
    EvidenceDigestBuilder, RecoveryObserverWalObservation,
};

pub(crate) struct WalEvidenceAccumulator {
    segments: u64,
    valid_prefix_bytes: u64,
    observed_bytes: u64,
    frame_count: u64,
    first_lsn: Option<u64>,
    last_lsn: Option<u64>,
    digest: EvidenceDigestBuilder,
}

impl WalEvidenceAccumulator {
    pub(crate) fn new() -> Self {
        Self {
            segments: 0,
            valid_prefix_bytes: 0,
            observed_bytes: 0,
            frame_count: 0,
            first_lsn: None,
            last_lsn: None,
            digest: EvidenceDigestBuilder::new(b"worth.store.recovery-observer.wal-prefix.v1"),
        }
    }

    pub(crate) fn observe(&mut self, wal: RecoveryObserverWalObservation) {
        self.segments = self.segments.saturating_add(1);
        self.valid_prefix_bytes = self
            .valid_prefix_bytes
            .saturating_add(wal.valid_prefix_bytes);
        self.observed_bytes = self.observed_bytes.saturating_add(wal.observed_bytes);
        self.frame_count = self.frame_count.saturating_add(wal.frame_count);
        if let Some(first) = wal.first_lsn {
            self.first_lsn = min_option(self.first_lsn, first);
        }
        if let Some(last) = wal.last_lsn {
            self.last_lsn = max_option(self.last_lsn, last);
        }
        self.digest.record(&wal.digest);
    }

    pub(crate) fn finish(self) -> RecoveryObserverWalPrefixEvidence {
        RecoveryObserverWalPrefixEvidence::from_parts(
            self.segments,
            self.valid_prefix_bytes,
            self.observed_bytes,
            self.frame_count,
            self.first_lsn,
            self.last_lsn,
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
