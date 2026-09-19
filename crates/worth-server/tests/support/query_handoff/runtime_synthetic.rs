use std::sync::{atomic::AtomicUsize, Arc};
use worth_query::facade::runtime::{
    WorthQueryRuntime, WorthQueryRuntimeSupportProfile, WorthQueryWorkspace,
};
use worth_server::{
    WorthServerQueryWorkspaceBindingError, WorthServerQueryWorkspaceBindingRequest,
    WorthServerQueryWorkspaceProvider,
};

use super::runtime_aspect_contracts::query_handoff_aspect_contracts;
use super::runtime_named_read::install_requested_named_read;

mod runtime_synthetic_backend;

use runtime_synthetic_backend::TestQueryRuntimeBackend;

#[derive(Clone, Debug, Default)]
pub(crate) struct TestWorkspaceProvider;

impl WorthServerQueryWorkspaceProvider for TestWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "test-workspace-provider"
    }

    fn bind_workspace(
        &self,
        request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<WorthQueryWorkspace, WorthServerQueryWorkspaceBindingError> {
        bind_with_backend(TestQueryRuntimeBackend::default(), request)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ProfiledTestWorkspaceProvider {
    support_profile: WorthQueryRuntimeSupportProfile,
}

impl ProfiledTestWorkspaceProvider {
    pub(crate) fn new(support_profile: WorthQueryRuntimeSupportProfile) -> Self {
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
        bind_with_backend(
            TestQueryRuntimeBackend::new(self.support_profile.clone()),
            request,
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ProfiledCountingTestWorkspaceProvider {
    support_profile: WorthQueryRuntimeSupportProfile,
    attempted_writes: Arc<AtomicUsize>,
}

impl ProfiledCountingTestWorkspaceProvider {
    pub(crate) fn new(
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
        bind_with_backend(
            TestQueryRuntimeBackend::new_with_attempted_writes(
                self.support_profile.clone(),
                self.attempted_writes.clone(),
            ),
            request,
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PanicOnReadTestWorkspaceProvider;

impl WorthServerQueryWorkspaceProvider for PanicOnReadTestWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "panic-on-read-test-workspace-provider"
    }

    fn bind_workspace(
        &self,
        request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<WorthQueryWorkspace, WorthServerQueryWorkspaceBindingError> {
        bind_with_backend(
            TestQueryRuntimeBackend::new_panicking_on_live_reads(),
            request,
        )
    }
}

fn bind_with_backend(
    backend: TestQueryRuntimeBackend,
    request: &WorthServerQueryWorkspaceBindingRequest,
) -> Result<WorthQueryWorkspace, WorthServerQueryWorkspaceBindingError> {
    let workspace_id = request
        .resolved_request_context()
        .request_context()
        .workspace_target()
        .workspace_id();
    let (product_source, product_bridge) =
        worth_query::facade::consumer_kit::in_memory_test_product_world_installation().map_err(
            |error| WorthServerQueryWorkspaceBindingError::new("product_world", error.to_string()),
        )?;
    let mut workspace = WorthQueryRuntime::builder(
        worth_query::facade::consumer_kit::in_memory_test_product_world_resources(),
    )
    .aspect_contracts(query_handoff_aspect_contracts())
    .map_err(|error| {
        WorthServerQueryWorkspaceBindingError::new(
            "aspect_contracts",
            format!("failed to install query handoff aspect contracts: {error}"),
        )
    })?
    .backend(backend.with_product_source(product_source))
    .installed_product_bridge(
        product_bridge,
        worth_query::facade::runtime::WorthQueryConditionalExecutionResources::development(),
    )
    .build()
    .map_err(|error| {
        WorthServerQueryWorkspaceBindingError::new("runtime_build", format!("{error:?}"))
    })?
    .workspace(workspace_id)
    .map_err(|error| {
        WorthServerQueryWorkspaceBindingError::new("workspace_bind", format!("{error:?}"))
    })?;
    install_requested_named_read(&mut workspace, request)?;
    Ok(workspace)
}
