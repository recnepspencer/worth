use crate::runtime::replacement::candidate::{
    WorthUiCandidateArtifactBundle, WorthUiCandidateAuthoringLane, WorthUiCandidateLoweringBasis,
    WorthUiCandidateProvenanceHandle, WorthUiReplacementCandidateBasis,
    WorthUiReplacementCandidateDenial, WorthUiReplacementCause,
};

#[derive(Debug, Eq, PartialEq)]
pub struct WorthUiReplacementCandidate {
    predecessor_snapshot_digest: u64,
    bundle: WorthUiCandidateArtifactBundle,
    cause: WorthUiReplacementCause,
    provenance_handle: WorthUiCandidateProvenanceHandle,
    authoring_lane: WorthUiCandidateAuthoringLane,
}

impl WorthUiReplacementCandidate {
    pub(crate) fn from_artifact_bundle(
        bundle: WorthUiCandidateArtifactBundle,
        cause: WorthUiReplacementCause,
        authoring_lane: WorthUiCandidateAuthoringLane,
    ) -> Result<Self, WorthUiReplacementCandidateDenial> {
        let provenance_handle = cause.provenance_handle();
        Ok(Self {
            predecessor_snapshot_digest: bundle.lowering_basis().snapshot_digest(),
            bundle,
            cause,
            provenance_handle,
            authoring_lane,
        })
    }

    pub fn basis(&self) -> WorthUiReplacementCandidateBasis {
        self.bundle.basis()
    }

    pub(crate) fn with_prepared_snapshot_succession(
        mut self,
        handoff: &crate::runtime::WorthUiSemanticHandoffEvidence,
    ) -> Self {
        assert_eq!(
            self.lowering_basis().snapshot_digest(),
            handoff.successor_snapshot_digest().as_u64()
        );
        self.predecessor_snapshot_digest = handoff.predecessor_snapshot_digest().as_u64();
        self
    }

    pub(crate) fn predecessor_snapshot_digest(&self) -> u64 {
        self.predecessor_snapshot_digest
    }

    pub fn lowering_basis(&self) -> WorthUiCandidateLoweringBasis {
        self.bundle.lowering_basis()
    }

    pub fn cause(&self) -> &WorthUiReplacementCause {
        &self.cause
    }

    pub fn provenance_handle(&self) -> WorthUiCandidateProvenanceHandle {
        self.provenance_handle
    }

    pub fn authoring_lane(&self) -> WorthUiCandidateAuthoringLane {
        self.authoring_lane
    }

    pub(crate) fn artifact_bundle(&self) -> &WorthUiCandidateArtifactBundle {
        &self.bundle
    }
}
