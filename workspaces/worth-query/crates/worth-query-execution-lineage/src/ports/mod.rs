//! Ports connecting primary-graph lineage, demand, and projection to higher execution layers.

mod attempt;
mod lineage;
mod query;

pub use attempt::{
    AdmittedOperationPort, ApplicationAttemptPort, BasisLeasePort, CoupledMemberPort,
    CoupledPreparationView, CurrentProofPort, StableObservedSourceFactsPort,
    TypedAdmittedOperationPort, WorkflowActorEvidenceRuntimePort,
};
pub use lineage::{
    AppliedManagedSourcePort, EmptyManagedDerivedViewPort, LineageHomeRetirementDenial,
    ManagedDerivedViewPort, PendingProducerPublicationPort, PreparingCommitPort,
    QueryGovernancePort, SourceReadClosurePort, UnpublishedIdempotencyPort,
    WorthQueryReadCleanupRuntime,
};
pub use query::{
    AppliedOutputsSelection, DiagnosticComparisonAdmissionPort, EmptyWorkflowRetentionPort,
    GraphIntegrationPort, ManagedIndexCausePort, SourceAuthorizationLocus, SourceQueryReaderPort,
    SourceReadSupportPort, WorkflowRetentionPort,
};
