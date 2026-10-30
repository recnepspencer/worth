//! Scheduled public-path qualification for a finite, sparse 10k-node definition.

pub use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorthQueryApplicationRequestExt,
    },
    declaration::application_program::{
        ApplicationWorkflowComponentLimits, ApplicationWorkflowControlOutcome,
        ApplicationWorkflowDefinitionBuilder, ApplicationWorkflowDefinitionLimits,
    },
};
pub use worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling;
pub use worth_query_replay::facade::WorthQueryCertificationCostRuntimeExt;

pub use super::document_retention_model::{
    host::publish_on_first_program_for_workflow_scale,
    operator_identity::authenticate_operator,
    retention_entry::DOCUMENT_IDENTITY,
    workflow::{
        retain_workflow_with_resources, start_instance, ReviewedDocumentWorkflow,
        WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringIntent,
        WorkflowDefinitionAuthoringOperation,
    },
};

pub const NODES: u16 = 10_000;
pub const CONNECTIONS: u16 = NODES - 1;
pub const COMPONENTS: ApplicationWorkflowComponentLimits =
    match ApplicationWorkflowComponentLimits::new(1, 1, 1, 1, 1) {
        Some(limits) => limits,
        None => panic!("nonzero component limits"),
    };

pub fn sparse_definition(
    nodes: u16,
) -> worth_query_host::facade::declaration::application_program::AuthoredWorkflowDefinition<
    ReviewedDocumentWorkflow,
> {
    let connections = nodes - 1;
    let limits = ApplicationWorkflowDefinitionLimits::new(
        nodes,
        connections,
        connections,
        COMPONENTS,
        super::document_retention_model::host::WORKFLOW_DEFINITION_BYTES,
    )
    .expect("finite workflow-scale definition limits");
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedDocumentWorkflow>::new(
        "sparse-definition-scale",
        limits,
    )
    .expect("document definition identity");
    let mut operations = Vec::with_capacity(usize::from(connections));
    for index in 0..connections {
        operations.push(
            builder
                .operation::<WorkflowDefinitionAuthoringOperation>(
                    format!("operation-{index:05}"),
                    false,
                )
                .expect("sparse operation"),
        );
    }
    let terminal = builder.terminal("done").expect("terminal node");
    builder.start(&operations[0]);
    builder.sequence(&operations).expect("sparse sequence");
    builder.control(
        operations.last().expect("nonempty sequence"),
        ApplicationWorkflowControlOutcome::Completed,
        &terminal,
    );
    builder.finish().expect("sparse definition closes")
}
