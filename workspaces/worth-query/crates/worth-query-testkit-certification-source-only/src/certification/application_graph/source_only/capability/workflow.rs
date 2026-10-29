//! Capability-selected completion acceptance is independent of permission to advance.
#[path = "workflow/actor_binding.rs"]
pub mod actor_binding;
#[path = "workflow/actor_program.rs"]
pub mod actor_program;
#[path = "workflow/bindings.rs"]
pub mod bindings;
#[path = "workflow/captured_actor.rs"]
pub mod captured_actor;
#[path = "workflow/inspection.rs"]
pub mod inspection;
#[path = "workflow/program.rs"]
pub mod program;
pub use super::{grants::*, host, schema::*};
pub use worth_query_testkit_document_retention::facade::document_retention_model::{
    operator_identity::request_scope,
    workflow::{
        definition_limits, install_certification_authentication_with_clock, workflow_resources,
    },
};
pub use program::*;
pub use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*, application_installation::*,
    declaration::application_program::*, declaration::application_schema::*,
    domain::WorthQueryApplicationWorkflowSpecInstallation,
};
pub struct Spec;
impl ApplicationWorkflowSpec for Spec {
    type Schema = CapabilitySchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("capability-workflow.spec.v1");
    fn observation_policy() -> ApplicationWorkflowObservationPolicy {
        // This completion-only fixture installs no governed workflow-read binding.
        ApplicationWorkflowObservationPolicy::disabled()
    }
    fn actor_policy() -> ApplicationWorkflowActorPolicy {
        // Its real retention/start/advance grants declare no recorded-actor rule.
        ApplicationWorkflowActorPolicy::not_applicable()
    }
}
pub fn input() -> SetRetentionInput {
    SetRetentionInput {
        identity: DOCUMENT_IDENTITY.into(),
        retention_days: 18,
    }
}
pub fn advance_input() -> SetRetentionInput {
    input()
}
