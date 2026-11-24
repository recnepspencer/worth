use worth_foundational::facade::{
    AspectKey, BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator, BoundaryHandle,
    FoundationalBoundaryEvidenceSourceBasis, FoundationalCommitId,
    FoundationalCommittedDeltaLocator, FoundationalCommittedDeltaLocus,
    FoundationalTransitionLocator,
};

use super::PolicyValueProvenance;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PolicyValueSourceBasis {
    VisibleReadState,
    CommitPatch {
        base_commit_id:
            worth_relational_contracts::internal_port::history::data::coordinates::CommitId,
    },
}

impl PolicyValueSourceBasis {
    pub(super) fn into_foundational_source_basis(
        self,
        provenance: PolicyValueProvenance,
        aspect_key: &AspectKey,
    ) -> FoundationalBoundaryEvidenceSourceBasis {
        match self {
            Self::VisibleReadState => visible_read_state_source_basis(provenance, aspect_key),
            Self::CommitPatch { base_commit_id } => {
                committed_delta_source_basis(base_commit_id, "base_commit_patch", aspect_key)
            }
        }
    }
}

fn visible_read_state_source_basis(
    provenance: PolicyValueProvenance,
    aspect_key: &AspectKey,
) -> FoundationalBoundaryEvidenceSourceBasis {
    FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(BoundaryArtifactLocator::new(
        BoundaryArtifactId::new(boundary_artifact_id_for_visible_state(
            provenance, aspect_key,
        )),
        BoundaryArtifactField::Basis,
    ))
}

fn committed_delta_source_basis(
    commit_id: worth_relational_contracts::internal_port::history::data::coordinates::CommitId,
    category: &str,
    aspect_key: &AspectKey,
) -> FoundationalBoundaryEvidenceSourceBasis {
    FoundationalBoundaryEvidenceSourceBasis::transition(
        FoundationalTransitionLocator::CommittedDelta(FoundationalCommittedDeltaLocator::new(
            FoundationalCommitId::new(BoundaryHandle::new(commit_id.0)),
            FoundationalCommittedDeltaLocus::new(category, aspect_key.as_str()),
        )),
    )
}

fn boundary_artifact_id_for_visible_state(
    provenance: PolicyValueProvenance,
    aspect_key: &AspectKey,
) -> u64 {
    let mut hash = 14695981039346656037_u64;
    mix_bytes(
        &mut hash,
        b"worth.relational.merge.policy_value.visible_state",
    );
    mix_bytes(&mut hash, provenance.source_basis_label().as_bytes());
    mix_bytes(&mut hash, aspect_key.as_str().as_bytes());
    hash
}

fn mix_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(1099511628211_u64);
    }
}
