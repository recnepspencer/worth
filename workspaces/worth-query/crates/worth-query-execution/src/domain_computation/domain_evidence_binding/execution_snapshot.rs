use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryBoundExecutionSnapshotIdentity(Arc<str>);

impl WorthQueryBoundExecutionSnapshotIdentity {
    pub(in crate::domain_computation) fn capture(identity: Arc<str>) -> Self {
        Self(identity)
    }

    pub(in crate::domain_computation) fn as_str(&self) -> &str {
        &self.0
    }

    pub(in crate::domain_computation) fn description(&self) -> Arc<str> {
        Arc::clone(&self.0)
    }
}
