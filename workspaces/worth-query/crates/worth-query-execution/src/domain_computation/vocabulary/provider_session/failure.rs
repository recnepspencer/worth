//! Moved whole from `domain_computation::provider_session::graph_provider::failure`: the items below and every impl they own.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphProviderFailure {
    detail: String,
}

impl WorthQueryGraphProviderFailure {
    pub fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}
