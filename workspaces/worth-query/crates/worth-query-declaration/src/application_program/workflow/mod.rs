//! Pure authored workflow meaning.
//!
//! This module owns no branch, principal, installed support, publication, or
//! execution state. Downstream owners may bind and publish only a validated
//! definition produced here.

mod authoring;
mod canonical;
mod identity;
mod model;
mod provenance;
mod validation;
mod vocabulary;

#[cfg(test)]
mod tests;

pub use authoring::{
    ApplicationWorkflowApprovalNode, ApplicationWorkflowAssessmentNode,
    ApplicationWorkflowAuthoringCommand, ApplicationWorkflowAuthoringDenial,
    ApplicationWorkflowCommandAdapter, ApplicationWorkflowComponentBuilder,
    ApplicationWorkflowComponentInputBinding, ApplicationWorkflowComponentInputPort,
    ApplicationWorkflowComponentNodeRef, ApplicationWorkflowComponentOutputBinding,
    ApplicationWorkflowComponentOutputPort, ApplicationWorkflowComponentResource,
    ApplicationWorkflowConditionNode, ApplicationWorkflowDefinitionBuilder,
    ApplicationWorkflowEvidenceJoinNode, ApplicationWorkflowInputBinding,
    ApplicationWorkflowNodeRef, ApplicationWorkflowOperationNode, ApplicationWorkflowOutputBinding,
    ApplicationWorkflowTerminalNode, AuthoredWorkflowComponent, ExpandedWorkflowComponent,
    ExpandedWorkflowComponentInComponent,
};
pub use identity::{
    ApplicationWorkflowComponentIdentity, ApplicationWorkflowDefinitionContentIdentity,
    ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowNodeIdentity,
    ApplicationWorkflowSpecIdentity,
};
pub use model::{
    ApplicationWorkflowConnection, ApplicationWorkflowConnectionKind,
    ApplicationWorkflowControlOutcome, ApplicationWorkflowDataFlow,
    ApplicationWorkflowDefinitionLimits, ApplicationWorkflowEvidenceJoinPolicy,
    ApplicationWorkflowNode, ApplicationWorkflowNodeKind, ApplicationWorkflowRetry,
    AuthoredWorkflowDefinition, ValidatedWorkflowDefinition,
};
pub use provenance::{
    ApplicationWorkflowComponentExpansion, ApplicationWorkflowComponentPortDirection,
    ApplicationWorkflowExpandedConnectionProvenance, ApplicationWorkflowExpandedNodeProvenance,
    ApplicationWorkflowExpandedPortProvenance,
};
pub use validation::{
    ApplicationWorkflowValidationDenial, ApplicationWorkflowValidationDenialKind,
};
pub use vocabulary::{
    ApplicationWorkflowApprovalRef, ApplicationWorkflowAssessmentRef,
    ApplicationWorkflowConditionRef, ApplicationWorkflowOperationRef, ApplicationWorkflowSpec,
};
