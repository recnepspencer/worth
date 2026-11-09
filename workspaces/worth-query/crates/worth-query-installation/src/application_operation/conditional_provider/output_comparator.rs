//! Installed host output comparator has one explicit memory or authored posture.
use crate::domain_operation::{
    BridgeAuthoredProviderDescription, WorthQueryAuthoredHostConditionalOutputComparatorProvider,
    WorthQueryHostConditionalOutputComparatorProvider,
};
use std::sync::Arc;
pub enum WorthQueryBoundHostOutputComparator<Node> {
    Memory(Arc<dyn WorthQueryHostConditionalOutputComparatorProvider<Node>>),
    Authored(Arc<dyn WorthQueryAuthoredHostConditionalOutputComparatorProvider<Node>>),
}
impl<Node> Clone for WorthQueryBoundHostOutputComparator<Node> {
    fn clone(&self) -> Self {
        match self {
            Self::Memory(p) => Self::Memory(Arc::clone(p)),
            Self::Authored(p) => Self::Authored(Arc::clone(p)),
        }
    }
}
impl<Node: 'static> WorthQueryBoundHostOutputComparator<Node> {
    pub fn provider(&self) -> Arc<dyn WorthQueryHostConditionalOutputComparatorProvider<Node>> {
        match self {
            Self::Memory(p) => Arc::clone(p),
            Self::Authored(p) => p.clone(),
        }
    }
    pub fn authored_provider(
        &self,
    ) -> Option<Arc<dyn WorthQueryAuthoredHostConditionalOutputComparatorProvider<Node>>> {
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
