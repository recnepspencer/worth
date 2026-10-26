//! The durable medium port: the one seam through which Relational's committed
//! history and branch lifecycle transitions reach durable storage.
//!
//! Relational never names a storage engine. A runtime with an attached medium
//! publishes each positioned commit's canonical artifact to it after the
//! commit is fully prepared and its position is reserved, and installs the
//! commit into live state only after the medium answers `Committed`. A fork,
//! archive or delete publishes its branch transition artifact the same way,
//! before any live effect. Live state therefore never selects a commit or a
//! branch the medium has not made durable.
//!
//! The medium's owner (Worth Store's Relational home) also owns recovery
//! sequencing: it hands the recovered checkpoint, journal tail and transition
//! tail back to Relational through
//! [`DurabilityAccess::recovery_plan_from_source`](crate::durability::access::DurabilityAccess::recovery_plan_from_source).

mod binding;
mod cold_source;
pub use cold_source::{RelationalDurableColdSource, RelationalDurableMemberLocator};
mod member_custody;
pub use member_custody::RelationalDurableMemberCustody;
mod recovery_source;
mod transition_read;
mod unresolved;
pub use transition_read::RelationalDurableTransitionReadFailure;
pub(crate) use transition_read::RelationalTransitionHistoryReader;

pub(crate) use binding::{
    RelationalDurableCommitReadmission, RelationalDurableMediumBinding, RelationalDurableStepStop,
};
pub use recovery_source::{RelationalDurableCheckpointSource, RelationalDurableRecoverySource};
pub use unresolved::{RelationalDurabilityUnresolved, RelationalDurabilityUnresolvedSubject};

use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;

/// Durable storage for Relational's canonical commit history and its branch
/// lifecycle transitions.
///
/// Implementations publish each commit artifact at exactly its position,
/// always one past the last position the medium made durable for this
/// runtime, and each transition artifact at exactly its sequence, always one
/// past the last durable sequence. The two streams are independent: a
/// transition never consumes a position. Publications are issued one at a
/// time across both.
///
/// A medium is moved into exactly one runtime, as a
/// `Box<dyn RelationalDurableMedium>`, by
/// [`RelationalRuntimeBuilder::build_durable`](crate::facade::runtime::RelationalRuntimeBuilder::build_durable).
/// Publication access stays with that owner: recovery moves it into the rebuilt
/// runtime, and a forked runtime never inherits publication access. Speculative
/// forks may carry only a read-only historical-transition reader.
pub trait RelationalDurableMedium: Send + Sync + std::fmt::Debug {
    /// The same durable journal's read-only source, available before the first
    /// publication. Each committed artifact remains readable at its position;
    /// member placement and retention stay owned by the source provider.
    fn cold_source(&self) -> std::sync::Arc<dyn RelationalDurableColdSource>;

    /// Read one historical transition for explicit replay. Implementations
    /// bound the admitted artifact size and verify its frame before returning.
    fn read_transition(
        &self,
        sequence: crate::branch::RelationalBranchTransitionSequence,
    ) -> Result<RelationalBranchTransitionArtifact, RelationalDurableTransitionReadFailure>;
    /// Make one commit artifact durable, or say exactly why not.
    ///
    /// `Committed` means the artifact is durable at its position, including an
    /// identical retry of an artifact that already was. `ProvenNoEffect` means
    /// nothing durable changed. `Indeterminate` means the publication may or
    /// may not have become durable; only the medium's own recovery can tell.
    fn publish(&self, artifact: &RelationalCanonicalCommitArtifact)
        -> RelationalDurablePublication;

    /// Make one branch transition artifact durable at its sequence, with the
    /// same answers as [`Self::publish`].
    fn publish_transition(
        &self,
        artifact: &RelationalBranchTransitionArtifact,
    ) -> RelationalDurablePublication;

    /// The in-crate test medium's durable contents behind this medium, so
    /// Relational's own durability tests can recover from the medium a
    /// runtime publishes to. Exists only in this crate's unit-test build.
    #[cfg(test)]
    #[doc(hidden)]
    fn in_crate_test_medium(
        &self,
    ) -> Option<std::sync::Arc<crate::tests::support::TestDurableMedium>> {
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

/// Why a medium-backed publication stopped before any effect.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RelationalDurablePublicationRefusal {
    /// The medium proved the publication had no durable effect.
    ProvenNoEffect { detail: String },
    /// The medium moved into the runtime recovery rebuilt, so this runtime no
    /// longer owns it and publishes nothing (D1).
    MediumMoved,
    /// The commit could not be encoded as its canonical artifact.
    Encoding(
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactEncodeDenial,
    ),
    /// The branch transition could not be encoded as its artifact.
    TransitionEncoding(
        crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifactEncodeDenial,
    ),
}

mod preparation_admission;
pub(crate) use preparation_admission::RelationalDurablePreparationAdmission;
