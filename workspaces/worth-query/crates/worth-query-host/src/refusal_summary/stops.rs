//! Shared nested stops: request work, provider sessions, authorization,
//! projection, native execution and World catalog refusals.
//!
//! Cancellation and deadlines become `Interrupted`, storage and content faults
//! `Storage`, panics and foreign or misused authority `Internal`, and only real
//! policy or disclosure refusals `PermissionDenied`.
use worth_execution::{MapKernelStop as Kernel, WorkCeilingDenial};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationProjectionDenialKind as Projection,
    WorthQueryOperationAuthorizationDenialKind as Authorization,
    WorthQueryOperationAuthorizationRefusalKind as Policy,
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as Session, WorthQueryValueDecodeDenial as Decode,
};
use worth_relational::facade::content::RelationalContentDenial;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Native;
use worth_runtime_world::facade::CompositeHistoryCatalogDenial as Catalog;

use super::request::{
    BAD_INPUT, BUSY, CANCELLED, CAPACITY, INTERNAL, MISSING_ITEM, PERMISSION, RULE_REFUSED, STALE,
    STORAGE, TIMED_OUT,
};
use super::summary::WorthQueryRefusalSummary as Summary;

pub(super) fn kernel(stop: &Kernel) -> Summary {
    match stop {
        Kernel::Cancelled => CANCELLED,
        Kernel::DeadlineElapsed => TIMED_OUT,
        Kernel::WorkCounterOverflow | Kernel::WorkCeiling => CAPACITY,
        Kernel::NestedStopped | Kernel::AccountingUnavailable => INTERNAL,
    }
}

pub(super) fn work(cause: &WorkCeilingDenial) -> Summary {
    match cause {
        WorkCeilingDenial::Admission(_) => CAPACITY,
        WorkCeilingDenial::Stopped(stop) => kernel(stop),
        WorkCeilingDenial::Panicked => INTERNAL,
    }
}

pub(super) fn catalog(cause: &Catalog) -> Summary {
    match cause {
        Catalog::Cancelled => CANCELLED,
        Catalog::DeadlineElapsed => TIMED_OUT,
        Catalog::CommitCapacityExhausted { .. }
        | Catalog::MetadataCapacityExhausted { .. }
        | Catalog::ArithmeticOverflow
        | Catalog::DependencyCountOverflow { .. }
        | Catalog::ProtectionCountOverflow { .. } => CAPACITY,
        Catalog::InspectionFrontierChanged { .. } => STALE,
        _ => STORAGE,
    }
}

pub(super) fn native(cause: &Native) -> Summary {
    match cause {
        Native::Cancelled => CANCELLED,
        Native::DeadlineElapsed => TIMED_OUT,
        Native::StorageUnavailable { .. }
        | Native::ContentInconsistency
        | Native::Content { .. } => STORAGE,
        Native::HistoricalVisibility { .. } => BUSY,
        Native::WorkerFailed
        | Native::NestedStopped
        | Native::AccountingUnavailable
        | Native::UnrelatedNestedLease
        | Native::NoActiveExecutionScope
        | Native::EquivalenceContractUnavailable
        | Native::UncheckedCustomKernel
        | Native::ExpectedIdentitiesNotCanonical
        | Native::ProjectionContract { .. } => INTERNAL,
        _ => CAPACITY,
    }
}

pub(super) fn session(kind: &Session) -> Summary {
    match kind {
        Session::ContentInconsistency
        | Session::Content { .. }
        | Session::StorageUnavailable { .. } => STORAGE,
        Session::HistoricalVisibility { .. } | Session::StaleIncarnation { .. } => BUSY,
        Session::ExecutionResource { .. }
        | Session::ActiveSnapshotCapacityExhausted { .. }
        | Session::RetentionCapacityExhausted { .. }
        | Session::RetentionIdentityExhausted { .. }
        | Session::SnapshotIdentityExhausted { .. }
        | Session::CandidateIdentityExhausted { .. }
        | Session::IndexMaintenanceBudgetExceeded { .. }
        | Session::IndexGenerationIdentityExhausted { .. }
        | Session::SessionIdentityExhausted { .. }
        | Session::AllocationDenied { .. } => CAPACITY,
        _ => INTERNAL,
    }
}

pub(super) fn stopped(stop: &Result<Session, Control>) -> Summary {
    match stop {
        Err(Control::Cancelled) => CANCELLED,
        Err(Control::TimedOut) => TIMED_OUT,
        Ok(kind) => session(kind),
    }
}

pub(super) fn authorization(kind: &Authorization) -> Summary {
    match kind {
        Authorization::Stopped(stop) => stopped(stop),
        Authorization::Refused(refusal) => policy(refusal),
    }
}

fn policy(refusal: &Policy) -> Summary {
    match refusal {
        Policy::StaleScope => MISSING_ITEM,
        Policy::StaleInstalledSchema
        | Policy::StaleInstalledOperation
        | Policy::StaleAuthorization => STALE,
        Policy::MutationPreconditionRejected => RULE_REFUSED,
        Policy::InvalidOperationInput => BAD_INPUT,
        Policy::CanonicalWorkDenied
        | Policy::AdmissionIdentityExhausted
        | Policy::ActiveSnapshotCapacityExhausted { .. }
        | Policy::SnapshotIdentityExhausted
        | Policy::RetentionCapacityExhausted
        | Policy::RetentionIdentityExhausted => CAPACITY,
        Policy::TrustedTimeUnavailable
        | Policy::GraphWorkAdmissionUnavailable
        | Policy::ProductSecurityBasis(_) => BUSY,
        Policy::ForeignPhase
        | Policy::ForeignRuntime
        | Policy::StaleIncarnation
        | Policy::InputNotAdmitted
        | Policy::PolicyNotInstalled
        | Policy::ScopeMismatch => INTERNAL,
        _ => PERMISSION,
    }
}

pub(super) fn projection(kind: &Projection) -> Summary {
    match kind {
        Projection::ValueDecode(cause) => decode(cause),
        Projection::FieldOmitted | Projection::RelationOmitted => PERMISSION,
        Projection::DomainProjectionRejected => RULE_REFUSED,
        Projection::PersistentNamesUnavailable => BUSY,
        Projection::PersistentNameMissing | Projection::PersistentNameKindMismatch => STORAGE,
        Projection::FieldNotProjected
        | Projection::FieldContractMismatch
        | Projection::FieldTypeMismatch
        | Projection::RelationNotProjected
        | Projection::RelationContractMismatch
        | Projection::RelationCardinalityMismatch => INTERNAL,
    }
}

fn decode(cause: &Decode) -> Summary {
    match cause {
        Decode::Checkpoint(stop) => kernel(stop),
        Decode::Scope(cause) => work(cause),
        Decode::Memory(_)
        | Decode::Reservation(_)
        | Decode::AccountingOverflow
        | Decode::ContentProjectionWorkExceeded { .. }
        | Decode::ContentProjectionWorkOverflow { .. } => CAPACITY,
        Decode::Content(RelationalContentDenial::Execution(cause)) => native(cause),
        Decode::CodecRejected
        | Decode::Content(_)
        | Decode::RequiredContentFieldAbsent
        | Decode::ContentBasisUnavailable => STORAGE,
        Decode::BindingContractMismatch => INTERNAL,
    }
}
