//! Branch fork transitions share an actual immutable source root.
//!
//! Current-source forks are durable branch transitions. Every fallible
//! step (source validation, target preparation, head retention and basis
//! issuance) runs before the transition is published; the stale-source check
//! is repeated inside the patch-position reservation, where no commit can
//! move the source; and the target is installed only after the medium
//! answers `Committed`. A refusal or an unresolved fate installs nothing.
//! Retained-source forks instead select original live immutable custody,
//! require a memory-only owner, and record distinct canonical meaning.

use crate::history::data::{BranchCreateError, CommitId};

use super::identity::RelationalBranchIdentity;
use super::reference::{RelationalBranchCellDenial, RelationalBranchReferenceObservation};
use crate::branch::version::RelationalBranchVersion;

#[path = "fork_association.rs"]
pub mod association;
pub use association::RelationalAssociatedFork;
#[path = "fork_source_validation.rs"]
pub mod source_validation;

/// Typed denials for the fork transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalForkDenial {
    SourceBranchMissing,
    SourceArchived,
    SourceDeleting,
    EmptySource,
    DuplicateTarget,
    RetiredTarget,
    ForeignRuntime,
    StaleSource,
    /// This receiver requires live immutable custody and supports no durable
    /// historical-root reopening. Refused before destination reservation.
    RetainedSourceDurabilityUnsupported,
    DurableAssociationRequiresMedium,
    MissingArtifact,
    /// Original-request or structural refusal before derived fork selection is installed.
    IndexPreparation(Box<crate::indexes::data::DerivedIndexMaintenanceDenial>),
    RootIdentityExhausted,
    ManagedSource(crate::mvcc::publication::companion::preflight_stop::CompanionPreflightStop),
    InvalidTarget(BranchCreateError),
    Cell(RelationalBranchCellDenial),
    RetentionCapacityExhausted,
    RetentionOwnerUnavailable,
    RetentionIdentityExhausted,
    RetentionInvariantViolation,
    OwnerUnavailable,
    /// The medium proved the fork had no durable effect; nothing was
    /// installed.
    DurablePublicationRefused(crate::durability::medium::RelationalDurablePublicationRefusal),
    /// No branch transition sequence remains.
    TransitionSequenceExhausted,
    /// The retained transition's concrete storage cannot be represented.
    TransitionStorageExhausted,
    /// Restored transition data differs from this source's original prefix.
    TransitionOriginalPrefixMismatch,
    /// Fork materialization accounting total could not be represented in `u64`.
    AllocationAccountingOverflow,
}

impl crate::branch::transition::durable_step::RelationalBranchTransitionDenial
    for RelationalForkDenial
{
    fn durable_publication_refused(
        refusal: crate::durability::medium::RelationalDurablePublicationRefusal,
    ) -> Self {
        Self::DurablePublicationRefused(refusal)
    }

    fn transition_sequence_exhausted() -> Self {
        Self::TransitionSequenceExhausted
    }

    fn transition_storage_exhausted() -> Self {
        Self::TransitionStorageExhausted
    }

    fn transition_original_prefix_mismatch() -> Self {
        Self::TransitionOriginalPrefixMismatch
    }
}

/// Read-only evidence returned after a successful branch-reference fork.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalForkOutcome {
    target_identity: RelationalBranchIdentity,
    source_observation: RelationalBranchReferenceObservation,
    target_observation: RelationalBranchReferenceObservation,
    fork_provenance: RelationalBranchReferenceObservation,
    source_truth_version: RelationalBranchVersion,
    target_truth_version: RelationalBranchVersion,
    shared_commit_id: Option<CommitId>,
}

impl RelationalForkOutcome {
    pub fn target_identity(&self) -> &RelationalBranchIdentity {
        &self.target_identity
    }

    pub fn source_observation(&self) -> &RelationalBranchReferenceObservation {
        &self.source_observation
    }

    pub fn target_observation(&self) -> &RelationalBranchReferenceObservation {
        &self.target_observation
    }

    pub fn fork_provenance(&self) -> &RelationalBranchReferenceObservation {
        &self.fork_provenance
    }

    pub const fn source_truth_version(&self) -> RelationalBranchVersion {
        self.source_truth_version
    }

    pub const fn target_truth_version(&self) -> RelationalBranchVersion {
        self.target_truth_version
    }

    pub const fn shared_commit_id(&self) -> Option<CommitId> {
        self.shared_commit_id
    }
}

pub struct RelationalForkOutcomeParts {
    pub target_identity: RelationalBranchIdentity,
    pub source_observation: RelationalBranchReferenceObservation,
    pub target_observation: RelationalBranchReferenceObservation,
    pub fork_provenance: RelationalBranchReferenceObservation,
    pub source_truth_version: RelationalBranchVersion,
    pub target_truth_version: RelationalBranchVersion,
    pub shared_commit_id: Option<CommitId>,
}

pub fn assemble_fork_outcome(parts: RelationalForkOutcomeParts) -> RelationalForkOutcome {
    RelationalForkOutcome {
        target_identity: parts.target_identity,
        source_observation: parts.source_observation,
        target_observation: parts.target_observation,
        fork_provenance: parts.fork_provenance,
        source_truth_version: parts.source_truth_version,
        target_truth_version: parts.target_truth_version,
        shared_commit_id: parts.shared_commit_id,
    }
}
