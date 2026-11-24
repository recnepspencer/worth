use super::StorageAuthority;
use crate::storage::overlay::published_delta::RelationalPublishedPartitionDelta;

impl StorageAuthority<'_> {
    pub(crate) fn affirm_no_partition_changes(&self) -> RelationalPublishedPartitionDelta {
        RelationalPublishedPartitionDelta::empty()
    }
}
