use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryBoundExecutionSnapshotIdentity(Arc<str>);

impl WorthQueryBoundExecutionSnapshotIdentity {
    pub fn capture(identity: Arc<str>) -> Self {
        Self(identity)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn description(&self) -> Arc<str> {
        Arc::clone(&self.0)
    }
}

mod court_doors_for_test;
