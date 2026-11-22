//! Principal-first workflow read denial vocabulary.
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;

/// No absence/status payload is carried by an admission refusal.
#[derive(Debug)]
pub enum WorthQueryWorkflowReadDenial {
    PolicyMismatch,
    SubscriptionCapacity,
    Work(worth_execution::WorkCeilingDenial),
    ContinuationMismatch,
    Page(worth_relational::facade::indexes::BoundedRelatedEntityPrefixDenial),
    PageShape(worth_relational::facade::indexes::BoundedRelatedEntityOrderedLookupDenial),
    Reconstruction(worth_relational::facade::indexes::DerivedIndexMaintenanceDenial),
    PinnedRead(worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial),
    Installation(worth_query_installation::facade::WorthQueryApplicationWorkflowInstallationDenial),
    Allocation(worth_execution::ExecutionAllocationDenial),
    Preparation(worth_relational::facade::mvcc::CompanionPreflightStop),
    Index(worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind),
    SelectedIndexUnavailable,
    BasisUnavailable,
    Corrupt,
    UnsupportedInstanceProtocol(u64),
    Relational(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    Phase(crate::domain_computation::primary_graph::WorthQueryForeignAdvancementPhase),
    Inspection(crate::domain_computation::primary_graph::WorthQuerySelectedProgramInspectionDenial),
    Execution(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial),
    Principal(crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial),
    Authorization(WorthQueryOperationAuthorizationDenial),
}
