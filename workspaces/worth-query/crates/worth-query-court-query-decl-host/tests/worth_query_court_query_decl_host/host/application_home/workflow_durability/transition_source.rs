//! Workflow transition fixtures have no authored Signal programme to migrate.
use super::*;
use document::schema::DocumentRetentionSchema as Schema;
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::{
    WorthQueryApplicationHomeLocation as Location, WorthQueryApplicationHomeRequest,
};
pub(super) struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (document::host::RelationalRetentionContribution,);
    type Outputs =
        <document::programs::WorkflowOwnerProgram as ApplicationProgramDefinition<Schema>>::Outputs;
    type Rules =
        <document::programs::WorkflowOwnerProgram as ApplicationProgramDefinition<Schema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("durable-workflow.relational-source.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        document::programs::WorkflowOwnerProgram::feature_specs()
    }
}
pub(super) fn request(
    location: &Location,
) -> WorthQueryApplicationHomeRequest<'static, Schema, Program> {
    WorthQueryApplicationHomeRequest::new(
        home_location(location), document::programs::validate::<Program>(),
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new()
            .support(document::programs::validate::<super::target_program::TargetProgram>())
            .support(document::actor_generation::validated_actor_program())
            .support(document::model_lifecycle::validated_model_lifecycle_program()),
        Schema::declaration().unwrap(), ((),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        document::host::host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        document::host::seed_host,
    )
}
pub(super) fn open(location: &Location) -> (Home<Program>, CertificationAuthenticationOwner) {
    let lease = checkpoint_capture::lease(256 << 20);
    let home = with_bootstrap_advancement_from_request(
        worth_execution::ExecutionRequest::leased(&lease),
        |phase| request(location).open_in_advancement(&phase).unwrap(),
    )
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
