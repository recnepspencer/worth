//! Branch-local authored workflow facts and their derived execution meaning.
//!
//! This owner uses the primary Relational graph and World publication path. It
//! is intentionally separate from managed provider-read workflows.

mod approval;
mod assessment;
pub(in crate::domain_computation::primary_graph) mod definition;
pub(in crate::domain_computation::primary_graph) mod evidence_dependency;
pub(in crate::domain_computation::primary_graph) mod instance;
pub(in crate::domain_computation::primary_graph) mod proposal;
pub(in crate::domain_computation::primary_graph) mod schema;

pub(in crate::domain_computation::primary_graph) use approval::{
    visit_workflow_approval_facts, workflow_approval_fields, WorkflowApprovalMeaning,
};
pub(in crate::domain_computation::primary_graph) use assessment::{
    visit_workflow_assessment_facts, WorkflowAssessmentEvidenceMeaning,
};

pub use definition::{
    WorkflowDefinitionBindingDenial, WorkflowDefinitionPreparationDenial,
    WorthQueryWorkflowDefinitionPublicationAdapter,
};
