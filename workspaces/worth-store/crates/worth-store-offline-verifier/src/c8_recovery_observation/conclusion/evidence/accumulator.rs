use super::super::super::artifact_walk::ObservedRecoveryArtifact;
use super::super::super::observer_evidence::RecoveryObserverEvidenceDigest;
use super::super::super::observer_evidence_summary::RecoveryObserverEvidence;
use super::{
    checkpoint::CheckpointEvidenceAccumulator, generation::GenerationLinksAccumulator,
    manifest::ManifestEvidenceAccumulator, pages::PageEvidenceAccumulator,
    residue::ResidueEvidenceAccumulator, selector::SelectorEvidenceAccumulator,
    wal::WalEvidenceAccumulator,
};

pub(crate) struct EvidenceAccumulator {
    generation_links: GenerationLinksAccumulator,
    selectors: SelectorEvidenceAccumulator,
    checkpoint: CheckpointEvidenceAccumulator,
    wal: WalEvidenceAccumulator,
    pages: PageEvidenceAccumulator,
    manifest: ManifestEvidenceAccumulator,
    residue: ResidueEvidenceAccumulator,
}

impl EvidenceAccumulator {
    pub(crate) fn new() -> Self {
        Self {
            generation_links: GenerationLinksAccumulator::new(),
            selectors: SelectorEvidenceAccumulator::new(),
            checkpoint: CheckpointEvidenceAccumulator::new(),
            wal: WalEvidenceAccumulator::new(),
            pages: PageEvidenceAccumulator::new(),
            manifest: ManifestEvidenceAccumulator::new(),
            residue: ResidueEvidenceAccumulator::new(),
        }
    }

    pub(crate) fn observe(&mut self, artifact: &ObservedRecoveryArtifact) {
        let evidence = artifact.evidence();
        self.generation_links.observe(artifact);
        if let Some(selector) = evidence.selector {
            self.selectors.observe(selector);
        }
        if let Some(checkpoint) = evidence.checkpoint {
            self.checkpoint.observe(checkpoint);
        }
        if let Some(wal) = evidence.wal_prefix {
            self.wal.observe(wal);
        }
        self.pages.observe(evidence.page_lsns);
        self.manifest.observe(evidence.manifest_membership);
        self.residue.observe(evidence.residue);
    }

    pub(crate) fn finish(
        self,
        artifact_identities: RecoveryObserverEvidenceDigest,
    ) -> RecoveryObserverEvidence {
        RecoveryObserverEvidence::from_parts(
            artifact_identities,
            self.generation_links.finish(),
            self.selectors.finish(),
            self.checkpoint.finish(),
            self.wal.finish(),
            self.pages.finish(),
            self.manifest.finish(),
            self.residue.finish(),
        )
    }
}
