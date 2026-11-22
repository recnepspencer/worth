//! Workflow transition fixtures have no authored Signal programme to migrate.
use super::*;
use document::schema::DocumentRetentionSchema as Schema;
use worth_query_host::facade::{
    WorthQueryApplicationHomeLocation as Location, WorthQueryApplicationHomeRequest,
};
pub(super) use document::workflow::home_transition::SourceProgram as Program;
pub(super) fn request(location: &Location) -> WorthQueryApplicationHomeRequest<'static, Schema, Program> {
    document::workflow::home_transition::source_request(location)
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
