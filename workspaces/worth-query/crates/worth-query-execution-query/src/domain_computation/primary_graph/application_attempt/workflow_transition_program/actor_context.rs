//! Original typed command locators. This carries neither an actor nor a grant.
pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::workflow_transition_program::actor_context::*;
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial;
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_program::ApplicationWorkflowSpec,
};
use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::workflow_transition_program::actor_context::{
    Command, OperationActorCommand,
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

impl WorthQueryWorkflowAdvanceAdapter {
    pub fn bind_operation_actor_context<Schema, Spec, Binding>(
        access: &worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccess,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        required: &RequiredWorkflowOperation,
    ) -> Result<Option<WorthQueryWorkflowActorContext>, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Binding: ApplicationMutationBinding<Schema>,
    {
        WorthQueryWorkflowActorContext::build::<Schema, Spec, Binding>(
            access,
            required.original_runtime(),
            installed,
            required.original_instance(),
            Command::Operation {
                required: OperationActorCommand {
                    captured_input: required.captured_input.clone(),
                    instance: required.instance(),
                    branch: required.branch(),
                    node_path: required.node_path().to_owned(),
                    operation: required.operation().to_owned(),
                    binding: required.binding().map(str::to_owned),
                    input_identity: *required.input_identity(),
                    original_sources: required.original_sources().cloned(),
                },
            },
        )
    }
}
