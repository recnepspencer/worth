//! Native owner causes retained without fabricating a graph invariant verdict.
use super::{
    WorthQueryInvariantExecutionDenialKind as Kind,
    WorthQueryInvariantExecutionFailurePosture as Posture,
};
use worth_execution::{
    ExecutionAllocationDenial, ExecutionAllocationDenialClass as Class,
    ExecutionPhysicalAllocationDenialKind as Physical, LeaseDenial, MapKernelStop as Stop,
    WorkCeilingDenial as Request,
};
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
        match cause.classify() {
            Class::Request(Request::Stopped(Stop::Cancelled)) => (
                Kind::RequestInterrupted(RelationalOperationInterruption::Cancelled),
                Posture::Denied,
            ),
            Class::Request(Request::Stopped(Stop::DeadlineElapsed)) => (
                Kind::RequestInterrupted(RelationalOperationInterruption::TimedOut),
                Posture::Denied,
            ),
            Class::Request(Request::Admission(LeaseDenial::MemoryExhausted(_)))
            | Class::Physical(Physical::Allocator) => (Kind::AllocationDenied, Posture::Exhausted),
            Class::Request(Request::Admission(_))
            | Class::Request(Request::Stopped(
                Stop::WorkCounterOverflow | Stop::WorkCeiling | Stop::NestedStopped,
            ))
            | Class::Request(Request::Panicked)
            | Class::Physical(_) => (Kind::AllocationDenied, Posture::Denied),
        }
    }
}
