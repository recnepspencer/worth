use super::super::super::artifact_walk::ObservedRecoveryArtifact;
use super::super::super::observer_evidence::RecoveryObserverEvidenceDigest;
use super::super::super::observer_evidence_accumulation::EvidenceDigestBuilder;

pub(crate) struct GenerationLinksAccumulator {
    digest: EvidenceDigestBuilder,
}

impl GenerationLinksAccumulator {
    pub(crate) fn new() -> Self {
        Self {
            digest: EvidenceDigestBuilder::new(b"worth.store.recovery-observer.generation-link.v1"),
        }
    }

    pub(crate) fn observe(&mut self, artifact: &ObservedRecoveryArtifact) {
        let evidence = artifact.evidence();
        if evidence.generation_links.observations() == 0 {
            return;
        }
        let mut record = Vec::with_capacity(artifact.path().len() + 40);
        record.extend_from_slice(artifact.path().as_bytes());
        record.extend_from_slice(&evidence.generation_links.digest());
        self.digest.record(&record);
    }

    pub(crate) fn finish(self) -> RecoveryObserverEvidenceDigest {
        self.digest.finish()
    }
}
