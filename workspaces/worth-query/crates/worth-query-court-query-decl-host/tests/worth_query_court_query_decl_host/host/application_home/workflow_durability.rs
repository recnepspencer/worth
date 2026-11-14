//! Quiescent workflow records recover through the real Directory Home.
use super::*;
use document::workflow::governed_read::{
    AuditRead, GovernedDocumentWorkflow, SubjectRead, WorkflowReadInput,
};
use document::{operator_identity::*, retention_entry::DOCUMENT_IDENTITY, workflow::*};
use worth_query_decl::facade::application_program::{
    ApplicationWorkflowNodeRef, ApplicationWorkflowOperationNode,
};
use worth_query_host::facade::application_discovery::*;
use worth_query_host::facade::application_entry::*;
use worth_query_testkit_document_retention::facade::document_retention_model as document;
type TargetNode = ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>;
mod assessment;
mod captured_resume;
mod definition;
mod fixture;
mod observation;
mod output_world;
mod requests;
mod resume;
mod target_program;
mod transition;
mod transition_source;
use fixture::*;
use requests::*;
