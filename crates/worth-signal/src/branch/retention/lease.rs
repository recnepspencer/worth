use crate::branch::{SignalBranchBasisDescriptor, SignalBranchTarget};
use crate::state::SignalBranchId;

use super::accounting::SignalRetentionTerminalCause;
use super::outcome::{
    SignalBranchRetentionOwnerPosture, SignalBranchRetentionReleaseReceipt,
    SignalBranchRetentionTerminalCounts, SignalBranchRetentionTerminalOutcome,
};
use super::registry::{SignalBranchRetentionBinding, SignalBranchRetentionOwnerRelationship};

/// Owner-internal obligation that keeps one admitted basis's branch alive.
///
/// Every clone of an [`crate::branch::AdmittedSignalBranchBasis`] shares one of
/// these through the basis's `Arc`, so admitted pinning is counted once per
/// admission rather than once per holder.
#[derive(Debug)]
pub(crate) struct SignalBranchAdmissionLease {
    binding: SignalBranchRetentionBinding,
    lease_id: u64,
    branch_id: SignalBranchId,
}

#[derive(Debug)]
pub(crate) struct SignalBranchAdmissionReservation {
    binding: SignalBranchRetentionBinding,
    lease_ids: Vec<u64>,
    branch_id: SignalBranchId,
}

/// Explicit external obligation over one exact immutable Signal target.
///
/// It is deliberately not `Clone`: the obligation is the value, so it cannot be
/// duplicated and cannot be released twice. It exposes no observation or
/// mutation capability; the cloneable part is the narrow owner binding it holds
/// internally. Releasing it consumes it and yields a governed receipt; dropping
/// it takes the same exactly-once terminal path and is recorded as a dropped
/// release instead of fabricating a receipt nobody asked for.
#[derive(Debug)]
pub struct SignalBranchRetentionLease {
    descriptor: Option<SignalBranchBasisDescriptor>,
    binding: SignalBranchRetentionBinding,
    lease_id: u64,
}

impl SignalBranchAdmissionLease {
    pub(crate) const fn owner_issued(
        binding: SignalBranchRetentionBinding,
        lease_id: u64,
        branch_id: SignalBranchId,
    ) -> Self {
        Self {
            binding,
            lease_id,
            branch_id,
        }
    }

    pub(crate) const fn lease_id(&self) -> u64 {
        self.lease_id
    }

    pub(crate) fn rebind_branch(&mut self, branch_id: SignalBranchId) {
        if self.branch_id == branch_id {
            return;
        }
        self.binding
            .rebind_admitted(self.lease_id, self.branch_id, branch_id);
        self.branch_id = branch_id;
    }

    pub(crate) fn owner_identity_relationship(
        &self,
        owner: &SignalBranchRetentionBinding,
    ) -> SignalBranchRetentionOwnerRelationship {
        self.binding.owner_identity_relationship(owner)
    }
}

impl SignalBranchAdmissionReservation {
    pub(crate) fn owner_reserved(
        binding: SignalBranchRetentionBinding,
        lease_ids: Vec<u64>,
        branch_id: SignalBranchId,
    ) -> Self {
        Self {
            binding,
            lease_ids,
            branch_id,
        }
    }

    pub(crate) fn take_one(&mut self) -> SignalBranchAdmissionLease {
        let lease_id = self
            .lease_ids
            .pop()
            .expect("an admitted-output reservation converts each reserved slot once");
        self.binding
            .activate_reserved_admitted(lease_id, self.branch_id);
        SignalBranchAdmissionLease::owner_issued(self.binding.clone(), lease_id, self.branch_id)
    }

    pub(crate) fn into_one(mut self) -> SignalBranchAdmissionLease {
        let lease = self.take_one();
        debug_assert!(self.lease_ids.is_empty());
        lease
    }

    pub(crate) fn rebind_all(&mut self, branch_id: SignalBranchId) {
        self.binding
            .rebind_reserved_admitted(self.lease_ids.len(), self.branch_id, branch_id);
        self.branch_id = branch_id;
    }
}

