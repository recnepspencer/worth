//! Snapshot identity and token shapes carried by the bridge.

use std::sync::Arc;

pub use crate::identity::truth::TruthSnapshotIdentity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSnapshotToken {
    snapshot_identity: TruthSnapshotIdentity,
    token_value: Arc<str>,
}

impl BridgeSnapshotToken {
    pub(crate) fn issued(
        snapshot_identity: TruthSnapshotIdentity,
        token_value: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            snapshot_identity,
            token_value: token_value.into(),
        }
    }

    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn token_value(&self) -> &str {
        self.token_value.as_ref()
    }
}
