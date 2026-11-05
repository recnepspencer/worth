//! HTTP disposition shared by every retained Relational execution cause.
use super::super::protocol::{
    BankHttpDenial, BankHttpDenialKind as Kind, BankHttpNextAction as Next,
};
use worth_query_host::facade::primary_graph::RelationalExecutionDenialCause as Cause;
pub(super) fn denial(cause: Cause) -> BankHttpDenial {
    let (kind, next) = match cause {
        Cause::Content(cause) => return content_rejection(cause),
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

// Bank owns these Native formats; repeating a request cannot repair their contracts.
fn content_rejection(
    cause: worth_query_host::facade::primary_graph::RelationalContentRejection,
) -> BankHttpDenial {
    use worth_query_host::facade::primary_graph::RelationalContentRejection as Content;
    let (kind, next) = match cause {
        Content::BackingUnavailable { .. } | Content::HistoryNotResident => {
            (Kind::Unavailable, Next::ContactOperator)
        }
        Content::SizeLimit { .. } => (Kind::ResourceExhausted, Next::NarrowRequest),
        Content::MissingContent
        | Content::FormatBindingMissing
        | Content::FormatMismatch
        | Content::UnsupportedVersion
        | Content::InvalidFormat
        | Content::NonCanonicalContent
        | Content::IntegrityMismatch
        | Content::ContentIdentityConflict
        | Content::EncodingInconsistency
        | Content::InputTypeMismatch
        | Content::DuplicateContentFormat
        | Content::ArithmeticOverflow
        | Content::UnreferencedAttachment { .. } => (Kind::InternalDenied, Next::ContactOperator),
    };
    BankHttpDenial::new(kind, next)
}
