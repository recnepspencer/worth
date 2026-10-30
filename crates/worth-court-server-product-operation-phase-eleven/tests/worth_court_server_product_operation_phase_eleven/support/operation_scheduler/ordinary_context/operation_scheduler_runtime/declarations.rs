pub(crate) use {
    worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily,
    worth_query::facade::runtime::WorthQueryRuntimeFamilySupport,
    worth_query::facade::runtime::WorthQueryRuntimeSupportProfile,
};
pub(crate) use {
    worth_server::WorthServerQueryHandoffOperation,
    worth_server::WorthServerQueryWorkspaceBindingError,
    worth_server::WorthServerQueryWorkspaceBindingRequest,
    worth_server::WorthServerQueryWorkspaceBindingTarget,
    worth_server::WorthServerQueryWorkspaceProvider,
};

#[allow(unused_imports)]
pub(crate) use worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime;

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
        worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::TestWorkspaceProvider.bind_workspace(request)
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
            return worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider::new(
                worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
                    worth_query::facade::runtime::WorthQueryRuntimeFamilySupport::unsupported(
                        worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily::SharedRead,
                        "shared-read is intentionally denied for scheduler hostility coverage",
                    ),
                ),
            )
            .bind_workspace(request);
        }
        worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::TestWorkspaceProvider.bind_workspace(request)
    }
}

pub(crate) fn validate_scheduler_world(
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
        Err(worth_server::WorthServerQueryWorkspaceBindingError::new(
            "scheduler_fixture_world",
            format!(
                "scheduler fixture world does not declare `{binding_label}` for workspace `{workspace_id}`"
            ),
        ))
    }
}

pub(crate) fn workspace_id(request: &WorthServerQueryWorkspaceBindingRequest) -> &str {
    request
        .resolved_request_context()
        .request_context()
        .workspace_target()
        .workspace_id()
}

pub(crate) fn query_handoff_binding_label(
    target: &WorthServerQueryWorkspaceBindingTarget,
) -> Option<&str> {
    let worth_server::WorthServerQueryWorkspaceBindingTarget::QueryHandoff { operation } = target
    else {
        return None;
    };
    match operation.as_ref() {
        worth_server::WorthServerQueryHandoffOperation::QueryRead { operation_name }
        | worth_server::WorthServerQueryHandoffOperation::DirectRead { operation_name } => {
            Some(operation_name)
        }
        worth_server::WorthServerQueryHandoffOperation::DirectState { target_label }
        | worth_server::WorthServerQueryHandoffOperation::DirectInspection { target_label }
        | worth_server::WorthServerQueryHandoffOperation::DirectProjection { target_label } => {
            Some(target_label)
        }
        worth_server::WorthServerQueryHandoffOperation::DirectMutation { .. }
        | worth_server::WorthServerQueryHandoffOperation::QueryMutation { .. }
        | worth_server::WorthServerQueryHandoffOperation::DownstreamDelivery { .. } => None,
    }
}
