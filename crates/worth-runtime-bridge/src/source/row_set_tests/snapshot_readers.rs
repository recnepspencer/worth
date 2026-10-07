//! Snapshot responses used to prove row materialization contracts.
use crate::snapshot::{
    SnapshotReadPacket, SnapshotReadPacketResult, TruthSnapshotIdentity, TruthSnapshotReader,
};
use worth_foundational::facade::AspectValue;

#[derive(Debug)]
pub(super) struct FixtureReader;

impl TruthSnapshotReader for FixtureReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a")
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, crate::snapshot::BridgeSnapshotReadError> {
        let records = request
            .reads()
            .iter()
            .map(|read| {
                let aspect_value = match (read.entity_identity(), read.aspect_key().as_str()) {
                    ("entity-1", "identity.id") => AspectValue::String("task-1".into()),
                    ("entity-1", "status") => AspectValue::String("todo".into()),
                    ("entity-2", "identity.id") => AspectValue::String("task-2".into()),
                    ("entity-2", "status") => AspectValue::String("doing".into()),
                    _ => AspectValue::String("unknown".into()),
                };
                crate::snapshot::SnapshotReadRecord::for_request(read, aspect_value)
            })
            .collect();
        Ok(SnapshotReadPacketResult::new(
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            records,
        ))
    }
}

#[derive(Debug)]
pub(super) struct MissingRecordReader;

impl TruthSnapshotReader for MissingRecordReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a")
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, crate::snapshot::BridgeSnapshotReadError> {
        let records = request
            .reads()
            .iter()
            .take(request.reads().len().saturating_sub(1))
            .map(|read| {
                crate::snapshot::SnapshotReadRecord::for_request(
                    read,
                    AspectValue::String("partial".into()),
                )
            })
            .collect();
        Ok(SnapshotReadPacketResult::new(
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            records,
        ))
    }
}

#[derive(Debug)]
pub(super) struct AuthoritativeAbsenceReader;

impl TruthSnapshotReader for AuthoritativeAbsenceReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a")
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, crate::snapshot::BridgeSnapshotReadError> {
        let records = request
            .reads()
            .iter()
            .map(|read| {
                if read.entity_identity() == "entity-2" && read.aspect_key().as_str() == "status" {
                    crate::snapshot::SnapshotReadRecord::absent_for_request(read)
                } else {
                    let value = match (read.entity_identity(), read.aspect_key().as_str()) {
                        ("entity-1", "identity.id") => "task-1",
                        ("entity-1", "status") => "todo",
                        ("entity-2", "identity.id") => "task-2",
                        _ => "unknown",
                    };
                    crate::snapshot::SnapshotReadRecord::for_request(
                        read,
                        AspectValue::String(value.into()),
                    )
                }
            })
            .collect();
        Ok(SnapshotReadPacketResult::new(
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            records,
        ))
    }
}

#[derive(Debug)]
pub(super) struct ChangedStatusReader;

impl TruthSnapshotReader for ChangedStatusReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a")
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, crate::snapshot::BridgeSnapshotReadError> {
        let records = request
            .reads()
            .iter()
            .map(|read| {
                let aspect_value = match (read.entity_identity(), read.aspect_key().as_str()) {
                    ("entity-1", "identity.id") => AspectValue::String("task-1".into()),
                    ("entity-1", "status") => AspectValue::String("done".into()),
                    ("entity-2", "identity.id") => AspectValue::String("task-2".into()),
                    ("entity-2", "status") => AspectValue::String("doing".into()),
                    _ => AspectValue::String("unknown".into()),
                };
                crate::snapshot::SnapshotReadRecord::for_request(read, aspect_value)
            })
            .collect();
        Ok(SnapshotReadPacketResult::new(
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            records,
        ))
    }
}
