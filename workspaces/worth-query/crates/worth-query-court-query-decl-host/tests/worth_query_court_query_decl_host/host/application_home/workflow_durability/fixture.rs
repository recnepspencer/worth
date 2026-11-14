//! Real Home construction and fresh process-local workflow/signing installation.
use super::*;
use document::programs::WorkflowOwnerProgram as InitialProgram;
use document::schema::DocumentRetentionSchema;
use worth_query_host::facade::{
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
    WorthQueryWorkflowApplicationHome,
};
pub(super) type Home<Program = InitialProgram> =
    WorthQueryWorkflowApplicationHome<DocumentRetentionSchema, GovernedDocumentWorkflow, Program>;
pub(super) fn request(
    location: &WorthQueryApplicationHomeLocation,
) -> WorthQueryApplicationHomeRequest<'static, DocumentRetentionSchema, InitialProgram> {
    WorthQueryApplicationHomeRequest::new(
        home_location(location), document::programs::validate::<InitialProgram>(),
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new()
            .support(document::programs::validate::<super::target_program::TargetProgram>())
            .support(document::actor_generation::validated_actor_program())
            .support(document::model_lifecycle::validated_model_lifecycle_program()),
        DocumentRetentionSchema::declaration().unwrap(),
        ((),), ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        document::host::host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        document::host::seed_host,
    )
}
pub(super) fn open(
    location: &WorthQueryApplicationHomeLocation,
) -> (Home, CertificationAuthenticationOwner) {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request(location).open_in_advancement(&phase).unwrap()
    })
    .unwrap();
    let workflow = install(
        home.runtime().installed_schema(),
        home.runtime().installed_program(),
    );
    let authentication = install_certification_authentication(home.runtime().installed_schema());
    (
        home.retain_workflow_spec(workflow, authentication.signing_owner())
            .unwrap(),
        authentication,
    )
}
#[track_caller]
pub(super) fn close<P: 'static>(home: Home<P>) {
    let lease = checkpoint_capture::lease(256 << 20);
    home.close(&worth_execution::ExecutionRequest::leased(&lease))
        .unwrap()
        .reclaim();
}

pub(super) fn install<
    Program: worth_query_decl::facade::application_program::ApplicationProgramDefinition<
        DocumentRetentionSchema,
    >,
>(
    schema: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        DocumentRetentionSchema,
    >,
    program: &worth_query_host::facade::domain::WorthQueryInstalledApplicationProgram<
        DocumentRetentionSchema,
        Program,
    >,
) -> worth_query_host::facade::domain::WorthQueryInstalledApplicationWorkflowSpec<
    DocumentRetentionSchema,
    GovernedDocumentWorkflow,
> {
    use document::retention_entry::{
        DocumentRetentionQueryBinding, OccurrenceSetRetentionBinding, ReviewedSetRetentionBinding,
    };
    use worth_query_host::facade::domain::WorthQueryApplicationWorkflowSpecInstallation;
    let resources = workflow_resources();
    WorthQueryApplicationWorkflowSpecInstallation::<
        DocumentRetentionSchema,
        GovernedDocumentWorkflow,
        Program,
    >::begin(schema, program, resources)
    .operation::<WorkflowDefinitionAuthoringBinding>()
    .expect("the workflow operation is installed")
    .operation::<ReviewedSetRetentionBinding>()
    .expect("the workflow apply operation is installed")
    .operation::<OccurrenceSetRetentionBinding>()
    .expect("the workflow occurrence-only operation is installed")
    .assessment::<DocumentRetentionQueryBinding>()
    .expect("the workflow assessment is installed")
    .approval::<WorkflowApprovalCapability, WorkflowAdvanceOperation, WorkflowAdvanceInput>()
    .expect("the workflow approval capability is installed")
    .authoring_capability::<WorkflowDefinitionAuthoringCapability, WorkflowDefinitionAuthoringOperation, WorkflowDefinitionAuthoringInput>()
    .expect("the workflow authoring capability is installed")
    .instance_start_capability::<WorkflowInstanceStartCapability, WorkflowInstanceStartOperation, WorkflowInstanceStartInput>()
    .expect("the workflow instance-start capability is installed")
    .advance_capability::<WorkflowAdvanceCapability, WorkflowAdvanceOperation, WorkflowAdvanceInput>()
    .expect("the workflow advance capability is installed")
    .finish()
    .expect("the workflow vocabulary is valid")
}

pub(super) fn home_location(
    location: &WorthQueryApplicationHomeLocation,
) -> WorthQueryApplicationHomeLocation {
    match location {
        WorthQueryApplicationHomeLocation::MemoryOnly => {
            WorthQueryApplicationHomeLocation::MemoryOnly
        }
        WorthQueryApplicationHomeLocation::Directory(path) => {
            WorthQueryApplicationHomeLocation::Directory(path.clone())
        }
    }
}
