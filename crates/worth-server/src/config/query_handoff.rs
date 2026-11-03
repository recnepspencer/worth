use std::{fmt, sync::Arc};

use crate::query_handoff::{UnavailableWorkspaceProvider, WorthServerQueryWorkspaceProvider};

#[derive(Clone)]
pub struct WorthServerQueryHandoffConfig {
    workspace_provider: Arc<dyn WorthServerQueryWorkspaceProvider>,
    application_mutation: Option<crate::WorthServerQueryApplicationMutationBinding>,
}

impl WorthServerQueryHandoffConfig {
    pub fn builder() -> WorthServerQueryHandoffConfigBuilder {
        WorthServerQueryHandoffConfigBuilder::default()
    }

    pub fn application_mutation(
        &self,
    ) -> Option<&crate::WorthServerQueryApplicationMutationBinding> {
        self.application_mutation.as_ref()
    }

    pub fn workspace_provider(&self) -> &Arc<dyn WorthServerQueryWorkspaceProvider> {
        &self.workspace_provider
    }
}

impl fmt::Debug for WorthServerQueryHandoffConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorthServerQueryHandoffConfig")
            .field(
                "workspace_provider",
                &self.workspace_provider.provider_name(),
            )
            .finish()
    }
}

#[derive(Clone)]
pub struct WorthServerQueryHandoffConfigBuilder {
    workspace_provider: Arc<dyn WorthServerQueryWorkspaceProvider>,
    application_mutation: Option<crate::WorthServerQueryApplicationMutationBinding>,
}

impl Default for WorthServerQueryHandoffConfigBuilder {
    fn default() -> Self {
        Self {
            workspace_provider: Arc::new(UnavailableWorkspaceProvider),
            application_mutation: None,
        }
    }
}

impl fmt::Debug for WorthServerQueryHandoffConfigBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorthServerQueryHandoffConfigBuilder")
            .field(
                "workspace_provider",
                &self.workspace_provider.provider_name(),
            )
            .finish()
    }
}

impl WorthServerQueryHandoffConfigBuilder {
    pub fn with_application_mutation(
        mut self,
        binding: crate::WorthServerQueryApplicationMutationBinding,
    ) -> Self {
        self.application_mutation = Some(binding);
        self
    }

    pub fn with_workspace_provider(
        mut self,
        workspace_provider: impl WorthServerQueryWorkspaceProvider,
    ) -> Self {
        self.workspace_provider = Arc::new(workspace_provider);
        self
    }

    pub fn with_workspace_provider_arc(
        mut self,
        workspace_provider: Arc<dyn WorthServerQueryWorkspaceProvider>,
    ) -> Self {
        self.workspace_provider = workspace_provider;
        self
    }

    pub fn build(
        self,
    ) -> Result<WorthServerQueryHandoffConfig, WorthServerQueryHandoffConfigError> {
        Ok(WorthServerQueryHandoffConfig {
            workspace_provider: self.workspace_provider,
            application_mutation: self.application_mutation,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthServerQueryHandoffConfigError {}
