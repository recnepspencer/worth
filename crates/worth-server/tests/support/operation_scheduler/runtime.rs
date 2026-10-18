use worth_query::facade::runtime::{
    WorthQueryRuntimeFacadeFamily, WorthQueryRuntimeFamilySupport, WorthQueryRuntimeSupportProfile,
};
use worth_server::{
    WorthServerQueryHandoffOperation, WorthServerQueryWorkspaceBindingError,
    WorthServerQueryWorkspaceBindingRequest, WorthServerQueryWorkspaceBindingTarget,
    WorthServerQueryWorkspaceProvider,
};

#[allow(unused_imports)]
use crate::shared_support_query_handoff_runtime as query_handoff_runtime;

#[derive(Clone, Debug, Default)]
pub(crate) struct SchedulerWorkspaceProvider;

impl WorthServerQueryWorkspaceProvider for SchedulerWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "scheduler-workspace-provider"
    }

    fn bind_workspace(
        &self,
        request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<
        worth_query::facade::runtime::WorthQueryWorkspace,
        WorthServerQueryWorkspaceBindingError,
    > {
        validate_scheduler_world(request)?;
        query_handoff_runtime::TestWorkspaceProvider.bind_workspace(request)
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SelectiveSharedReadWorkspaceProvider;

impl WorthServerQueryWorkspaceProvider for SelectiveSharedReadWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "selective-shared-read-workspace-provider"
    }

    fn bind_workspace(
        &self,
        request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<
        worth_query::facade::runtime::WorthQueryWorkspace,
        WorthServerQueryWorkspaceBindingError,
    > {
        validate_scheduler_world(request)?;
        let workspace_id = workspace_id(request);
        if workspace_id == "workspace-shared-read-denied" {
            return query_handoff_runtime::ProfiledTestWorkspaceProvider::new(
                WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
                    WorthQueryRuntimeFamilySupport::unsupported(
                        WorthQueryRuntimeFacadeFamily::SharedRead,
                        "shared-read is intentionally denied for scheduler hostility coverage",
                    ),
                ),
            )
            .bind_workspace(request);
        }
        query_handoff_runtime::TestWorkspaceProvider.bind_workspace(request)
    }
}

fn validate_scheduler_world(
    request: &WorthServerQueryWorkspaceBindingRequest,
) -> Result<(), WorthServerQueryWorkspaceBindingError> {
    let workspace_id = workspace_id(request);
    let Some(binding_label) = query_handoff_binding_label(request.target()) else {
        return Ok(());
    };

    let admitted = matches!(
        (workspace_id, binding_label),
        ("workspace-42", "users.profile")
            | ("workspace-43", "users.profile")
            | ("workspace-shared-read-denied", "users.profile")
            | ("workspace-missing", "users.profile.missing")
    );
    if admitted {
        Ok(())
    } else {
        Err(WorthServerQueryWorkspaceBindingError::new(
            "scheduler_fixture_world",
            format!(
                "scheduler fixture world does not declare `{binding_label}` for workspace `{workspace_id}`"
            ),
        ))
    }
}

fn workspace_id(request: &WorthServerQueryWorkspaceBindingRequest) -> &str {
    request
        .resolved_request_context()
        .request_context()
        .workspace_target()
        .workspace_id()
}

fn query_handoff_binding_label(target: &WorthServerQueryWorkspaceBindingTarget) -> Option<&str> {
    let WorthServerQueryWorkspaceBindingTarget::QueryHandoff { operation } = target else {
        return None;
    };
    match operation.as_ref() {
        WorthServerQueryHandoffOperation::QueryRead { operation_name }
        | WorthServerQueryHandoffOperation::DirectRead { operation_name } => Some(operation_name),
        WorthServerQueryHandoffOperation::DirectState { target_label }
        | WorthServerQueryHandoffOperation::DirectInspection { target_label }
        | WorthServerQueryHandoffOperation::DirectProjection { target_label } => Some(target_label),
        WorthServerQueryHandoffOperation::DirectMutation { .. }
        | WorthServerQueryHandoffOperation::QueryMutation { .. }
        | WorthServerQueryHandoffOperation::DownstreamDelivery { .. } => None,
    }
}
