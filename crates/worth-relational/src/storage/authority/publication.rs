use super::StorageAuthority;
use crate::storage::overlay::published_delta::RelationalPublishedPartitionDelta;
use crate::storage::PreparedMirrorPublication;

mod execution;

impl StorageAuthority<'_> {
    pub(crate) fn publish_prepared_mirror(&self, prepared: PreparedMirrorPublication) {
        execution::execute_partition_publication(self, prepared);
    }

    pub(crate) fn affirm_no_partition_changes(&self) -> RelationalPublishedPartitionDelta {
        RelationalPublishedPartitionDelta::empty()
    }
}
