use crate::facade::{
    RelationalBridgeRecordIdentityParts, RelationalBridgeSourceError, TruthCommitIdentity,
    TruthSnapshotIdentity,
};
use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::VersionId;
use worth_relational::facade::snapshots::{SnapshotHandle, SnapshotId};
use worth_relational::facade::transactions::RecordRef;

pub(crate) fn relational_bridge_adapter_semantic_identity() -> std::sync::Arc<str> {
    std::sync::Arc::from("worth-relational-bridge-adapter-v1")
}

pub fn bridge_snapshot_identity_for_handle(handle: &SnapshotHandle) -> TruthSnapshotIdentity {
    TruthSnapshotIdentity::retained_by(
        handle.runtime_instance_id(),
        handle.snapshot_id(),
        handle.version_id(),
    )
}

pub fn bridge_snapshot_identity_for_commit(
    commit_id: CommitId,
    version_id: VersionId,
) -> TruthSnapshotIdentity {
    TruthSnapshotIdentity::retained(SnapshotId(commit_id.0), version_id)
}

pub(crate) fn record_ref_identity(record: &RecordRef) -> RelationalBridgeRecordIdentityParts {
    record.into()
}

pub(crate) fn require_relational_commit(
    identity: &TruthCommitIdentity,
) -> Result<CommitId, RelationalBridgeSourceError> {
    identity.commit_id().ok_or_else(|| {
        RelationalBridgeSourceError::new(
            crate::adapter::RelationalBridgeSourceErrorTag::Binding(
                crate::adapter::BridgeSourceBindingDenial::UnsupportedCommitIdentity,
            ),
            "unsupported relational bridge commit identity",
        )
    })
}
