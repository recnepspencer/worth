pub use super::*;
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
#[path = "tests/workflow_registration.rs"]
pub mod workflow_registration;
pub use declaration::application_operation::ApplicationMutationBinding;

pub use super::super::retention_entry::DOCUMENT_IDENTITY;
pub use super::super::schema::{Document, DocumentRetentionSchema};
pub use super::super::workflow::{
    WorkflowAdvanceBinding, WorkflowAdvanceInput, WorkflowAdvanceIntent,
};

pub type WorkflowAdvanceAccepted =
    <WorkflowAdvanceBinding as ApplicationMutationBinding<DocumentRetentionSchema>>::Result;
pub type WorkflowAdvanceDenial =
    <WorkflowAdvanceBinding as ApplicationMutationBinding<DocumentRetentionSchema>>::Denial;

/// A handler an author might wrongly write for a kernel-recorded control step.
pub struct StrayAdvanceHandler;

impl primary_graph::OperationHandler<DocumentRetentionSchema, WorkflowAdvanceBinding>
    for StrayAdvanceHandler
{
    fn decide(
        &self,
        _: &WorkflowAdvanceInput,
        _: &mut primary_graph::DecisionReader<
            '_,
            '_,
            '_,
            DocumentRetentionSchema,
            WorkflowAdvanceBinding,
        >,
    ) -> primary_graph::HandlerResult<
        primary_graph::WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
        WorkflowAdvanceDenial,
    > {
        primary_graph::HandlerResult::Cancelled
    }

    fn build_candidate(
        &self,
        _: &WorkflowAdvanceInput,
        _: primary_graph::WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>,
        _: &mut primary_graph::CandidateWriter<'_, DocumentRetentionSchema, WorkflowAdvanceBinding>,
    ) -> primary_graph::HandlerResult<WorkflowAdvanceAccepted, WorkflowAdvanceDenial> {
        primary_graph::HandlerResult::Cancelled
    }
}
