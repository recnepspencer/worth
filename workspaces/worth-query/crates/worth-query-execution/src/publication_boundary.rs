//! Seam between execution and the publication audience.
//!
//! It carries the capability Publication uses to drive installed program
//! custody after source discovery and retained-read checks, and the workflow
//! owner vocabulary Publication alone drives: prepared definition,
//! instance, proposal and transition attempts and their adapters. Boundary
//! checks deny naming this module outside execution and Publication, and the
//! execution facade no longer carries this vocabulary:
//!
//! ```compile_fail,E0432
//! use worth_query_execution::facade::workflow_advance::WorthQueryWorkflowAdvanceAdapter;
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
        WorthQueryWorkflowInstanceAdapter,
    };
}

pub mod workflow_advance {
    pub use crate::domain_computation::primary_graph::application_attempt::WorthQueryGuardedWorkflowOperationCustody;
    pub use crate::domain_computation::primary_graph::{
        PerformedWorkflowApproval, PerformedWorkflowAssessmentEvidence,
        PerformedWorkflowTransition, PreparedWorkflowAdvance, PreparedWorkflowAssessment,
        PreparedWorkflowOperation, PublishedWorkflowInstanceRef, PublishedWorkflowProposalRef,
        RequiredWorkflowActor, RequiredWorkflowApproval, RequiredWorkflowAssessment,
        RequiredWorkflowCondition, RequiredWorkflowEvidence, RequiredWorkflowOperation,
        WorkflowApprovalDecision, WorkflowConditionOperand, WorkflowOperationAuthority,
        WorkflowOperationAuthoritySlot, WorkflowProgressOutcome, WorkflowTransitionBindingDenial,
        WorkflowTransitionPreparationDenial, WorthQueryWorkflowAdvanceAdapter,
        WorthQueryWorkflowConditionSources,
    };
}

pub mod workflow_proposal {
    pub use crate::domain_computation::primary_graph::{
        PerformedWorkflowProposal, PreparedWorkflowProposal, PublishedWorkflowInstanceRef,
        PublishedWorkflowProposalRef, WorkflowProposalBindingDenial, WorkflowProposalOutcome,
        WorkflowProposalPreparationDenial, WorthQueryWorkflowProposalAdapter,
    };
}
