mod pinning;
mod publication;
mod retention;

pub(crate) use publication::{PreparedMirrorPublication, RelationalPublishedPartitionDelta};

use crate::identity::data::{PartitionId, RecordId};
use crate::runtime::RelationalRuntime;
use crate::storage::substrate::RecordKind;

pub struct StorageAuthority<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

impl RelationalRuntime {
    pub(crate) fn storage_authority(&self) -> StorageAuthority<'_> {
        StorageAuthority::new(self)
    }
}

impl<'runtime> StorageAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}

pub(super) fn partition_of<K: RecordKind>(id: &RecordId<K::Domain>) -> PartitionId {
    id.partition_id
}

pub(super) fn slot_of<K: RecordKind>(id: &RecordId<K::Domain>) -> usize {
    id.slot_index()
}
