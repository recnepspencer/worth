use super::super::observer_evidence::RecoveryObserverEvidenceDigest;
use super::super::observer_evidence_accumulation::{
    EvidenceDigestBuilder, RecoveryObserverWalObservation, RecoveryObserverWalTopologyObservation,
};
use super::wal_frame_decode::DecodedWalFrame;

pub(super) struct WalPrefixProgression {
    offset: usize,
    previous_lsn_end: Option<u64>,
    frame_count: u64,
    first_lsn: Option<u64>,
    last_lsn: Option<u64>,
    expected_segment: Option<u64>,
    expected_generation: Option<u64>,
    wal_digest: EvidenceDigestBuilder,
    generation_digest: EvidenceDigestBuilder,
    topology: Option<RecoveryObserverWalTopologyObservation>,
}

pub(super) struct FinalizedWalPrefix {
    pub(super) offset: usize,
    pub(super) wal: RecoveryObserverWalObservation,
    pub(super) generation_links: RecoveryObserverEvidenceDigest,
    pub(super) topology: Option<RecoveryObserverWalTopologyObservation>,
}

impl WalPrefixProgression {
    pub(super) fn new() -> Self {
        Self {
            offset: 0,
            previous_lsn_end: None,
            frame_count: 0,
            first_lsn: None,
            last_lsn: None,
            expected_segment: None,
            expected_generation: None,
            wal_digest: EvidenceDigestBuilder::new(b"worth.store.recovery-observer.wal-prefix.v1"),
            generation_digest: EvidenceDigestBuilder::new(
                b"worth.store.recovery-observer.generation-link.v1",
            ),
            topology: None,
        }
    }

    pub(super) const fn offset(&self) -> usize {
        self.offset
    }

    pub(super) const fn expected_segment(&self) -> Option<u64> {
        self.expected_segment
    }

    pub(super) const fn expected_generation(&self) -> Option<u64> {
        self.expected_generation
    }

    pub(super) const fn previous_lsn_end(&self) -> Option<u64> {
        self.previous_lsn_end
    }

    pub(super) fn record(&mut self, frame: DecodedWalFrame) {
        self.expected_segment.get_or_insert(frame.segment_id);
        self.expected_generation.get_or_insert(frame.generation);
        let mut record = Vec::with_capacity(40);
        record.extend_from_slice(&frame.segment_id.to_le_bytes());
        record.extend_from_slice(&frame.generation.to_le_bytes());
        record.extend_from_slice(&frame.lsn_start.to_le_bytes());
        record.extend_from_slice(&frame.lsn_end.to_le_bytes());
        self.wal_digest.record(&record);
        self.generation_digest.record(&record);
        self.first_lsn.get_or_insert(frame.lsn_start);
        self.last_lsn = Some(frame.lsn_end);
        self.previous_lsn_end = Some(frame.lsn_end);
        self.frame_count = self.frame_count.saturating_add(1);
        match &mut self.topology {
            Some(topology) => topology.last_lsn = frame.lsn_end,
            None => {
                self.topology = Some(RecoveryObserverWalTopologyObservation {
                    segment: frame.segment_id,
                    generation: frame.generation,
                    first_lsn: frame.lsn_start,
                    last_lsn: frame.lsn_end,
                    denial: None,
                });
            }
        }
        self.offset = self.offset.saturating_add(frame.total_bytes);
    }

    pub(super) fn stop(&mut self, topology: Option<RecoveryObserverWalTopologyObservation>) {
        if topology.is_some() {
            self.topology = topology;
        }
    }

    pub(super) fn finish(self, observed_bytes: u64) -> FinalizedWalPrefix {
        FinalizedWalPrefix {
            offset: self.offset,
            wal: RecoveryObserverWalObservation {
                valid_prefix_bytes: self.offset as u64,
                observed_bytes,
                frame_count: self.frame_count,
                first_lsn: self.first_lsn,
                last_lsn: self.last_lsn,
                digest: self.wal_digest.finish().digest(),
            },
            generation_links: self.generation_digest.finish(),
            topology: self.topology,
        }
    }
}
