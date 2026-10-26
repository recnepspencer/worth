//! Capture resident canonical content without reconstructing a cold history.
use super::RelationalRegionResidency;
use crate::storage::data::RelationalStorageUnavailable;
use crate::storage::overlay::{PartitionContentCommitment, PartitionContentDigestError};
use crate::symbols::data::StringInterner;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) enum RegionContentFailure {
    Content(PartitionContentDigestError),
    Adjacency,
    Storage(RelationalStorageUnavailable),
}

impl RelationalRegionResidency {
    pub(crate) fn capture_resident_content(
        self: &Arc<Self>,
        symbols: &StringInterner,
    ) -> Result<Option<[u8; 32]>, RegionContentFailure> {
        let Some(lease) = self
            .lease_checkpoint_contents()
            .map_err(RegionContentFailure::Storage)?
        else {
            return Ok(None);
        };
        let partition = lease.checkpoint_contents();
        let symbols = self.content_symbols(symbols);
        let (commitment, _) = PartitionContentCommitment::capture_layered(partition, &symbols)
            .map_err(RegionContentFailure::Content)?;
        Ok(Some(commitment.digest(partition.partition_id)))
    }
}
