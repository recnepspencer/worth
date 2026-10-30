#[allow(unused_imports)]
pub(crate) use worth_testkit_server::shared_support_compat_http_phase_three_runtime as compat_http_phase_three_runtime;
#[allow(unused_imports)]
pub(crate) use worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime;
#[allow(unused_imports)]
pub(crate) use worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy;

pub(crate) use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};

pub(crate) use serde_json::json;
pub(crate) use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
pub(crate) use {
    worth_server::WorthServerQueryHandoffDenialCode,
    worth_server::WorthServerQueryWorkspaceBindingError,
    worth_server::WorthServerQueryWorkspaceBindingRequest,
    worth_server::WorthServerQueryWorkspaceProvider,
};

pub(crate) use {
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::build_phase_three_server,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::build_phase_three_server_with_workspace_provider,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::compat_mutation_denied,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::compat_mutation_success,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::mutation_input,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::prepared_mutation_request,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::single_insert_body,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::StatefulCountingMutationWorkspaceProvider,
};

#[derive(Clone, Debug)]
pub(crate) struct BindCountingMutationWorkspaceProvider {
    pub(crate) inner: StatefulCountingMutationWorkspaceProvider,
    pub(crate) bind_count: Arc<AtomicUsize>,
}

impl BindCountingMutationWorkspaceProvider {
    pub(crate) fn new(
        support_profile: WorthQueryRuntimeSupportProfile,
        attempted_writes: Arc<AtomicUsize>,
        bind_count: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            inner: worth_testkit_server::shared_support_compat_http_phase_three_runtime::StatefulCountingMutationWorkspaceProvider::new(
                support_profile,
                attempted_writes,
            ),
            bind_count,
        }
    }
}

impl WorthServerQueryWorkspaceProvider for BindCountingMutationWorkspaceProvider {
    fn provider_name(&self) -> &'static str {
        "bind-counting-mutation-workspace-provider"
    }

    fn bind_workspace(
        &self,
        request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<
        worth_query::facade::runtime::WorthQueryWorkspace,
        WorthServerQueryWorkspaceBindingError,
    > {
        self.bind_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.inner.bind_workspace(request)
    }
}
