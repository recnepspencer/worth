//! Ports connecting primary-graph lineage, demand, and projection to higher execution layers.

mod attempt;
mod lineage;
mod query;

pub use attempt::{
    AdmittedOperationPort, ApplicationAttemptPort, BasisLeasePort, CommitAuthorityAdmissionPort,
    CoupledMemberPort, CoupledPreparationView, CurrentProofPort, StableObservedSourceFactsPort,
    TypedAdmittedOperationPort, WorkflowActorEvidenceRuntimePort,
};
pub use lineage::{
    AppliedManagedSourcePort, EmptyManagedDerivedViewPort, LineageHomeRetirementDenial,
    ManagedDerivedViewCourtDoor, ManagedDerivedViewPort, ManagedViewPublicationRequestPort,
    PendingProducerPublicationPort, PreparingCommitPort, QueryGovernancePort,
    SourceReadClosurePort, UnpublishedIdempotencyPort, WorthQueryReadCleanupRuntime,
};
pub use query::{
    AppliedOutputsSelection, DiagnosticComparisonAdmissionPort, EmptyWorkflowRetentionPort,
    GraphIntegrationPort, ManagedIndexCausePort, SourceAuthorizationLocus, SourceQueryReaderPort,
    SourceReadSupportPort, WorkflowRetentionPort,
};

#[doc(hidden)]
pub trait ProducerEditionDeclarationPort {
    type Error;

    fn build_installed_producer_edition(
        &self,
        build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    ) -> Result<
        crate::domain_computation::primary_graph::application_contribution::producer::registry::semantic_edition::InstalledProducerEdition,
        Self::Error,
    >;
}
