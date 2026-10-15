//! The real retention host with a separate governed workflow vocabulary.
use super::super::super::{host, programs::RetentionProgramP0};
use super::*;
use worth_query_host::facade::application_installation::WorthQueryWorkflowApplicationRuntime;
pub type GovernedRuntime = WorthQueryWorkflowApplicationRuntime<
    DocumentRetentionSchema,
    GovernedDocumentWorkflow,
    RetentionProgramP0,
>;
pub fn install() -> GovernedRuntime {
    retain(host::publish_on_first_program())
}
pub(super) fn retain(
    application: super::super::super::host::DocumentRetentionRuntime<RetentionProgramP0>,
) -> GovernedRuntime {
    retain_with_auth(application).0
}
pub(super) fn retain_with_auth(
    application: super::super::super::host::DocumentRetentionRuntime<RetentionProgramP0>,
) -> (
    GovernedRuntime,
    super::super::CertificationAuthenticationOwner,
) {
    let workflow = super::super::runtime::install_workflow_spec_for::<
        RetentionProgramP0,
        GovernedDocumentWorkflow,
    >(
        application.runtime().installed_schema(),
        application.installed_program(),
        super::super::runtime::workflow_resources(),
    );
    let (authentication, _clock) = super::super::install_certification_authentication_with_clock(
        application.runtime().installed_schema(),
    );
    let runtime = application
        .retain_workflow_spec(workflow, authentication.signing_owner())
        .expect("real governed vocabulary installs");
    (runtime, authentication)
}
