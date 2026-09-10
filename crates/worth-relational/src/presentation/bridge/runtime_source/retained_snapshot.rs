use worth_runtime_bridge::facade::{
    BridgeSnapshotReadError, RelationalBridgeSourceError, SnapshotReadPacket,
    SnapshotReadPacketResult, TruthSnapshotIdentity, TruthSnapshotReader,
};

use crate::presentation::bridge::snapshot_reading::RuntimePublicationSnapshotReader;

use super::{RelationalBridgeObservationLease, RuntimeBridgeRelationalSource};

/// Reader and registration custody for one exact retained source observation.
///
/// This reader cannot be constructed from a snapshot descriptor. Dropping it
/// releases its Bridge registration and component retention together. A named
/// managed observer may retain the whole reader; its lease cannot be detached.
#[derive(Debug)]
pub struct RelationalBridgeRetainedSnapshot {
    reader: RuntimePublicationSnapshotReader,
    _lease: RelationalBridgeObservationLease,
}

impl RuntimeBridgeRelationalSource {
    /// Open the exact observation issued by this source (or a clone of it).
    ///
    /// Admission consumes the registration lease. On denial the unused lease
    /// is released; on success it remains held until the returned reader drops.
    /// Registry admission finishes before any snapshot data is read.
    pub fn open_retained_snapshot(
        &self,
        lease: RelationalBridgeObservationLease,
    ) -> Result<RelationalBridgeRetainedSnapshot, RelationalBridgeSourceError> {
        let selected = lease.resolve_for(&self.observation_bindings)?;
        let reader = RuntimePublicationSnapshotReader::for_observation_authority(
            self.runtime.clone(),
            lease.snapshot_identity().clone(),
            selected.observation().clone(),
            self.partition
                .as_ref()
                .map(|partition| partition.relational),
        );
        Ok(RelationalBridgeRetainedSnapshot {
            reader,
            _lease: lease,
        })
    }
}

impl TruthSnapshotReader for RelationalBridgeRetainedSnapshot {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        self.reader.snapshot_identity()
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        self.reader.read_packet(request)
    }
}
