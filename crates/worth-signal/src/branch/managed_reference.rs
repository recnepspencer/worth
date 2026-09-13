use std::fmt;
use std::marker::PhantomData;
use std::sync::{Arc, Weak};

use worth_proof::{AuthorityProves, Proof, ProofMarker};

use super::owner_services::{
    SignalBranchCellIncarnation, SignalOwnerLifecycleIdentity, SignalOwnerLifecycleState,
};
use super::SignalOwnerUnavailable;
use crate::state::SignalBranchId;

worth_proof::authority_marker!(ManagedSignalBranchReferenceAuthorityMarker);

impl Clone for ManagedSignalBranchReferenceAuthorityMarker {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for ManagedSignalBranchReferenceAuthorityMarker {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ManagedSignalBranchReferenceProofMarker(PhantomData<()>);

impl ProofMarker for ManagedSignalBranchReferenceProofMarker {}

impl AuthorityProves<ManagedSignalBranchReferenceProofMarker>
    for ManagedSignalBranchReferenceAuthorityMarker
{
}

type ManagedSignalBranchReferenceProof =
    Proof<ManagedSignalBranchReferenceProofMarker, ManagedSignalBranchReferenceAuthorityMarker>;

/// A weak, owner-issued reference to one Signal branch lifecycle incarnation.
///
/// This reference identifies where the Signal owner must revalidate a branch.
/// It is not an exact basis, snapshot, retention obligation, or mutation
/// capability. Construction and raw identifiers remain private to the owner.
#[must_use = "a managed reference carries owner admission authority until it is dropped"]
pub struct ManagedSignalBranchReference {
    owner_lifecycle: Weak<SignalOwnerLifecycleState>,
    owner_runtime_instance_id: u64,
    owner_lifecycle_identity: SignalOwnerLifecycleIdentity,
    branch_id: SignalBranchId,
    cell_incarnation: SignalBranchCellIncarnation,
    _proof: ManagedSignalBranchReferenceProof,
}

/// Why a managed reference could not re-enter its issuing Signal owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ManagedSignalBranchReferenceAdmissionDenial {
    OwnerUnavailable(SignalOwnerUnavailable),
    OperationCapacityExhausted { maximum_in_flight_operations: usize },
    OwnerReentry,
    OwnerCellMisuse,
    ForeignOwner,
    BranchLifecycleEnded,
    BranchRetirementInProgress,
    BranchIncarnationReplaced,
    OwnerInvariantViolation,
}

impl Clone for ManagedSignalBranchReference {
    fn clone(&self) -> Self {
        Self {
            owner_lifecycle: self.owner_lifecycle.clone(),
            owner_runtime_instance_id: self.owner_runtime_instance_id,
            owner_lifecycle_identity: self.owner_lifecycle_identity,
            branch_id: self.branch_id,
            cell_incarnation: self.cell_incarnation,
            _proof: self._proof,
        }
    }
}

impl PartialEq for ManagedSignalBranchReference {
    fn eq(&self, other: &Self) -> bool {
        self.owner_runtime_instance_id == other.owner_runtime_instance_id
            && self.owner_lifecycle_identity == other.owner_lifecycle_identity
            && self.branch_id == other.branch_id
            && self.cell_incarnation == other.cell_incarnation
            && Weak::ptr_eq(&self.owner_lifecycle, &other.owner_lifecycle)
    }
}

impl Eq for ManagedSignalBranchReference {}

impl fmt::Debug for ManagedSignalBranchReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ManagedSignalBranchReference")
            .finish_non_exhaustive()
    }
}

impl ManagedSignalBranchReference {
    /// Whether this owner-issued branch reference names the branch carrying
    /// `basis`. This is descriptive lifecycle matching only; it does not
    /// readmit the basis or grant mutation authority.
    pub fn targets_basis(&self, basis: &super::AdmittedSignalBranchBasis) -> bool {
        self.branch_id == basis.branch_id()
    }

    #[allow(
        dead_code,
        reason = "Phase 4 basis-port issuance consumes this private owner seam"
    )]
    pub(in crate::branch) fn owner_issued(
        lifecycle: &Arc<SignalOwnerLifecycleState>,
        branch_id: SignalBranchId,
        cell_incarnation: SignalBranchCellIncarnation,
    ) -> Self {
        let authority = ManagedSignalBranchReferenceAuthorityMarker::witness();
        Self {
            owner_lifecycle: Arc::downgrade(lifecycle),
            owner_runtime_instance_id: lifecycle.owner_runtime_instance_id(),
            owner_lifecycle_identity: lifecycle.lifecycle_identity(),
            branch_id,
            cell_incarnation,
            _proof: Proof::from_authority_witness(&authority),
        }
    }

    pub(in crate::branch) fn is_bound_to(
        &self,
        lifecycle: &Arc<SignalOwnerLifecycleState>,
    ) -> bool {
        self.owner_runtime_instance_id == lifecycle.owner_runtime_instance_id()
            && self.owner_lifecycle_identity == lifecycle.lifecycle_identity()
            && Weak::ptr_eq(&self.owner_lifecycle, &Arc::downgrade(lifecycle))
    }

    #[allow(
        dead_code,
        reason = "Phase 4 owner-service methods consume this sealed target"
    )]
    pub(in crate::branch) const fn branch_id(&self) -> SignalBranchId {
        self.branch_id
    }

    #[allow(
        dead_code,
        reason = "Phase 4 owner-service methods validate this sealed target"
    )]
    pub(in crate::branch) const fn cell_incarnation(&self) -> SignalBranchCellIncarnation {
        self.cell_incarnation
    }
}
