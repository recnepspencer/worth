pub(crate) use {
    worth_query::facade::runtime::WorthQueryRuntimeSupportProfile,
    worth_query::facade::runtime::WorthQueryWorkspace,
};
pub(crate) use {
    worth_server::WorthServerQueryWorkspaceBindingError,
    worth_server::WorthServerQueryWorkspaceBindingRequest,
    worth_server::WorthServerQueryWorkspaceBindingTarget,
    worth_server::WorthServerQueryWorkspaceProvider,
};

pub(crate) use {
    worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider,
    worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::TestWorkspaceProvider,
};

#[derive(Clone, Debug)]
pub(crate) struct DeclarationAdmitsButLiveDeliveryDeniesProvider {
    pub(crate) handoff_provider: ProfiledTestWorkspaceProvider,
}

impl DeclarationAdmitsButLiveDeliveryDeniesProvider {
    pub(crate) fn new(handoff_profile: WorthQueryRuntimeSupportProfile) -> Self {
        Self {
            handoff_provider: worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider::new(handoff_profile),
        }
    }
}

impl WorthServerQueryWorkspaceProvider for DeclarationAdmitsButLiveDeliveryDeniesProvider {
    fn provider_name(&self) -> &'static str {
        "declaration-admits-but-live-delivery-denies-provider"
    }

    fn bind_workspace(
        &self,
        request: &WorthServerQueryWorkspaceBindingRequest,
    ) -> Result<WorthQueryWorkspace, WorthServerQueryWorkspaceBindingError> {
        match request.target() {
            worth_server::WorthServerQueryWorkspaceBindingTarget::DirectDeclaration { .. } => {
                TestWorkspaceProvider.bind_workspace(request)
            }
            worth_server::WorthServerQueryWorkspaceBindingTarget::QueryHandoff { .. } => {
                self.handoff_provider.bind_workspace(request)
            }
        }
    }
}
