//! Pure authored workflow meaning.
//!
//! This module owns no branch, principal, installed support, publication, or
//! execution state. Downstream owners may bind and publish only a validated
//! definition produced here.

mod authoring;
mod canonical;
mod identity;
mod model;
mod validation;
mod vocabulary;

#[cfg(test)]
mod tests;

pub use authoring::{
    ApplicationWorkflowApprovalNode, ApplicationWorkflowAssessmentNode,
    ApplicationWorkflowAuthoringCommand, ApplicationWorkflowAuthoringDenial,
    ApplicationWorkflowCommandAdapter, ApplicationWorkflowDefinitionBuilder,
    ApplicationWorkflowEvidenceJoinNode, ApplicationWorkflowNodeRef,
    ApplicationWorkflowOperationNode, ApplicationWorkflowTerminalNode,
};
pub use identity::{
    ApplicationWorkflowDefinitionContentIdentity, ApplicationWorkflowDefinitionIdentity,
    ApplicationWorkflowNodeIdentity, ApplicationWorkflowSpecIdentity,
};
pub use model::{
    ApplicationWorkflowConnection, ApplicationWorkflowConnectionKind,
    ApplicationWorkflowControlOutcome, ApplicationWorkflowDataFlow,
    ApplicationWorkflowDefinitionLimits, ApplicationWorkflowNode, ApplicationWorkflowNodeKind,
    AuthoredWorkflowDefinition, ValidatedWorkflowDefinition,
};
pub use validation::{
    ApplicationWorkflowValidationDenial, ApplicationWorkflowValidationDenialKind,
};
pub use vocabulary::{
    ApplicationWorkflowApprovalRef, ApplicationWorkflowAssessmentRef,
    ApplicationWorkflowOperationRef, ApplicationWorkflowSpec,
};
