use worth_query::facade::consumer_kit::{in_memory_test_runtime, WorthQueryTestBackendSchema};
use worth_query::facade::runtime::WorthQueryWorkspace;
use worth_server::{
    WorthServerQueryHandoffOperation, WorthServerQueryWorkspaceBindingError,
    WorthServerQueryWorkspaceBindingRequest, WorthServerQueryWorkspaceBindingTarget,
    WorthServerQueryWorkspaceProvider,
};

use super::runtime_aspect_contracts::query_handoff_aspect_contracts;

#[derive(Clone, Debug, Default)]
pub(crate) struct RealMutationWorkspaceProvider;

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
                WorthServerQueryWorkspaceBindingTarget::QueryHandoff { operation }
                    if matches!(
                        operation.as_ref(),
                        WorthServerQueryHandoffOperation::DirectMutation { .. }
                            | WorthServerQueryHandoffOperation::QueryMutation { .. }
                    ) => {}
                target => {
                    return Err(WorthServerQueryWorkspaceBindingError::new(
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
                    WorthServerQueryWorkspaceBindingError::new(
                        "workspace_bind",
                        format!("{error:?}"),
                    )
                })
        })
    }
}

fn task_schema() -> WorthQueryTestBackendSchema {
    WorthQueryTestBackendSchema::single_collection("Task")
        .aspect_contracts(query_handoff_aspect_contracts())
        .expect("task schema aspect contracts should install")
        .aspect("identity.id", "identity.id")
        .expect("task schema identity aspect should be valid")
        .aspect("title.value", "title.value")
        .expect("task schema title aspect should be valid")
}
