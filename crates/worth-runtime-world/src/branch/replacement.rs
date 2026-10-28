//! Exact paired reference retirement; neither observation grants mutation.
use super::OwnerRetirementWork;
use crate::identity::{CompositeCommitIdentity, ProductBranchIncarnation};
use worth_execution::{ExecutionMemoryReservation, WorkCeilingDenial};

/// A paired retirement refused before either reference was retired.
#[derive(Debug)]
pub enum RuntimeWorldBranchReplacementDenial {
    OwnerUnavailable,
    ForeignOwner,
    SameOccurrence,
    AliasedCells,
    OldRetired,
    CandidateRetired,
    OldHeadMoved,
    CandidateHeadMoved,
    RegistryBusy,
    CellBusy,
    CustodyBusy,
    CapacityExhausted,
    SizeOverflow,
    WorkCounterOverflow,
    BackingAllocation(std::collections::TryReserveError),
    Request(WorkCeilingDenial),
}

/// The actual moved component obligations and their new vector's original
/// reservation. Removing an obligation moves its existing owner custody; it
/// does not release the vector's payer before its last backing drops.
pub struct FundedOwnerRetirementWork {
    pub(crate) work: Vec<OwnerRetirementWork>,
    pub(crate) _memory: Option<ExecutionMemoryReservation>,
}

impl std::fmt::Debug for FundedOwnerRetirementWork {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_list().entries(&self.work).finish()
    }
}

impl FundedOwnerRetirementWork {
    /// Borrow the actual pending component obligations.
    pub fn as_slice(&self) -> &[OwnerRetirementWork] {
        &self.work
    }
    /// Move one completed obligation out without allocating new backing.
    pub fn remove(&mut self, index: usize) -> OwnerRetirementWork {
        self.work.remove(index)
    }
}

/// Old-reference retirement already happened. A stop at the final original
/// request handoff does not undo it. The candidate retains its own reference;
/// this artifact neither publishes it nor transfers application permissions.
#[derive(Debug)]
#[must_use = "retirement and its cleanup custody must not be mistaken for refusal"]
pub struct RuntimeWorldRetiredReplacement {
    pub(crate) incarnation: ProductBranchIncarnation,
    pub(crate) retired_head: CompositeCommitIdentity,
    pub(crate) boundary: Option<CompositeCommitIdentity>,
    pub(crate) handoff_stop: Option<WorkCeilingDenial>,
    pub(crate) work: FundedOwnerRetirementWork,
}

impl RuntimeWorldRetiredReplacement {
    /// The exact old occurrence that was retired.
    pub const fn retired_incarnation(&self) -> ProductBranchIncarnation {
        self.incarnation
    }
    /// The old reference's actual final commit.
    pub const fn retired_head(&self) -> &CompositeCommitIdentity {
        &self.retired_head
    }
    /// A late stop from the same original request after retirement, if any.
    pub const fn handoff_stop(&self) -> Option<WorkCeilingDenial> {
        self.handoff_stop
    }
    /// Transfer every actual cleanup payload with its original backing payer.
    pub fn into_cleanup_parts(
        self,
    ) -> (
        CompositeCommitIdentity,
        Option<CompositeCommitIdentity>,
        FundedOwnerRetirementWork,
        Option<WorkCeilingDenial>,
    ) {
        (
            self.retired_head,
            self.boundary,
            self.work,
            self.handoff_stop,
        )
    }
}