impl Drop for SignalBranchAdmissionReservation {
    fn drop(&mut self) {
        self.binding
            .cancel_reserved_admitted_for_branch(self.lease_ids.len(), self.branch_id);
    }
}

impl Drop for SignalBranchAdmissionLease {
    fn drop(&mut self) {
        self.binding.release_admitted(self.lease_id, self.branch_id);
    }
}

impl SignalBranchRetentionLease {
    pub(crate) const fn owner_issued(
        descriptor: SignalBranchBasisDescriptor,
        binding: SignalBranchRetentionBinding,
        lease_id: u64,
    ) -> Self {
        Self {
            descriptor: Some(descriptor),
            binding,
            lease_id,
        }
    }

    /// The exact admitted description this obligation retains.
    ///
    /// It is the description as of acquisition, never the branch's current one.
    pub fn descriptor(&self) -> &SignalBranchBasisDescriptor {
        self.descriptor
            .as_ref()
            .expect("a live retention lease carries its retained descriptor")
    }

    /// The exact immutable target this obligation keeps available.
    pub fn retained_target(&self) -> &SignalBranchTarget {
        self.descriptor()
            .observation()
            .target()
            .as_basis()
            .expect("an owner-issued retention lease retains a basis target")
    }

    pub fn branch_id(&self) -> SignalBranchId {
        self.descriptor().branch_id()
    }

    /// Whether the runtime that issued this obligation is still live.
    pub fn owner_posture(&self) -> SignalBranchRetentionOwnerPosture {
        self.binding.owner_posture()
    }

    /// Terminality recorded by the owner ledger that issued this obligation.
    ///
    /// This remains readable after the issuing runtime is gone, which is how
    /// owner-loss releases stay observable.
    pub fn owner_terminal_counts(&self) -> SignalBranchRetentionTerminalCounts {
        self.binding.terminal_counts()
    }

    /// Consume this obligation and return governed evidence of its release.
    pub fn release(mut self) -> SignalBranchRetentionReleaseReceipt {
        let descriptor = self
            .descriptor
            .take()
            .expect("a retention lease can reach its terminal path only once");
        let released_target = descriptor
            .observation()
            .target()
            .as_basis()
            .expect("an owner-issued retention lease retains a basis target")
            .clone();
        let branch_id = descriptor.branch_id();
        let (posture, accounting) = self
            .binding
            .terminate_external(self.lease_id, SignalRetentionTerminalCause::ExplicitRelease);
        // An obligation the ledger no longer recognises is reported the same way
        // as one whose owner is gone: it ended, and no live owner accounted for
        // it. The ledger has already recorded the defense separately.
        let outcome = match (posture, accounting.is_some()) {
            (SignalBranchRetentionOwnerPosture::Live, true) => {
                SignalBranchRetentionTerminalOutcome::Released
            }
            _ => SignalBranchRetentionTerminalOutcome::OwnerUnavailable,
        };
        SignalBranchRetentionReleaseReceipt::owner_issued(
            released_target,
            branch_id,
            outcome,
            accounting.map_or(0, |accounting| accounting.remaining_target_leases),
            accounting.map_or(0, |accounting| accounting.remaining_branch_leases),
        )
    }

    pub(crate) fn owner_relationship(
        &self,
        owner: &SignalBranchRetentionBinding,
    ) -> SignalBranchRetentionOwnerRelationship {
        self.binding.owner_relationship(owner)
    }

    pub(crate) fn retains_live_obligation(&self) -> bool {
        self.descriptor.is_some() && self.binding.holds_external_obligation(self.lease_id)
    }
}

impl Drop for SignalBranchRetentionLease {
    fn drop(&mut self) {
        if self.descriptor.take().is_some() {
            let _ = self
                .binding
                .terminate_external(self.lease_id, SignalRetentionTerminalCause::DroppedLease);
        }
    }
}
