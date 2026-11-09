//! Installed host output version has one explicit memory or authored posture.
use crate::domain_operation::{
    BridgeAuthoredProviderDescription, WorthQueryAuthoredHostConditionalOutputVersionProvider,
    WorthQueryHostConditionalOutputVersionProvider,
};
use std::sync::Arc;
pub enum WorthQueryBoundHostOutputVersion<Node> {
    Memory(Arc<dyn WorthQueryHostConditionalOutputVersionProvider<Node>>),
    Authored(Arc<dyn WorthQueryAuthoredHostConditionalOutputVersionProvider<Node>>),
}
impl<Node> Clone for WorthQueryBoundHostOutputVersion<Node> {
    fn clone(&self) -> Self {
        match self {
            Self::Memory(p) => Self::Memory(Arc::clone(p)),
            Self::Authored(p) => Self::Authored(Arc::clone(p)),
        }
    }
}
impl<Node: 'static> WorthQueryBoundHostOutputVersion<Node> {
    pub fn provider(&self) -> Arc<dyn WorthQueryHostConditionalOutputVersionProvider<Node>> {
        match self {
            Self::Memory(p) => Arc::clone(p),
            Self::Authored(p) => p.clone(),
        }
    }
    pub fn authored_provider(
        &self,
    ) -> Option<Arc<dyn WorthQueryAuthoredHostConditionalOutputVersionProvider<Node>>> {
        match self {
            Self::Memory(_) => None,
            Self::Authored(p) => Some(Arc::clone(p)),
        }
    }
    pub fn description(&self) -> Option<BridgeAuthoredProviderDescription> {
        match self {
            Self::Memory(_) => None,
            Self::Authored(p) => Some(p.authored_provider_description()),
        }
    }
}
