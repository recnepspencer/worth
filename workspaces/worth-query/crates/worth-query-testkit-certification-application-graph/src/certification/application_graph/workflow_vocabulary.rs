//! Operation-only vocabularies need no invented assessment or signed approval.
pub use super::document_retention_model::{
    host::publish_on_first_program,
    programs::RetentionProgramP0,
    schema::DocumentRetentionSchema,
    workflow::{
        WorkflowAdvanceCapability, WorkflowAdvanceInput, WorkflowAdvanceOperation,
        WorkflowDefinitionAuthoringBinding, WorkflowDefinitionAuthoringCapability,
        WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringOperation,
        WorkflowInstanceStartCapability, WorkflowInstanceStartInput,
        WorkflowInstanceStartOperation,
    },
};
pub use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowActorPolicy, ApplicationWorkflowObservationPolicy, ApplicationWorkflowSpec,
    ApplicationWorkflowSpecIdentity,
};

pub struct InformationOnlyWorkflow;
impl ApplicationWorkflowSpec for InformationOnlyWorkflow {
    type Schema = DocumentRetentionSchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("worth.query.tests.information-only.v1");
    fn observation_policy() -> ApplicationWorkflowObservationPolicy {
        ApplicationWorkflowObservationPolicy::disabled()
    }
    fn actor_policy() -> ApplicationWorkflowActorPolicy {
        ApplicationWorkflowActorPolicy::not_applicable()
    }
}
pub use worth_query_host::facade::{
    declaration::application_program::ApplicationWorkflowComponentLimits,
    domain::{
        WorthQueryApplicationWorkflowInstallationDenialKind,
        WorthQueryApplicationWorkflowResourceCeiling,
        WorthQueryApplicationWorkflowSpecInstallation,
    },
};
