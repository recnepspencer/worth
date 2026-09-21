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
#[path = "application_graph/workflow_proposal.rs"]
mod workflow_proposal;
