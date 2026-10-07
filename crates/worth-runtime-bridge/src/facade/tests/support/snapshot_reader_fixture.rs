use crate::snapshot::{
    BridgeSnapshotReadError, SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadRecord,
    TruthSnapshotIdentity, TruthSnapshotReader,
};
use worth_foundational::facade::AspectValue;

#[derive(Clone)]
pub(in crate::facade::tests) struct StaticSnapshotReader;

impl TruthSnapshotReader for StaticSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a")
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        Ok(SnapshotReadPacketResult::new(
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            request
                .reads()
                .iter()
                .map(|read| {
                    SnapshotReadRecord::for_request(
                        read,
                        AspectValue::String(("fixture-value").into()),
                    )
                })
                .collect(),
        ))
    }
}

#[derive(Clone)]
pub(in crate::facade::tests) struct DriftSnapshotReader;

impl TruthSnapshotReader for DriftSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-bad")
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        StaticSnapshotReader.read_packet(request, execution)
    }
}
