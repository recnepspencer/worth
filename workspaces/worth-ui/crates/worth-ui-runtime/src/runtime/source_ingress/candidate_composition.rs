use crate::facade::prepared_application_authority::WorthUiPreparedDeclarationSourceIdentity;
use crate::runtime::{
    WorthUiCandidateAuthoringLane, WorthUiReplacementCandidate, WorthUiReplacementCandidateBasis,
};

use super::{WorthUiPreparedDeclarationMaterial, WorthUiSemanticHandoffEvidence};

/// Comparison evidence binding candidate artifact identity to the distinct
/// declaration-source identity produced by the same structured input.
#[derive(Clone, Debug)]
pub struct WorthUiCandidateCompositionBasis {
    candidate: WorthUiReplacementCandidateBasis,
    declaration_source: WorthUiPreparedDeclarationSourceIdentity,
    semantic_handoff: WorthUiSemanticHandoffEvidence,
}

impl PartialEq for WorthUiCandidateCompositionBasis {
    fn eq(&self, other: &Self) -> bool {
        self.candidate == other.candidate
            && self.declaration_source == other.declaration_source
            && self.semantic_handoff.identity() == other.semantic_handoff.identity()
            && self.semantic_handoff.protocol() == other.semantic_handoff.protocol()
    }
}

impl Eq for WorthUiCandidateCompositionBasis {}

/// Sealed pre-authority input. Artifact and declaration source can only move
/// together into application preparation.
#[derive(Debug, Eq, PartialEq)]
pub struct WorthUiCandidateComposition {
    candidate: WorthUiReplacementCandidate,
    declaration_material: WorthUiPreparedDeclarationMaterial,
    semantic_handoff: WorthUiSemanticHandoffEvidence,
    basis: WorthUiCandidateCompositionBasis,
}

pub(crate) struct WorthUiCandidatePreparationHandoff {
    candidate: WorthUiReplacementCandidate,
    declaration_material: WorthUiPreparedDeclarationMaterial,
    semantic_handoff: WorthUiSemanticHandoffEvidence,
    composition_basis: WorthUiCandidateCompositionBasis,
}

impl WorthUiCandidateComposition {
    pub(super) fn file_authored(
        candidate: WorthUiReplacementCandidate,
        declaration_material: WorthUiPreparedDeclarationMaterial,
        semantic_handoff: WorthUiSemanticHandoffEvidence,
    ) -> Self {
        Self::new(candidate, declaration_material, semantic_handoff)
    }

    pub(super) fn rust_authored(
        candidate: WorthUiReplacementCandidate,
        declaration_material: WorthUiPreparedDeclarationMaterial,
        semantic_handoff: WorthUiSemanticHandoffEvidence,
    ) -> Self {
        Self::new(candidate, declaration_material, semantic_handoff)
    }

    fn new(
        candidate: WorthUiReplacementCandidate,
        declaration_material: WorthUiPreparedDeclarationMaterial,
        semantic_handoff: WorthUiSemanticHandoffEvidence,
    ) -> Self {
        let candidate = candidate.with_prepared_snapshot_succession(&semantic_handoff);
        let basis = WorthUiCandidateCompositionBasis {
            candidate: candidate.basis(),
            declaration_source: declaration_material.identity().clone(),
            semantic_handoff: semantic_handoff.clone(),
        };
        Self {
            candidate,
            declaration_material,
            semantic_handoff,
            basis,
        }
    }

    pub fn basis(&self) -> &WorthUiCandidateCompositionBasis {
        &self.basis
    }

    pub fn authoring_lane(&self) -> WorthUiCandidateAuthoringLane {
        self.candidate.authoring_lane()
    }

    pub(super) fn snapshot_digest(&self) -> u64 {
        self.candidate.predecessor_snapshot_digest()
    }

    pub(super) fn into_preparation_handoff(self) -> WorthUiCandidatePreparationHandoff {
        WorthUiCandidatePreparationHandoff {
            candidate: self.candidate,
            declaration_material: self.declaration_material,
            semantic_handoff: self.semantic_handoff,
            composition_basis: self.basis,
        }
    }
}

impl WorthUiCandidatePreparationHandoff {
    pub(crate) fn composition_basis(&self) -> &WorthUiCandidateCompositionBasis {
        &self.composition_basis
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationArtifact,
        WorthUiPreparedDeclarationMaterial,
        WorthUiSemanticHandoffEvidence,
    ) {
        let canonical_artifact =
            crate::facade::prepared_application_authority::WorthUiPreparedApplicationArtifact::source_backed(
                &self.candidate,
            );
        (
            canonical_artifact,
            self.declaration_material,
            self.semantic_handoff,
        )
    }

    pub(crate) fn into_replacement_parts(
        self,
    ) -> (
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationArtifact,
        WorthUiReplacementCandidate,
        WorthUiPreparedDeclarationMaterial,
        WorthUiSemanticHandoffEvidence,
    ) {
        let canonical_artifact =
            crate::facade::prepared_application_authority::WorthUiPreparedApplicationArtifact::source_backed(
                &self.candidate,
            );
        (
            canonical_artifact,
            self.candidate,
            self.declaration_material,
            self.semantic_handoff,
        )
    }
}

impl WorthUiCandidateCompositionBasis {
    pub fn candidate_basis(&self) -> WorthUiReplacementCandidateBasis {
        self.candidate
    }

    pub fn declaration_source_identity(&self) -> &WorthUiPreparedDeclarationSourceIdentity {
        &self.declaration_source
    }

    pub fn semantic_handoff(&self) -> &WorthUiSemanticHandoffEvidence {
        &self.semantic_handoff
    }
}
