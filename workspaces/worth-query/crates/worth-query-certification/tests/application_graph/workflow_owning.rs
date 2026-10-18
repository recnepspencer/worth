//! Ordinary owning workflow and separate assessment courts through real native owners.
use super::document_retention_model as model;
use model::programs as program;
#[path = "workflow_owning/approval_revocation.rs"]
mod approval_revocation;
#[path = "workflow_owning/assessment.rs"]
mod assessment;
#[path = "workflow_owning/assessment_cleanup.rs"]
mod assessment_cleanup;
#[path = "workflow_owning/host.rs"]
mod host;
#[path = "workflow_owning/keyed.rs"]
mod keyed;
#[path = "workflow_owning/unpublished.rs"]
mod unpublished;

#[path = "workflow_owning/proposal_wait.rs"]
mod proposal_wait;
