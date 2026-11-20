use std::sync::Arc;

use crate::domain_computation::WorthQueryGraphProviderCallKind;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryManagedGraphCallRequest {
    kind: WorthQueryGraphProviderCallKind,
    scope_identity: Arc<str>,
}

impl WorthQueryManagedGraphCallRequest {
    pub fn new(kind: WorthQueryGraphProviderCallKind, scope_identity: impl Into<Arc<str>>) -> Self {
        Self {
            kind,
            scope_identity: scope_identity.into(),
        }
    }

    pub fn kind(&self) -> WorthQueryGraphProviderCallKind {
        self.kind
    }

    pub fn scope_identity(&self) -> &str {
        &self.scope_identity
    }
}
