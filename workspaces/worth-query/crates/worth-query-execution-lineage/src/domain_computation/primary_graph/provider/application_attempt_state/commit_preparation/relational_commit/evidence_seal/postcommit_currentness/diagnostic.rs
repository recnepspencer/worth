//! Public cause vocabulary omits positions already consumed by native fact lookup.
use crate::domain_computation::primary_graph::application_attempt::IndexedSelectionReobserveDenial;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RebaseVerificationReason {
    ManagedIndex(crate::domain_computation::primary_graph::application_query::managed_index::WorthQueryManagedIndexWorkDenial),
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    NativeRevisionUnavailable,
    NativeFactRevisionUnavailable,
    IndexedSelectionDenied(IndexedSelectionReobserveDenial),
    IndexedSelectionFactDenied(IndexedSelectionReobserveDenial),
    UnsupportedDecisionFact,
    NativePayloadStorage { class: worth_execution::ExecutionAllocationDenialClass, requested: Option<u64> },
    AdmissionDenied(worth_relational::facade::mvcc::CompanionPreflightStop),
}
