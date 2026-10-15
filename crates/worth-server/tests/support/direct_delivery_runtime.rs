use worth_query::facade::runtime::{WorthQueryRuntimeSupportProfile, WorthQueryWorkspace};
use worth_server::{
    WorthServerQueryWorkspaceBindingError, WorthServerQueryWorkspaceBindingRequest,
    WorthServerQueryWorkspaceBindingTarget, WorthServerQueryWorkspaceProvider,
};

use crate::shared_support_query_handoff_runtime::{ProfiledTestWorkspaceProvider, TestWorkspaceProvider};

#[derive(Clone, Debug)]
pub(crate) struct DeclarationAdmitsButLiveDeliveryDeniesProvider {
    handoff_provider: ProfiledTestWorkspaceProvider,
}

impl DeclarationAdmitsButLiveDeliveryDeniesProvider {
    pub(crate) fn new(handoff_profile: WorthQueryRuntimeSupportProfile) -> Self {
        Self {
            handoff_provider: ProfiledTestWorkspaceProvider::new(handoff_profile),
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
            WorthServerQueryWorkspaceBindingTarget::DirectDeclaration { .. } => {
                TestWorkspaceProvider.bind_workspace(request)
            }
            WorthServerQueryWorkspaceBindingTarget::QueryHandoff { .. } => {
                self.handoff_provider.bind_workspace(request)
            }
        }
    }
}
