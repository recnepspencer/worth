use super::home_write_denial::RelationalHomeProgramWriteDenial;

/// Why a medium-backed publication stopped before any effect.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RelationalDurablePublicationRefusal {
    HomeProgramTransition(RelationalHomeProgramWriteDenial),
    /// The medium proved the publication had no durable effect.
    ProvenNoEffect { detail: String },
    /// The medium moved into the runtime recovery rebuilt, so this runtime no
    /// longer owns it and publishes nothing (D1).
    MediumMoved,
    /// The commit could not be encoded as its canonical artifact.
    Encoding(
        crate::durability::canonical_commit_artifact::encode_denial::RelationalCanonicalCommitArtifactEncodeDenial,
    ),
    /// The branch transition could not be encoded as its artifact.
    TransitionEncoding(
        crate::durability::branch_transition_artifact::encode_denial::RelationalBranchTransitionArtifactEncodeDenial,
    ),
}
