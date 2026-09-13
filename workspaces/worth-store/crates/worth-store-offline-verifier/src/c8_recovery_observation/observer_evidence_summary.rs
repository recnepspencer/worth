use super::observer_evidence::{
    RecoveryObserverCheckpointCoverageEvidence, RecoveryObserverEvidenceDigest,
    RecoveryObserverManifestMembershipEvidence, RecoveryObserverPageLsnEvidence,
    RecoveryObserverResidueEvidence, RecoveryObserverSelectorEvidence,
    RecoveryObserverWalPrefixEvidence,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RecoveryObserverEvidence {
    artifact_identities: RecoveryObserverEvidenceDigest,
    generation_links: RecoveryObserverEvidenceDigest,
    durable_selectors: RecoveryObserverSelectorEvidence,
    checkpoint_coverage: RecoveryObserverCheckpointCoverageEvidence,
    valid_wal_prefix: RecoveryObserverWalPrefixEvidence,
    page_lsns: RecoveryObserverPageLsnEvidence,
    manifest_membership: RecoveryObserverManifestMembershipEvidence,
    residue: RecoveryObserverResidueEvidence,
}

impl RecoveryObserverEvidence {
    pub(super) const fn artifact_identities(self) -> RecoveryObserverEvidenceDigest {
        self.artifact_identities
    }

    pub(super) const fn generation_links(self) -> RecoveryObserverEvidenceDigest {
        self.generation_links
    }

    pub(super) const fn durable_selectors(self) -> RecoveryObserverSelectorEvidence {
        self.durable_selectors
    }

    pub(super) const fn checkpoint_coverage(self) -> RecoveryObserverCheckpointCoverageEvidence {
        self.checkpoint_coverage
    }

    pub(super) const fn valid_wal_prefix(self) -> RecoveryObserverWalPrefixEvidence {
        self.valid_wal_prefix
    }

    pub(super) const fn page_lsns(self) -> RecoveryObserverPageLsnEvidence {
        self.page_lsns
    }

    pub(super) const fn manifest_membership(self) -> RecoveryObserverManifestMembershipEvidence {
        self.manifest_membership
    }

    pub(super) const fn residue(self) -> RecoveryObserverResidueEvidence {
        self.residue
    }

    pub(super) const fn from_parts(
        artifact_identities: RecoveryObserverEvidenceDigest,
        generation_links: RecoveryObserverEvidenceDigest,
        durable_selectors: RecoveryObserverSelectorEvidence,
        checkpoint_coverage: RecoveryObserverCheckpointCoverageEvidence,
        valid_wal_prefix: RecoveryObserverWalPrefixEvidence,
        page_lsns: RecoveryObserverPageLsnEvidence,
        manifest_membership: RecoveryObserverManifestMembershipEvidence,
        residue: RecoveryObserverResidueEvidence,
    ) -> Self {
        Self {
            artifact_identities,
            generation_links,
            durable_selectors,
            checkpoint_coverage,
            valid_wal_prefix,
            page_lsns,
            manifest_membership,
            residue,
        }
    }
}
