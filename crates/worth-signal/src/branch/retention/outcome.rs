use serde::{Deserialize, Serialize};

use crate::branch::{SignalBranchTarget, SignalOwnerUnavailable};
use crate::state::{SignalBranchId, SignalSnapshotId};

use super::lease::SignalBranchRetentionLease;

/// Why one external Signal component obligation could not be opened.
///
/// Every variant is a fact about the owner, the branch, or the exact immutable
/// target the caller named. None of them is a fact about how current that
/// target is: an exact obligation over a historical target is legitimate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalBranchRetentionAcquisitionDenial {
    OwnerUnavailable(SignalOwnerUnavailable),
    OperationCapacityExhausted {
        maximum_in_flight_operations: usize,
    },
    OwnerReentry,
    /// A claimant unwound while contacting the owner; waiters were released
    /// with this typed denial and a later claimant may retry the key.
    OwnerOperationPanicked,
    ForeignBasis,
    UnknownBranch {
        branch_id: SignalBranchId,
    },
    RetiredBranch {
        branch_id: SignalBranchId,
    },
    DefinitionMismatch {
        basis_definition_basis: u64,
        runtime_definition_basis: u64,
    },
    UnavailableTarget {
        branch_id: SignalBranchId,
        snapshot_id: SignalSnapshotId,
    },
    CapacityExhausted {
        maximum_active_leases: usize,
    },
    IdentityExhausted,
}

/// Why one explicit release was refused.
///
/// Releasing twice is representationally unavailable because release consumes
/// the lease. A weak port can refuse before consumption when its owner is gone,
/// and a live owner refuses a lease issued by another runtime. Both paths return
/// the still-live lease to the caller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalBranchRetentionReleaseDenial {
    OwnerUnavailable(SignalOwnerUnavailable),
    OperationCapacityExhausted { maximum_in_flight_operations: usize },
    OwnerReentry,
    ForeignRuntime,
}

/// How the owner that issued one obligation currently stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalBranchRetentionOwnerPosture {
    Live,
    Lost,
}

/// Which terminal path one external obligation actually took.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalBranchRetentionTerminalOutcome {
    /// The owner ledger accounted for the release.
    Released,
    /// The issuing owner was already gone; the obligation ended against a
    /// ledger no live runtime can observe.
    OwnerUnavailable,
}

/// Evidence that exactly one external obligation reached a terminal state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalBranchRetentionReleaseReceipt {
    released_target: SignalBranchTarget,
    branch_id: SignalBranchId,
    outcome: SignalBranchRetentionTerminalOutcome,
    remaining_target_leases: u32,
    remaining_branch_leases: u32,
}

impl SignalBranchRetentionReleaseReceipt {
    /// The exact immutable target this obligation released, not the branch's
    /// current target.
    pub const fn released_target(&self) -> &SignalBranchTarget {
        &self.released_target
    }

    pub const fn branch_id(&self) -> SignalBranchId {
        self.branch_id
    }

    pub const fn outcome(&self) -> SignalBranchRetentionTerminalOutcome {
        self.outcome
    }

    /// Obligations still held over the same exact target after this release.
    pub const fn remaining_target_leases(&self) -> u32 {
        self.remaining_target_leases
    }

    /// Obligations still held over any target of the same branch.
    pub const fn remaining_branch_leases(&self) -> u32 {
        self.remaining_branch_leases
    }

    pub(crate) const fn owner_issued(
        released_target: SignalBranchTarget,
        branch_id: SignalBranchId,
        outcome: SignalBranchRetentionTerminalOutcome,
        remaining_target_leases: u32,
        remaining_branch_leases: u32,
    ) -> Self {
        Self {
            released_target,
            branch_id,
            outcome,
            remaining_target_leases,
            remaining_branch_leases,
        }
    }
}

/// The result of offering one lease back to a Signal runtime.
///
/// A denial hands the still-live obligation back rather than dissolving it, so
/// a caller that guessed the wrong owner has not lost its retention.
#[derive(Debug)]
pub enum SignalBranchRetentionReleaseOutcome {
    Released(SignalBranchRetentionReleaseReceipt),
    Denied {
        lease: SignalBranchRetentionLease,
        denial: SignalBranchRetentionReleaseDenial,
    },
}

/// Owner-observed terminality of external component obligations.
///
/// `explicit_releases + dropped_releases` counts obligations that reached a
/// terminal release through the owner ledger. `owner_loss_releases` counts how
/// many of those found no live owner, and `unknown_lease_defenses` counts
/// terminal attempts that found no live ledger record at all.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalBranchRetentionTerminalCounts {
    explicit_releases: u64,
    dropped_releases: u64,
    owner_loss_releases: u64,
    unknown_lease_defenses: u64,
}

impl SignalBranchRetentionTerminalCounts {
    pub const fn explicit_releases(&self) -> u64 {
        self.explicit_releases
    }

    pub const fn dropped_releases(&self) -> u64 {
        self.dropped_releases
    }

    pub const fn owner_loss_releases(&self) -> u64 {
        self.owner_loss_releases
    }

    pub const fn unknown_lease_defenses(&self) -> u64 {
        self.unknown_lease_defenses
    }

    pub const fn terminal_releases(&self) -> u64 {
        self.explicit_releases.saturating_add(self.dropped_releases)
    }

    pub(crate) const fn owner_observed(
        explicit_releases: u64,
        dropped_releases: u64,
        owner_loss_releases: u64,
        unknown_lease_defenses: u64,
    ) -> Self {
        Self {
            explicit_releases,
            dropped_releases,
            owner_loss_releases,
            unknown_lease_defenses,
        }
    }
}
