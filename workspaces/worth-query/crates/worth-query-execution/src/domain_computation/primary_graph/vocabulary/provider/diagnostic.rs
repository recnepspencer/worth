//! Moved whole from `domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::diagnostic`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::application_attempt::indexed_entity_selection::IndexedSelectionReobserveDenial;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RebaseVerificationReason {
    ManagedIndex(crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial),
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    NativeRevisionUnavailable,
    NativeFactRevisionUnavailable,
    IndexedSelectionDenied(IndexedSelectionReobserveDenial),
    IndexedSelectionFactDenied(IndexedSelectionReobserveDenial),
    UnsupportedDecisionFact,
    NativePayloadStorage { class: worth_execution::ExecutionAllocationDenialClass, requested: Option<u64> },
    AdmissionDenied(worth_relational::facade::mvcc::CompanionPreflightStop),
}
