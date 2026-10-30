//! Ordinary owning workflow and separate assessment courts through real native owners.
pub use super::document_retention_model as model;
pub use model::programs as program;
#[path = "workflow_owning/approval_revocation.rs"]
pub mod approval_revocation;
#[path = "workflow_owning/assessment.rs"]
pub mod assessment;
#[path = "workflow_owning/assessment_cleanup.rs"]
pub mod assessment_cleanup;
#[path = "workflow_owning/host.rs"]
pub mod host;
#[path = "workflow_owning/keyed.rs"]
pub mod keyed;

#[path = "workflow_owning/proposal_wait.rs"]
pub mod proposal_wait;
