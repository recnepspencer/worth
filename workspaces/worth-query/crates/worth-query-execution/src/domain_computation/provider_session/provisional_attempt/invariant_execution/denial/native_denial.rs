//! Native owner causes retained without fabricating a graph invariant verdict.
use super::{
    WorthQueryInvariantExecutionDenialKind as Kind,
    WorthQueryInvariantExecutionFailurePosture as Posture,
};
use worth_execution::{ExecutionAllocationDenial, ExecutionAllocationDenialKind, LeaseDenial};
use worth_relational::facade::mvcc::{
    RelationalOperationInterruption, RelationalTransactionStagingDenial,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum NativeDenial {
    Staging(RelationalTransactionStagingDenial),
    Allocation(ExecutionAllocationDenial),
}
impl NativeDenial {
    pub(super) fn allocation(&self) -> Option<&ExecutionAllocationDenial> {
        match self {
            Self::Staging(denial) => denial.allocation_denial(),
            Self::Allocation(denial) => Some(denial),
        }
    }
    pub(super) fn classification(&self) -> (Kind, Posture) {
        let Some(cause) = self.allocation() else {
            return (Kind::ProviderRejected, Posture::Denied);
        };
        match cause.kind() {
            ExecutionAllocationDenialKind::Cancelled => (
                Kind::RequestInterrupted(RelationalOperationInterruption::Cancelled),
                Posture::Denied,
            ),
            ExecutionAllocationDenialKind::DeadlineElapsed => (
                Kind::RequestInterrupted(RelationalOperationInterruption::TimedOut),
                Posture::Denied,
            ),
            ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(_))
            | ExecutionAllocationDenialKind::Allocator => {
                (Kind::AllocationDenied, Posture::Exhausted)
            }
            _ => (Kind::AllocationDenied, Posture::Denied),
        }
    }
}
