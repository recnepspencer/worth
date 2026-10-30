pub use worth_query::facade::runtime::WorthQueryWorkspace;
pub use {
    worth_query::facade::consumer_kit::in_memory_test_runtime,
    worth_query::facade::consumer_kit::WorthQueryTestBackendSchema,
};
pub use {
    worth_server::WorthServerQueryHandoffOperation,
    worth_server::WorthServerQueryWorkspaceBindingError,
    worth_server::WorthServerQueryWorkspaceBindingRequest,
    worth_server::WorthServerQueryWorkspaceBindingTarget,
    worth_server::WorthServerQueryWorkspaceProvider,
};

pub use crate::shared_support_query_handoff_runtime::runtime_aspect_contracts::query_handoff_aspect_contracts;

#[derive(Clone, Debug, Default)]
pub struct RealMutationWorkspaceProvider;

impl WorthServerQueryWorkspaceProvider for RealMutationWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "real-mutation-workspace-provider"
    }

    fn bind_workspace(
        &self,
        request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<WorthQueryWorkspace, WorthServerQueryWorkspaceBindingError> {
        crate::shared_support_test_bootstrap_policy::with_bootstrap(|request_execution| {
            match request.target() {
                worth_server::WorthServerQueryWorkspaceBindingTarget::QueryHandoff {
                    operation,
                } if matches!(
                    operation.as_ref(),
                    worth_server::WorthServerQueryHandoffOperation::DirectMutation { .. }
                        | worth_server::WorthServerQueryHandoffOperation::QueryMutation { .. }
                ) => {}
                target => {
                    return Err(worth_server::WorthServerQueryWorkspaceBindingError::new(
                    "workspace_target",
                    format!(
                        "real mutation workspace provider only supports mutation handoff targets, got {target:?}"
                    ),
                ));
                }
            }

            let workspace_id = request
                .resolved_request_context()
                .request_context()
                .workspace_target()
                .workspace_id();
            in_memory_test_runtime()
                .with_schema(task_schema())
                .workspace(request_execution, workspace_id)
                .map_err(|error| {
                    worth_server::WorthServerQueryWorkspaceBindingError::new(
                        "workspace_bind",
                        format!("{error:?}"),
                    )
                })
        })
    }
}

pub fn task_schema() -> WorthQueryTestBackendSchema {
    worth_query::facade::consumer_kit::WorthQueryTestBackendSchema::single_collection("Task")
        .aspect_contracts(query_handoff_aspect_contracts())
        .expect("task schema aspect contracts should install")
        .aspect("identity.id", "identity.id")
        .expect("task schema identity aspect should be valid")
        .aspect("title.value", "title.value")
        .expect("task schema title aspect should be valid")
}
