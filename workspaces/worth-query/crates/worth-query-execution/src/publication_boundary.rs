//! Seam between execution and the publication audience.
//!
//! It carries the capability Publication uses to drive installed program
//! custody after source discovery and retained-read checks, and the workflow
//! owner vocabulary Publication alone drives: prepared definition,
//! instance, proposal and transition attempts and their adapters, and the
//! idempotency binding of capability workflow requests. The same capability
//! gates admitting capability access for an input a request already encoded
//! and reporting a request's own identity work. Boundary checks deny naming
//! this module outside execution and Publication, and the execution facade no
//! longer carries this vocabulary:
//!
//! ```compile_fail,E0432
//! use worth_query_execution::facade::workflow_advance::WorthQueryWorkflowAdvanceAdapter;
//! ```
//!
//! ```compile_fail,E0432
//! use worth_query_execution::facade::primary_graph::WorthQueryCapabilityWorkflowIdempotency;
//! ```

/// Compiler-visible access to program-output progression owned by the
/// publication crate. The host facade intentionally does not re-export this
/// type or its issuer.
pub struct WorthQueryProgramPublicationAccess {
    _private: (),
}

pub fn program_publication_access() -> WorthQueryProgramPublicationAccess {
    WorthQueryProgramPublicationAccess { _private: () }
}

pub mod original_read_outputs {
    pub use crate::domain_computation::primary_graph::WorthQueryOriginalReadOutputDenial;
}

pub mod document_history {
    pub use crate::domain_computation::primary_graph::application_query::{
        WorthQueryDocumentHistoryRead, WorthQueryHistoryPreviewReadAdmission,
    };
    pub use crate::domain_computation::primary_graph::WorthQueryCompletedHistoryOutputs;
    #[doc(hidden)]
    pub use crate::domain_computation::primary_graph::WorthQueryDocumentHistoryCapturePauseForTest;
    #[doc(hidden)]
    pub use crate::domain_computation::primary_graph::WorthQueryHistoryNavigationPauseForTest;
    pub use crate::domain_computation::primary_graph::WorthQueryHistoryOutputAuthority;
    pub use crate::domain_computation::primary_graph::WorthQueryHistoryOutputIntent;
    pub use crate::domain_computation::primary_graph::WorthQueryHistoryOutputNavigationAdvance;
    pub use crate::domain_computation::primary_graph::WorthQueryHistoryOutputNavigationDenial;
    pub use crate::domain_computation::primary_graph::{
        WorthQueryAdmittedHistoryProgramOutput, WorthQueryHistoryDependentOutputRead,
        WorthQueryHistoryOutputAdvanceDenial, WorthQueryHistoryOutputControlDenial,
        WorthQueryHistoryOutputDiscovery, WorthQueryHistoryOutputReadDenial,
        WorthQueryHistoryProgramOutputAdvance, WorthQueryHistoryRootOutputAdmissionDenial,
        WorthQueryHistoryRootOutputRead, WorthQuerySettledHistoryProgramOutput,
    };
    pub use crate::domain_computation::primary_graph::{
        WorthQueryDocumentHistory, WorthQueryDocumentHistoryAdoptionFailure,
        WorthQueryDocumentHistoryAttachmentFailure, WorthQueryDocumentHistoryCandidateDenial,
        WorthQueryDocumentHistoryCloseFailure, WorthQueryDocumentHistoryCloseReceipt,
        WorthQueryDocumentHistoryClosing, WorthQueryDocumentHistoryClosingDenial,
        WorthQueryDocumentHistoryDenial, WorthQueryDocumentHistoryGroupAborting,
        WorthQueryDocumentHistoryGroupAdoptionFailure, WorthQueryDocumentHistoryGroupPreparation,
        WorthQueryDocumentHistoryGroupReservation, WorthQueryDocumentHistoryNavigation,
        WorthQueryDocumentHistoryOperation, WorthQueryDocumentHistoryOperationConstraint,
        WorthQueryDocumentHistoryRecordPreparation, WorthQueryDocumentHistoryReservation,
        WorthQueryHistoryCommandContextDenial, WorthQueryHistoryContextRefreshDenial,
        WorthQueryHistoryContextRefreshLimits, WorthQueryHistoryContextRefreshPauseForTest,
        WorthQueryHistoryDescriptionContext, WorthQueryHistoryGroupMemberAdmission,
        WorthQueryHistoryNavigationCompletion, WorthQueryHistoryNavigationDenial,
        WorthQueryHistoryNavigationEligibilityDenial, WorthQueryHistoryNavigationIntent,
        WorthQueryHistoryNavigationPosture, WorthQueryHistoryNodeId,
        WorthQueryHistoryOutputProfile, WorthQueryHistoryOutputSource, WorthQueryHistoryPage,
        WorthQueryHistoryPageAssemblyPauseForTest, WorthQueryHistoryPageCursor,
        WorthQueryHistoryPageDenial, WorthQueryHistoryPageEntry, WorthQueryHistoryPageLimits,
        WorthQueryHistoryPageRow, WorthQueryHistoryParentBoundary, WorthQueryHistoryPreview,
        WorthQueryHistoryRedoChoice, WorthQueryHistoryRedoChoices, WorthQueryHistoryReleaseOutcome,
        WorthQueryHistoryRetentionChangePauseForTest, WorthQueryHistoryRetentionStatus,
        WorthQueryHistorySelection, WorthQueryHistoryStep,
    };
    pub use crate::domain_computation::primary_graph::{
        WorthQueryDocumentHistoryIdentity, WorthQueryHistoryGroupCandidateCreation,
    };
    #[doc(hidden)]
    pub use crate::domain_computation::primary_graph::{
        WorthQueryHistoryEnrollmentObservationForTest, WorthQueryHistoryEnrollmentReportForTest,
    };
}

