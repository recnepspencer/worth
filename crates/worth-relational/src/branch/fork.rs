//! The branch fork transition: a fresh branch reference sharing an exact
//! live source's immutable root.
//!
//! A fork is a durable branch transition (Milestone 2 D8). Every fallible
//! step (source validation, target preparation, head retention and basis
//! issuance) runs before the transition is published; the stale-source check
//! is repeated inside the patch-position reservation, where no commit can
//! move the source; and the target is installed only after the medium
//! answers `Committed`. A refusal or an unresolved fate installs nothing.

use crate::history::data::{BranchCreateError, CommitId};

use super::identity::RelationalBranchIdentity;
use super::reference::{RelationalBranchCellDenial, RelationalBranchReferenceObservation};
use super::RelationalBranchVersion;

#[path = "fork_port_operations.rs"]
mod port_operations;
#[path = "fork_runtime_adapters.rs"]
mod runtime_adapters;
#[path = "fork_source_validation.rs"]
mod source_validation;
#[path = "fork_target_installation.rs"]
mod target_installation;
#[path = "fork_target_preparation.rs"]
mod target_preparation;

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
    MissingArtifact,
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
}

impl super::RelationalBranchTransitionDenial for RelationalForkDenial {
    fn durable_publication_refused(
        refusal: crate::durability::medium::RelationalDurablePublicationRefusal,
    ) -> Self {
        Self::DurablePublicationRefused(refusal)
    }

    fn transition_sequence_exhausted() -> Self {
        Self::TransitionSequenceExhausted
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

use source_validation::ValidatedForkSource;
use target_preparation::PreparedForkTarget;
