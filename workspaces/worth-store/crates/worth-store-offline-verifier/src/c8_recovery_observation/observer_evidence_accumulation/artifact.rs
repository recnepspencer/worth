use super::super::observer_evidence::RecoveryObserverEvidenceDigest;
use super::{
    checkpoint::RecoveryObserverCheckpointObservation,
    manifest::RecoveryObserverManifestMembershipObservation,
    page::RecoveryObserverPageLsnObservation, residue::RecoveryObserverResidueObservation,
    selector::RecoveryObserverSelectorObservation, wal::RecoveryObserverWalObservation,
    wal_topology::RecoveryObserverWalTopologyObservation,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RecoveryObserverArtifactEvidence {
    pub(crate) generation_links: RecoveryObserverEvidenceDigest,
    pub(crate) selector: Option<RecoveryObserverSelectorObservation>,
    pub(crate) checkpoint: Option<RecoveryObserverCheckpointObservation>,
    pub(crate) wal_prefix: Option<RecoveryObserverWalObservation>,
    pub(crate) wal_topology: Option<RecoveryObserverWalTopologyObservation>,
    pub(crate) page_lsns: RecoveryObserverPageLsnObservation,
    pub(crate) manifest_membership: RecoveryObserverManifestMembershipObservation,
    pub(crate) residue: RecoveryObserverResidueObservation,
}

impl RecoveryObserverArtifactEvidence {
    pub(crate) const fn empty() -> Self {
        Self {
            generation_links: RecoveryObserverEvidenceDigest::empty(),
            selector: None,
            checkpoint: None,
            wal_prefix: None,
            wal_topology: None,
            page_lsns: RecoveryObserverPageLsnObservation::empty(),
            manifest_membership: RecoveryObserverManifestMembershipObservation::empty(),
            residue: RecoveryObserverResidueObservation::empty(),
        }
    }
}
