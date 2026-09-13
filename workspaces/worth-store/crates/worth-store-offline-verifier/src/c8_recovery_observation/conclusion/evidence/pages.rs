use super::super::super::observer_evidence::RecoveryObserverPageLsnEvidence;
use super::super::super::observer_evidence_accumulation::{
    EvidenceDigestBuilder, RecoveryObserverPageLsnObservation,
};

pub(crate) struct PageEvidenceAccumulator {
    count: u64,
    minimum: Option<u64>,
    maximum: Option<u64>,
    digest: EvidenceDigestBuilder,
}

impl PageEvidenceAccumulator {
    pub(crate) fn new() -> Self {
        Self {
            count: 0,
            minimum: None,
            maximum: None,
            digest: EvidenceDigestBuilder::new(b"worth.store.recovery-observer.page-lsn.v1"),
        }
    }

    pub(crate) fn observe(&mut self, page_lsns: RecoveryObserverPageLsnObservation) {
        self.count = self.count.saturating_add(page_lsns.count);
        if let Some(minimum) = page_lsns.minimum {
            self.minimum = min_option(self.minimum, minimum);
        }
        if let Some(maximum) = page_lsns.maximum {
            self.maximum = max_option(self.maximum, maximum);
        }
        if page_lsns.count > 0 {
            self.digest.record(&page_lsns.digest);
        }
    }

    pub(crate) fn finish(self) -> RecoveryObserverPageLsnEvidence {
        RecoveryObserverPageLsnEvidence::from_parts(
            self.count,
            self.minimum,
            self.maximum,
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
