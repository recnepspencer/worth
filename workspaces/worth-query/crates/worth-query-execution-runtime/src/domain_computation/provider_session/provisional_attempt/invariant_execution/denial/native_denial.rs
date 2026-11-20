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
    PublicationReadmission(Box<worth_relational::facade::mvcc::TransactionCommitError>),
    PublicationCellRegistration(
        worth_relational::facade::mvcc::PublicationCompanionRegistrationStop,
    ),
    PublicationCompanion(worth_relational::facade::mvcc::CompanionPreflightStop),
    Staging(RelationalTransactionStagingDenial),
    Content(worth_relational::facade::content::RelationalContentDenial),
    AuthoredRetirement(Box<worth_relational::facade::publication::PublicationError>),
    Allocation(ExecutionAllocationDenial),
    RelationLaw(Box<worth_relational::facade::transactions::InvariantViolationFields>),
    ManagedSource(worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial),
}
impl NativeDenial {
    pub(super) fn allocation(&self) -> Option<&ExecutionAllocationDenial> {
        match self {
            Self::AuthoredRetirement(_)
            | Self::PublicationCompanion(_)
            | Self::PublicationReadmission(_)
            | Self::PublicationCellRegistration(_) => None,
            Self::Staging(denial) => denial.allocation_denial(),
            Self::Allocation(denial) => Some(denial),
            Self::RelationLaw(_) | Self::ManagedSource(_) | Self::Content(_) => None,
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
            Class::Request(Request::Admission(LeaseDenial::MemoryExhausted(
                worth_execution::MemoryLimitDenial::Capacity { .. },
            )))
            | Class::Physical(Physical::Allocator) => (Kind::AllocationDenied, Posture::Exhausted),
            Class::Request(Request::Admission(_))
            | Class::Request(Request::Stopped(
                Stop::WorkCounterOverflow
                | Stop::AccountingUnavailable
                | Stop::WorkCeiling
                | Stop::NestedStopped,
            ))
            | Class::Request(Request::Panicked)
            | Class::Physical(_) => (Kind::AllocationDenied, Posture::Denied),
        }
    }
}
