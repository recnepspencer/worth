//! A public consumer's document-retention application.
//!
//! One schema installs two commit-boundary rule contracts with distinct stable
//! identifiers. Program P0 declares `document-retention-v1`, program P1 declares
//! `document-retention-v2`, and nothing else about them differs. Everything here
//! is authored through the `worth-query-host` and `worth-query-decl` facades,
//! exactly as an external consumer would have to author it.

#[path = "document_retention_model/assessment_output.rs"]
pub mod assessment_output;
#[path = "document_retention_model/assessment_readiness.rs"]
pub mod assessment_readiness;
#[path = "document_retention_model/host.rs"]
pub mod host;
#[path = "document_retention_model/operator_identity.rs"]
pub mod operator_identity;
#[path = "document_retention_model/presented_request.rs"]
pub mod presented_request;
#[path = "document_retention_model/programs.rs"]
pub mod programs;
#[path = "document_retention_model/readback.rs"]
pub mod readback;
#[path = "document_retention_model/retention_days.rs"]
pub mod retention_days;
#[path = "document_retention_model/retention_entry.rs"]
pub mod retention_entry;
#[path = "document_retention_model/rules.rs"]
pub mod rules;
#[path = "document_retention_model/schema.rs"]
pub mod schema;
#[path = "document_retention_model/settled_verdict.rs"]
pub mod settled_verdict;
#[path = "document_retention_model/workflow.rs"]
pub mod workflow;
