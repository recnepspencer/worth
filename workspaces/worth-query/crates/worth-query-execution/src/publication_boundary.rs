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
    pub use crate::domain_computation::primary_graph::application_attempt::WorthQueryCapabilityWorkflowIdempotency;
}

pub mod signed_creation {
    pub use crate::domain_computation::primary_graph::application_attempt::WorthQuerySignedCreationAdapter;
}

pub mod workflow_definition_publication {

    pub use crate::domain_computation::primary_graph::{
        PerformedWorkflowDefinitionPublication, PreparedWorkflowDefinitionPublication,
        PublishedWorkflowDefinitionRef, WorkflowDefinitionBindingDenial,
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPreparationDenial,
        WorkflowDefinitionPublicationOutcome, WorthQueryWorkflowDefinitionPublicationAdapter,
    };
}

pub mod workflow_definition_retirement {
    pub use crate::domain_computation::primary_graph::{
        PerformedWorkflowDefinitionRetirement, PreparedWorkflowDefinitionRetirement,
        PublishedWorkflowDefinitionRef, WorkflowDefinitionPreparationDenial,
        WorkflowDefinitionRetirementOutcome, WorthQueryWorkflowDefinitionRetirementAdapter,
    };
}

pub mod workflow_instance {
    pub use crate::domain_computation::primary_graph::{
        PerformedWorkflowInstanceCancellation, PerformedWorkflowInstanceStart,
        PreparedWorkflowInstanceCancellation, PreparedWorkflowInstanceStart,
        PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef,
        RetiredWorkflowDefinitionStart, SupersededWorkflowDefinitionStart,
        WorkflowInstanceBindingDenial, WorkflowInstanceCancellationOutcome,
        WorkflowInstancePreparationDenial, WorkflowInstanceStartOutcome,
        WorthQueryWorkflowInstanceAdapter, WorthQueryWorkflowInstanceDiscoveryIntent,
    };
}

pub mod workflow_advance {
    pub use crate::domain_computation::primary_graph::application_attempt::WorthQueryGuardedWorkflowOperationCustody;
    pub use crate::domain_computation::primary_graph::application_attempt::WorthQueryWorkflowActorContext;
    pub use crate::domain_computation::primary_graph::{
        PerformedWorkflowApproval, PerformedWorkflowAssessmentEvidence,
        PerformedWorkflowTransition, PreparedWorkflowAdvance, PreparedWorkflowAssessment,
        PreparedWorkflowOperation, PublishedWorkflowInstanceRef, PublishedWorkflowProposalRef,
        RequiredWorkflowActor, RequiredWorkflowApproval, RequiredWorkflowAssessment,
        RequiredWorkflowCondition, RequiredWorkflowEvidence, RequiredWorkflowOperation,
        RequiredWorkflowProposal, WorkflowApprovalBodyDenial, WorkflowApprovalBodyPreparation,
        WorkflowApprovalDecision, WorkflowConditionOperand, WorkflowLocalActionKind,
        WorkflowOperationAuthority, WorkflowOperationAuthoritySlot, WorkflowProgressOutcome,
        WorkflowTransitionBindingDenial, WorkflowTransitionPreparationDenial,
        WorthQueryWorkflowAdvanceAdapter, WorthQueryWorkflowConditionSources,
    };
}

pub mod workflow_birth {
    pub use crate::domain_computation::primary_graph::application_attempt::{
        RequiredWorkflowBirth, WorkflowBirthAuthority, WorkflowBirthAuthoritySlot,
        WorthQueryWorkflowBirthAdapter,
    };
}
pub mod workflow_succession {
    pub use crate::domain_computation::primary_graph::application_attempt::{
        RequiredWorkflowSuccession, WorkflowSuccessionAuthority, WorkflowSuccessionAuthoritySlot,
        WorkflowSuccessionTarget, WorthQueryWorkflowSuccessionAdapter,
    };
}

pub mod workflow_proposal {
    pub use crate::domain_computation::primary_graph::{
        PerformedWorkflowProposal, PreparedWorkflowProposal, PublishedWorkflowInstanceRef,
        PublishedWorkflowProposalRef, WorkflowProposalBindingDenial, WorkflowProposalOutcome,
        WorkflowProposalPreparationDenial, WorthQueryWorkflowProposalAdapter,
    };
    pub use crate::domain_computation::primary_graph::{
        PreparedWorkflowCapturedInput, WorkflowCapturedInputContract, WorkflowCapturedInputDenial,
        WorkflowCapturedInputSource, WorkflowCapturedInputs,
    };
}

/// Opaque original-discovery evidence for Publication's complete-set guard.
pub mod discovery_currentness {
    pub use crate::domain_computation::primary_graph::application_query::{
        WorthQueryRequiredSetCurrentnessDenial, WorthQueryRetainedQueryDependencies,
    };
}

pub mod operation_input_capture {
    pub use crate::domain_computation::primary_graph::WorthQueryOperationInputCaptureDenial;
}

/// Native descriptive principal pin; no actor id or grant extraction.
pub mod capability_command {
    pub use crate::domain_computation::primary_graph::authenticated_principal::WorthQueryCommandPrincipalPin;
}
