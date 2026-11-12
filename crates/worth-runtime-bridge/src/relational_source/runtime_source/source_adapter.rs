//! The actual Relational source supplies its existing snapshot capability.
use super::RuntimeBridgeRelationalSource;
use crate::facade::{
    BridgeSourceAdapter, BridgeSourceCapability, BridgeSourceCapabilitySet,
    RelationalBridgeSourceError, SnapshotReadSource, TruthSnapshotIdentity, TruthSnapshotReader,
};

impl BridgeSourceAdapter for RuntimeBridgeRelationalSource {
    fn declared_capabilities(&self) -> BridgeSourceCapabilitySet {
        BridgeSourceCapabilitySet::new(vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::BranchRead,
        ])
    }
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        SnapshotReadSource::open_snapshot(self, identity, execution)
    }
}
