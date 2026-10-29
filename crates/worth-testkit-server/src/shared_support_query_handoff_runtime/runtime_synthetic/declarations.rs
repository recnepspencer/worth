pub use {std::sync::atomic::AtomicUsize, std::sync::Arc};
pub use {worth_query::facade::runtime::WorthQueryRuntime, worth_query::facade::runtime::WorthQueryRuntimeSupportProfile, worth_query::facade::runtime::WorthQueryWorkspace};
pub use {worth_server::WorthServerQueryWorkspaceBindingError, worth_server::WorthServerQueryWorkspaceBindingRequest, worth_server::WorthServerQueryWorkspaceProvider};

pub use {crate::shared_support_query_handoff_runtime::runtime_aspect_contracts::query_handoff_aspect_contracts};
pub use {crate::shared_support_query_handoff_runtime::runtime_named_read::install_requested_named_read};


pub use {crate::shared_support_query_handoff_runtime::runtime_synthetic::runtime_synthetic_backend::TestQueryRuntimeBackend};

#[derive(Clone, Debug, Default)]
pub struct TestWorkspaceProvider;

impl WorthServerQueryWorkspaceProvider for TestWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "test-workspace-provider"
    }

    fn bind_workspace(
        &self,
        request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<WorthQueryWorkspace, WorthServerQueryWorkspaceBindingError> {
        crate::shared_support_test_bootstrap_policy::with_bootstrap(|request_execution| {
            bind_with_backend(
                request_execution,
                crate::shared_support_query_handoff_runtime::runtime_synthetic::runtime_synthetic_backend::TestQueryRuntimeBackend::default(),
                request,
            )
        })
    }
}

#[derive(Clone, Debug)]
pub struct ProfiledTestWorkspaceProvider {
    pub support_profile: WorthQueryRuntimeSupportProfile,
}

impl ProfiledTestWorkspaceProvider {
    pub fn new(support_profile: WorthQueryRuntimeSupportProfile) -> Self {
        Self { support_profile }
    }
}

impl WorthServerQueryWorkspaceProvider for ProfiledTestWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "profiled-test-workspace-provider"
    }

    fn bind_workspace(
        &self,
        request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<WorthQueryWorkspace, WorthServerQueryWorkspaceBindingError> {
        crate::shared_support_test_bootstrap_policy::with_bootstrap(|request_execution| {
            bind_with_backend(
                request_execution,
                crate::shared_support_query_handoff_runtime::runtime_synthetic::runtime_synthetic_backend::TestQueryRuntimeBackend::new(self.support_profile.clone()),
                request,
            )
        })
    }
}

#[derive(Clone, Debug)]
pub struct ProfiledCountingTestWorkspaceProvider {
    pub support_profile: WorthQueryRuntimeSupportProfile,
    pub attempted_writes: Arc<AtomicUsize>,
}

impl ProfiledCountingTestWorkspaceProvider {
    pub fn new(
        support_profile: WorthQueryRuntimeSupportProfile,
        attempted_writes: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            support_profile,
            attempted_writes,
        }
    }
}

impl WorthServerQueryWorkspaceProvider for ProfiledCountingTestWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "profiled-counting-test-workspace-provider"
    }

    fn bind_workspace(
        &self,
        request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<WorthQueryWorkspace, WorthServerQueryWorkspaceBindingError> {
        crate::shared_support_test_bootstrap_policy::with_bootstrap(|request_execution| {
            bind_with_backend(
                request_execution,
                crate::shared_support_query_handoff_runtime::runtime_synthetic::runtime_synthetic_backend::TestQueryRuntimeBackend::new_with_attempted_writes(
                    self.support_profile.clone(),
                    self.attempted_writes.clone(),
                ),
                request,
            )
        })
    }
}

#[derive(Clone, Debug)]
pub struct PanicOnReadTestWorkspaceProvider;

impl WorthServerQueryWorkspaceProvider for PanicOnReadTestWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "panic-on-read-test-workspace-provider"
    }

    fn bind_workspace(
        &self,
        request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<WorthQueryWorkspace, WorthServerQueryWorkspaceBindingError> {
        crate::shared_support_test_bootstrap_policy::with_bootstrap(|request_execution| {
            bind_with_backend(
                request_execution,
                crate::shared_support_query_handoff_runtime::runtime_synthetic::runtime_synthetic_backend::TestQueryRuntimeBackend::new_panicking_on_live_reads(),
                request,
            )
        })
    }
}

pub fn bind_with_backend(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    backend: TestQueryRuntimeBackend,
    request: &WorthServerQueryWorkspaceBindingRequest,
) -> Result<WorthQueryWorkspace, WorthServerQueryWorkspaceBindingError> {
    let workspace_id = request
        .resolved_request_context()
        .request_context()
        .workspace_target()
        .workspace_id();
    let resources = worth_query::facade::consumer_kit::in_memory_test_product_world_resources();
    let (product_source, product_bridge) =
        worth_query::facade::consumer_kit::in_memory_test_product_world_installation(&resources)
            .map_err(|error| {
                worth_server::WorthServerQueryWorkspaceBindingError::new("product_world", error.to_string())
            })?;
    let mut workspace = worth_query::facade::runtime::WorthQueryRuntime::builder(resources)
        .aspect_contracts(query_handoff_aspect_contracts())
        .map_err(|error| {
            worth_server::WorthServerQueryWorkspaceBindingError::new(
                "aspect_contracts",
                format!("failed to install query handoff aspect contracts: {error}"),
            )
        })?
        .backend(backend.with_product_source(product_source))
        .installed_product_bridge(
            product_bridge,
            worth_query::facade::runtime::WorthQueryConditionalExecutionResources::development(),
        )
        .build(request_execution)
        .map_err(|error| {
            worth_server::WorthServerQueryWorkspaceBindingError::new("runtime_build", format!("{error:?}"))
        })?
        .workspace(workspace_id)
        .map_err(|error| {
            worth_server::WorthServerQueryWorkspaceBindingError::new("workspace_bind", format!("{error:?}"))
        })?;
    install_requested_named_read(request_execution, &mut workspace, request)?;
    Ok(workspace)
}