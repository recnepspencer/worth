use crate::history::data::CommitId;
use crate::identity::data::{EntityId, PartitionId, RelationId, VersionId};
use crate::snapshots::data::{SnapshotHandle, SnapshotId};
use crate::transactions::data::RecordRef;
use worth_runtime_bridge::facade::{
    RelationalBridgeRecordIdentityKind, RelationalBridgeRecordIdentityParts,
    RelationalBridgeSnapshotIdentityParts, RelationalBridgeSourceError, TruthCommitIdentity,
    TruthSnapshotIdentity,
};

pub(crate) fn relational_bridge_adapter_semantic_identity() -> std::sync::Arc<str> {
    std::sync::Arc::from("worth-relational-bridge-adapter-v1")
}

pub fn bridge_snapshot_identity_for_handle(handle: &SnapshotHandle) -> TruthSnapshotIdentity {
    bridge_snapshot_identity_for_binding(handle.snapshot_id, handle.version_id)
}

pub fn bridge_snapshot_identity_for_commit(
    commit_id: CommitId,
    version_id: VersionId,
) -> TruthSnapshotIdentity {
    bridge_snapshot_identity_for_binding(SnapshotId(commit_id.0), version_id)
}

pub(crate) fn record_ref_identity(record: &RecordRef) -> RelationalBridgeRecordIdentityParts {
    match record {
        RecordRef::Entity(entity) => RelationalBridgeRecordIdentityParts::entity(
            entity.partition_id.0,
            entity.local_slot.0,
            entity.generation.0,
        ),
        RecordRef::Relation(relation) => RelationalBridgeRecordIdentityParts::relation(
            relation.partition_id.0,
            relation.local_slot.0,
            relation.generation.0,
        ),
    }
}

pub(crate) fn record_ref_from_identity_parts(
    parts: RelationalBridgeRecordIdentityParts,
) -> Result<RecordRef, RelationalBridgeSourceError> {
    let partition_id = PartitionId(parts.partition_id());
    Ok(match parts.kind() {
        RelationalBridgeRecordIdentityKind::Entity => RecordRef::Entity(EntityId::new(
            partition_id,
            parts.local_slot(),
            parts.generation(),
        )),
        RelationalBridgeRecordIdentityKind::Relation => RecordRef::Relation(RelationId::new(
            partition_id,
            parts.local_slot(),
            parts.generation(),
        )),
    })
}

pub(crate) fn parse_bridge_commit_identity(
    identity: &TruthCommitIdentity,
) -> Result<CommitId, RelationalBridgeSourceError> {
    let commit_id = identity.relational_commit_id().ok_or_else(|| {
        RelationalBridgeSourceError::new("unsupported relational bridge commit identity")
    })?;
    Ok(CommitId(commit_id))
}

pub(crate) fn bridge_snapshot_identity_for_binding(
    snapshot_id: SnapshotId,
    version_id: VersionId,
) -> TruthSnapshotIdentity {
    TruthSnapshotIdentity::from_relational_snapshot(RelationalBridgeSnapshotIdentityParts::new(
        snapshot_id.0,
        version_id.0,
    ))
}

pub(crate) fn parse_bridge_snapshot_identity(
    identity: &TruthSnapshotIdentity,
) -> Result<(SnapshotId, VersionId), RelationalBridgeSourceError> {
    let parts = identity.relational_snapshot_parts().ok_or_else(|| {
        RelationalBridgeSourceError::new("unsupported relational bridge snapshot identity")
    })?;
    Ok((
        SnapshotId(parts.snapshot_id()),
        VersionId(parts.version_id()),
    ))
}
