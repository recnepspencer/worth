#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalSnapshotReleaseDenial {
    ForeignRuntime {
        expected_runtime_instance_id: u64,
        actual_runtime_instance_id: u64,
    },
    UnknownSnapshot,
    BindingMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalSnapshotReleaseReceipt {
    snapshot_id: crate::snapshots::data::coordinates::SnapshotId,
}

impl RelationalSnapshotReleaseReceipt {
    pub(crate) const fn new(snapshot_id: crate::snapshots::data::coordinates::SnapshotId) -> Self {
        Self { snapshot_id }
    }

    pub const fn snapshot_id(&self) -> crate::snapshots::data::coordinates::SnapshotId {
        self.snapshot_id
    }
}

mod runtime;
