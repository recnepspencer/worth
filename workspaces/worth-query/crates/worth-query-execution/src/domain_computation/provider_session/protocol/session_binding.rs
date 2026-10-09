use std::sync::Arc;

use super::WorthQueryProviderSessionToken;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthQuerySessionBinding {
    identity: Arc<str>,
    token_identity: Arc<str>,
    token_generation: u64,
    provider_identity: Arc<str>,
    provider_generation: u64,
}

impl WorthQuerySessionBinding {
    pub(super) fn seal(
        token: &WorthQueryProviderSessionToken,
        contract: &super::WorthQueryProviderExecutionPlanContract,
    ) -> Self {
        Self {
            identity: contract.resource_attempt_identity().into(),
            token_identity: token.identity().into(),
            token_generation: token.generation(),
            provider_identity: contract.provider_identity().into(),
            provider_generation: contract.provider_generation(),
        }
    }

    pub(crate) fn canonical_identity(&self) -> &str {
        &self.identity
    }

    pub(crate) fn retain_canonical_identity(&self) -> Arc<str> {
        Arc::clone(&self.identity)
    }

    pub(crate) fn token_identity(&self) -> &str {
        &self.token_identity
    }

    pub(crate) fn token_generation(&self) -> u64 {
        self.token_generation
    }

    pub(crate) fn provider_identity(&self) -> &str {
        &self.provider_identity
    }

    pub(crate) fn provider_generation(&self) -> u64 {
        self.provider_generation
    }
}