pub mod recorded_correction {
    pub use crate::domain_computation::primary_graph::{
        WorthQueryCorrectionActionInputSelection, WorthQueryCorrectionAlternativeRef,
        WorthQueryCorrectionAlternativeResolution, WorthQueryCorrectionAlternativesContinuation,
        WorthQueryCorrectionAlternativesPage, WorthQueryCorrectionCommandKey,
        WorthQueryCorrectionInputSelection, WorthQueryCorrectionIntent,
        WorthQueryPreparedCorrectionSource, WorthQuerySelectedCorrectionAlternative,
    };
}

pub mod capability_workflow {
    pub use crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency::WorthQueryCapabilityWorkflowIdempotency;
}

pub mod signed_creation {
    pub use crate::domain_computation::primary_graph::application_attempt::WorthQuerySignedCreationAdapter;
}

pub mod workflow_definition_publication {

    pub use {
        crate::domain_computation::primary_graph::application_attempt::workflow::WorkflowDefinitionBindingDenial,
        crate::domain_computation::primary_graph::application_attempt::workflow::WorkflowDefinitionPreparationDenial,
        crate::domain_computation::primary_graph::product_operation::WorthQueryWorkflowDefinitionPublicationAdapter,
        crate::domain_computation::primary_graph::vocabulary::application_attempt::publication::PublishedWorkflowDefinitionRef,
        crate::domain_computation::primary_graph::PerformedWorkflowDefinitionPublication,
        crate::domain_computation::primary_graph::PreparedWorkflowDefinitionPublication,
        crate::domain_computation::primary_graph::WorkflowDefinitionExpectedPredecessor,
        crate::domain_computation::primary_graph::WorkflowDefinitionPublicationOutcome,
    };
}

pub mod workflow_definition_retirement {
    pub use {
        crate::domain_computation::primary_graph::application_attempt::workflow::WorkflowDefinitionPreparationDenial,
        crate::domain_computation::primary_graph::product_operation::WorthQueryWorkflowDefinitionRetirementAdapter,
        crate::domain_computation::primary_graph::vocabulary::application_attempt::publication::PublishedWorkflowDefinitionRef,
        crate::domain_computation::primary_graph::PerformedWorkflowDefinitionRetirement,
        crate::domain_computation::primary_graph::PreparedWorkflowDefinitionRetirement,
        crate::domain_computation::primary_graph::WorkflowDefinitionRetirementOutcome,
    };
}

pub mod workflow_instance {
    pub use {crate::domain_computation::primary_graph::PerformedWorkflowInstanceCancellation, crate::domain_computation::primary_graph::PerformedWorkflowInstanceStart, crate::domain_computation::primary_graph::PreparedWorkflowInstanceCancellation, crate::domain_computation::primary_graph::PreparedWorkflowInstanceStart, crate::domain_computation::primary_graph::vocabulary::application_attempt::publication::PublishedWorkflowDefinitionRef, crate::domain_computation::primary_graph::vocabulary::application_attempt::workflow_instance_program_publication::PublishedWorkflowInstanceRef, crate::domain_computation::primary_graph::RetiredWorkflowDefinitionStart, crate::domain_computation::primary_graph::SupersededWorkflowDefinitionStart, crate::domain_computation::primary_graph::WorkflowInstanceBindingDenial, crate::domain_computation::primary_graph::WorkflowInstanceCancellationOutcome, crate::domain_computation::primary_graph::WorkflowInstancePreparationDenial, crate::domain_computation::primary_graph::WorkflowInstanceStartOutcome, crate::domain_computation::primary_graph::WorthQueryWorkflowInstanceAdapter, crate::domain_computation::primary_graph::WorthQueryWorkflowInstanceDiscoveryIntent};
}

