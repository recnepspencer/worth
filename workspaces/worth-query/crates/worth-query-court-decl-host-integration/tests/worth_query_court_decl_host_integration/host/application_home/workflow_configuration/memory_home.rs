//! Actual public Memory Home construction, retaining every original owner.
use super::*;
use document::{
    programs::RetentionProgramP0, schema::DocumentRetentionSchema,
    workflow::ReviewedDocumentWorkflow,
};
use worth_query_host::facade::{WorthQueryApplicationHome, WorthQueryWorkflowApplicationHome};
pub(super) type MemoryHome = WorthQueryApplicationHome<DocumentRetentionSchema, RetentionProgramP0>;
pub(super) type WorkflowHome = WorthQueryWorkflowApplicationHome<
    DocumentRetentionSchema,
    ReviewedDocumentWorkflow,
    RetentionProgramP0,
>;

pub(super) fn open() -> MemoryHome {
    use worth_query_host::facade::{
        WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
    };
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::MemoryOnly,
        document::programs::validated_first_program(),
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new()
            .support(document::programs::validated_second_program())
            .support(document::actor_generation::validated_actor_program())
            .support(document::model_lifecycle::validated_model_lifecycle_program()),
        <DocumentRetentionSchema as worth_query_host::facade::declaration::application_schema::ApplicationSchema>::declaration().unwrap(),
        ((),), ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        document::host::host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        document::host::seed_host,
    );
    with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase).unwrap()
    })
    .unwrap()
}

pub(super) fn bind(home: MemoryHome) -> WorkflowHome {
    let workflow = document::workflow::install_workflow_spec(
        home.runtime().installed_schema(),
        home.runtime().installed_program(),
        document::workflow::workflow_resources(),
    );
    let (authentication, _) = document::workflow::install_certification_authentication_with_clock(
        home.runtime().installed_schema(),
    );
    home.retain_workflow_spec(workflow, authentication.signing_owner())
        .unwrap()
}
