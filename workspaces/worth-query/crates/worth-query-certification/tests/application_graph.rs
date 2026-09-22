//! Certification of the application graph: what a branch's own program means
//! for the rules it actually enforces.

#[path = "application_graph/adoption.rs"]
mod adoption;
#[path = "application_graph/bounded_dimension_model.rs"]
mod bounded_dimension_model;
#[path = "application_graph/workflow.rs"]
mod workflow;
#[path = "application_graph/workflow_approval.rs"]
mod workflow_approval;
#[path = "application_graph/workflow_assessment.rs"]
mod workflow_assessment;
#[path = "application_graph/workflow_compilation.rs"]
mod workflow_compilation;
#[path = "application_graph/workflow_condition.rs"]
mod workflow_condition;
#[path = "application_graph/workflow_history_scale.rs"]
mod workflow_history_scale;
#[path = "application_graph/workflow_progress_retention.rs"]
mod workflow_progress_retention;
#[path = "application_graph/workflow_proposal.rs"]
mod workflow_proposal;
#[path = "application_graph/workflow_receipt_lifecycle.rs"]
mod workflow_receipt_lifecycle;
#[path = "application_graph/workflow_retry.rs"]
mod workflow_retry;