pub mod workflow_advance {
    pub use crate::domain_computation::primary_graph::application_attempt::WorthQueryGuardedWorkflowOperationCustody;
    pub use crate::domain_computation::primary_graph::application_attempt::WorthQueryWorkflowActorContext;
    pub use {crate::domain_computation::primary_graph::vocabulary::application_attempt::condition::WorkflowConditionOperand, crate::domain_computation::primary_graph::PerformedWorkflowApproval, crate::domain_computation::primary_graph::PerformedWorkflowAssessmentEvidence, crate::domain_computation::primary_graph::PerformedWorkflowTransition, crate::domain_computation::primary_graph::PreparedWorkflowAdvance, crate::domain_computation::primary_graph::PreparedWorkflowAssessment, crate::domain_computation::primary_graph::PreparedWorkflowOperation, crate::domain_computation::primary_graph::vocabulary::application_attempt::workflow_instance_program_publication::PublishedWorkflowInstanceRef, crate::domain_computation::primary_graph::vocabulary::application_attempt::reference::PublishedWorkflowProposalRef, crate::domain_computation::primary_graph::RequiredWorkflowActor, crate::domain_computation::primary_graph::RequiredWorkflowApproval, crate::domain_computation::primary_graph::RequiredWorkflowAssessment, crate::domain_computation::primary_graph::vocabulary::application_attempt::workflow_transition_program_publication::RequiredWorkflowCondition, crate::domain_computation::primary_graph::RequiredWorkflowEvidence, crate::domain_computation::primary_graph::RequiredWorkflowOperation, crate::domain_computation::primary_graph::RequiredWorkflowProposal, crate::domain_computation::primary_graph::WorkflowApprovalBodyDenial, crate::domain_computation::primary_graph::WorkflowApprovalBodyPreparation, crate::domain_computation::primary_graph::vocabulary::application_attempt::approval::WorkflowApprovalDecision, crate::domain_computation::primary_graph::WorkflowLocalActionKind, crate::domain_computation::primary_graph::WorkflowOperationAuthority, crate::domain_computation::primary_graph::WorkflowOperationAuthoritySlot, crate::domain_computation::primary_graph::WorkflowProgressOutcome, crate::domain_computation::primary_graph::WorkflowTransitionBindingDenial, crate::domain_computation::primary_graph::WorkflowTransitionPreparationDenial, crate::domain_computation::primary_graph::WorthQueryWorkflowAdvanceAdapter, crate::domain_computation::primary_graph::WorthQueryWorkflowConditionSources};
}

pub mod workflow_birth {
    pub use crate::domain_computation::primary_graph::application_attempt::{
        RequiredWorkflowBirth, WorkflowBirthAuthority, WorkflowBirthAuthoritySlot,
    };
    pub use crate::domain_computation::primary_graph::product_operation::WorthQueryWorkflowBirthAdapter;
}
pub mod workflow_succession {
    pub use crate::domain_computation::primary_graph::application_attempt::{
        RequiredWorkflowSuccession, WorkflowSuccessionAuthority, WorkflowSuccessionAuthoritySlot,
        WorkflowSuccessionTarget,
    };
    pub use crate::domain_computation::primary_graph::product_operation::WorthQueryWorkflowSuccessionAdapter;
}

pub mod workflow_proposal {
    pub use {crate::domain_computation::primary_graph::PerformedWorkflowProposal, crate::domain_computation::primary_graph::PreparedWorkflowProposal, crate::domain_computation::primary_graph::vocabulary::application_attempt::workflow_instance_program_publication::PublishedWorkflowInstanceRef, crate::domain_computation::primary_graph::vocabulary::application_attempt::reference::PublishedWorkflowProposalRef, crate::domain_computation::primary_graph::WorkflowProposalBindingDenial, crate::domain_computation::primary_graph::WorkflowProposalOutcome, crate::domain_computation::primary_graph::WorkflowProposalPreparationDenial, crate::domain_computation::primary_graph::WorthQueryWorkflowProposalAdapter};
    pub use {
        crate::domain_computation::primary_graph::PreparedWorkflowCapturedInput,
        crate::domain_computation::primary_graph::WorkflowCapturedInputContract,
        crate::domain_computation::primary_graph::WorkflowCapturedInputDenial,
        crate::domain_computation::primary_graph::WorkflowCapturedInputSource,
        crate::domain_computation::primary_graph::WorkflowCapturedInputs,
    };
}

/// Opaque original-discovery evidence for Publication's complete-set guard.
pub mod discovery_currentness {
    pub use crate::domain_computation::primary_graph::application_query::{
        WorthQueryRequiredSetCurrentnessDenial, WorthQueryRetainedQueryDependencies,
    };
}

pub mod operation_input_capture {
    pub use crate::domain_computation::primary_graph::product_operation::operation_input_capture::WorthQueryOperationInputCaptureDenial;
}

/// Native descriptive principal pin; no actor id or grant extraction.
pub mod capability_command {
    pub use crate::domain_computation::primary_graph::authenticated_principal::WorthQueryCommandPrincipalPin;
}
