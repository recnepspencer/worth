//! HTTP disposition shared by every retained Relational execution cause.
use super::super::protocol::{
    BankHttpDenial, BankHttpDenialKind as Kind, BankHttpNextAction as Next,
};
use worth_query_host::facade::primary_graph::RelationalExecutionDenialCause as Cause;
pub(super) fn denial(cause: Cause) -> BankHttpDenial {
    let (kind, next) = match cause {
        Cause::Content(cause) => return content(cause.into()),
        Cause::ContentInconsistency => (Kind::InternalDenied, Next::ContactOperator),
        Cause::StorageUnavailable(_) => (Kind::Unavailable, Next::ContactOperator),
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
        Cause::ProjectionContract(_)
        | Cause::HistoricalVisibility(_)
        | Cause::ChargedBytesOverflow
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

pub(super) fn content(
    cause: worth_query_host::facade::primary_graph::RelationalContentDenial,
) -> BankHttpDenial {
    use worth_query_host::facade::primary_graph::RelationalContentDenial as Content;
    let (kind, next) = match cause {
        Content::Execution(cause) => return denial(cause),
        Content::BackingUnavailable { .. } => (Kind::Unavailable, Next::ContactOperator),
        Content::HistoryNotResident => (Kind::Unavailable, Next::Retry),
        Content::SizeLimit { .. } => (Kind::ResourceExhausted, Next::NarrowRequest),
        Content::InputTypeMismatch | Content::UnreferencedAttachment { .. } => {
            (Kind::MalformedRequest, Next::CorrectRequest)
        }
        Content::MissingContent
        | Content::FormatBindingMissing
        | Content::FormatMismatch
        | Content::UnsupportedVersion
        | Content::InvalidFormat
        | Content::NonCanonicalContent
        | Content::IntegrityMismatch
        | Content::ContentIdentityConflict
        | Content::EncodingInconsistency
        | Content::DuplicateContentFormat
        | Content::ArithmeticOverflow => (Kind::InternalDenied, Next::ContactOperator),
    };
    BankHttpDenial::new(kind, next)
}

use worth_query_host::facade::primary_graph::{
    CompanionPreflightStop, RelationalOperationInterruption,
};
pub(super) fn companion(cause: CompanionPreflightStop) -> BankHttpDenial {
    use CompanionPreflightStop as D;
    match cause {
        D::Execution(cause) => crate::http::server::relational_execution_denial::denial(cause),
        D::Interrupted(cause) => match cause.interruption() {
            RelationalOperationInterruption::Cancelled => {
                BankHttpDenial::new(Kind::Cancelled, Next::Retry)
            }
            RelationalOperationInterruption::TimedOut => {
                BankHttpDenial::new(Kind::DeadlineExceeded, Next::Retry)
            }
        },
        D::ManagedSourceNotPrepared | D::TopologyPending => {
            BankHttpDenial::new(Kind::Unavailable, Next::Retry)
        }
        D::ManagedSourceChanged
        | D::SelectedSourceMismatch
        | D::SelectedPositionUnavailable { .. }
        | D::RegistrationChanged => BankHttpDenial::new(Kind::Stale, Next::Refresh),

        D::ForeignCell | D::WorkCounterOverflow | D::PreparationMemoryCounterOverflow => {
            BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator)
        }
    }
}
