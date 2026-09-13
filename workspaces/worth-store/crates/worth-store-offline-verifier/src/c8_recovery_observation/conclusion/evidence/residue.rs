use super::super::super::observer_evidence::RecoveryObserverResidueEvidence;
use super::super::super::observer_evidence_accumulation::{
    EvidenceDigestBuilder, RecoveryObserverResidueObservation,
};

pub(crate) struct ResidueEvidenceAccumulator {
    artifact_count: u64,
    bytes: u64,
    digest: EvidenceDigestBuilder,
}

impl ResidueEvidenceAccumulator {
    pub(crate) fn new() -> Self {
        Self {
            artifact_count: 0,
            bytes: 0,
            digest: EvidenceDigestBuilder::new(b"worth.store.recovery-observer.residue.v1"),
        }
    }

    pub(crate) fn observe(&mut self, residue: RecoveryObserverResidueObservation) {
        if residue.bytes == 0 {
            return;
        }
        self.artifact_count = self.artifact_count.saturating_add(1);
        self.bytes = self.bytes.saturating_add(residue.bytes);
        self.digest.record(&residue.digest);
    }

    pub(crate) fn finish(self) -> RecoveryObserverResidueEvidence {
        RecoveryObserverResidueEvidence::from_parts(
            self.artifact_count,
            self.bytes,
            self.digest.finish().digest(),
        )
    }
}
