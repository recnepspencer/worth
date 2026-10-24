//! The durable medium port: the one seam through which Relational's committed
//! history reaches durable storage.
//!
//! Relational never names a storage engine. A runtime with an attached medium
//! publishes each positioned commit's canonical artifact to it after the
//! commit is fully prepared and its position is reserved, and installs the
//! commit into live state only after the medium answers `Committed`. Live
//! state therefore never selects a commit the medium has not made durable.
//!
//! The medium's owner (Worth Store's Relational home) also owns recovery
//! sequencing: it hands the recovered checkpoint and journal tail back to
//! Relational through
//! [`DurabilityAccess::recovery_plan_from_source`](crate::durability::access::DurabilityAccess::recovery_plan_from_source).

mod binding;
mod recovery_source;

pub(crate) use binding::{RelationalDurableMediumBinding, RelationalDurableStepStop};
pub use recovery_source::{RelationalDurableCheckpointSource, RelationalDurableRecoverySource};

use crate::durability::canonical_commit_artifact::{
    RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactDigest,
};
use crate::history::data::CommitId;
use crate::publication::patch::data::PatchStreamPosition;

/// Durable storage for Relational's canonical commit history.
///
/// Implementations publish each artifact at exactly its position, which is
/// always one past the last position the medium made durable for this
/// runtime. Publications are issued one at a time.
pub trait RelationalDurableMedium: Send + Sync + std::fmt::Debug {
    /// Make one artifact durable, or say exactly why not.
    ///
    /// `Committed` means the artifact is durable at its position, including an
    /// identical retry of an artifact that already was. `ProvenNoEffect` means
    /// nothing durable changed. `Indeterminate` means the publication may or
    /// may not have become durable; only the medium's own recovery can tell.
    fn publish(&self, artifact: &RelationalCanonicalCommitArtifact)
        -> RelationalDurablePublication;

    /// The in-crate test medium behind this binding, so Relational's own
    /// durability tests can recover from the medium a runtime publishes to.
    /// Exists only in this crate's unit-test build.
    #[cfg(test)]
    #[doc(hidden)]
    fn in_crate_test_medium(&self) -> Option<&crate::tests::support::TestDurableMedium> {
        None
    }
}

/// A medium's answer to one publication.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RelationalDurablePublication {
    Committed,
    ProvenNoEffect { detail: String },
    Indeterminate { detail: String },
}

/// A commit whose durable publication neither completed nor was proven to
/// have had no effect.
///
/// The commit was not installed into live state. Its fate is decided by the
/// medium's recovery: after reopening, the commit is durable exactly when the
/// recovered journal holds this position with this digest. Until the runtime
/// is reopened it refuses every later publication with the same answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalDurabilityUnresolved {
    position: PatchStreamPosition,
    commit_id: CommitId,
    digest: RelationalCanonicalCommitArtifactDigest,
    detail: String,
}

impl RelationalDurabilityUnresolved {
    pub(crate) fn new(
        position: PatchStreamPosition,
        commit_id: CommitId,
        digest: RelationalCanonicalCommitArtifactDigest,
        detail: String,
    ) -> Self {
        Self {
            position,
            commit_id,
            digest,
            detail,
        }
    }

    /// The position whose fate is unresolved.
    pub fn position(&self) -> PatchStreamPosition {
        self.position
    }

    /// The commit identity the unresolved artifact carries.
    pub fn commit_id(&self) -> CommitId {
        self.commit_id
    }

    /// The unresolved artifact's payload digest, for resolving its fate.
    pub fn digest(&self) -> RelationalCanonicalCommitArtifactDigest {
        self.digest
    }

    /// The medium's description of why the fate is unknown.
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// Why a medium-backed publication stopped before any effect.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RelationalDurablePublicationRefusal {
    /// The medium proved the publication had no durable effect.
    ProvenNoEffect { detail: String },
    /// The commit could not be encoded as its canonical artifact.
    Encoding(
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactEncodeDenial,
    ),
}
