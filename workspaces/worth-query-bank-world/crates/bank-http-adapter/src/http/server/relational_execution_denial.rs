//! HTTP disposition shared by every retained Relational execution cause.
use super::super::protocol::{
    BankHttpDenial, BankHttpDenialKind as Kind, BankHttpNextAction as Next,
};
use worth_query_host::facade::primary_graph::RelationalExecutionDenialCause as Cause;
pub(super) fn denial(cause: Cause) -> BankHttpDenial {
    let (kind, next) = match cause {
        Cause::Cancelled => (Kind::Cancelled, Next::Retry),
        Cause::DeadlineElapsed => (Kind::DeadlineExceeded, Next::Retry),
        Cause::WorkerLimitExceedsParent
        | Cause::MemoryLimitExceedsParent
        | Cause::WorkLimitExceedsParent
        | Cause::PolicyMemoryExhausted { .. }
        | Cause::ProcessMemoryExhausted { .. }
        | Cause::DeclaredMemoryExhausted { .. }
        | Cause::WorkExhausted
        | Cause::ResultCapacityExceeded
        | Cause::ScratchCapacityExceeded => (Kind::ResourceExhausted, Next::NarrowRequest),
        Cause::NestedStopped | Cause::AllocationFailed => (Kind::Unavailable, Next::Retry),
        Cause::ChargedBytesOverflow
        | Cause::AccountingUnavailable
        | Cause::UnrelatedNestedLease
        | Cause::NoActiveExecutionScope
        | Cause::EquivalenceContractUnavailable
        | Cause::WorkCounterOverflow
        | Cause::WorkerFailed
        | Cause::UncheckedCustomKernel
        | Cause::ExpectedIdentitiesNotCanonical
        | Cause::MemoryOverflow => (Kind::InternalDenied, Next::ContactOperator),
    };
    BankHttpDenial::new(kind, next)
}
