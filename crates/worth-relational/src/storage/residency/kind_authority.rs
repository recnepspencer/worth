//! Root-qualified identity of the region proofs admitted from its manifest.
//! Member admission verifies the region's own kind commitment; it never walks
//! this directory to acquire another region's payload.
use super::RelationalRegionResidency;
use crate::identity::data::PartitionId;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock, Weak};

#[derive(Debug, Default)]
pub(crate) struct RelationalRegionKindAuthority {
    regions: OnceLock<BTreeMap<PartitionId, Weak<RelationalRegionResidency>>>,
}

impl RelationalRegionKindAuthority {
    pub(super) fn require_cell(&self, cell: &RelationalRegionResidency) -> Result<(), Cause> {
        if let Some(regions) = self.regions.get() {
            let expected = regions
                .get(&cell.partition_id)
                .ok_or(Cause::IntegrityMismatch)?;
            if !std::ptr::eq(expected.as_ptr(), cell) {
                return Err(Cause::IntegrityMismatch);
            }
        }
        Ok(())
    }
    pub(crate) fn bind(
        &self,
        regions: impl IntoIterator<Item = (PartitionId, Arc<RelationalRegionResidency>)>,
    ) -> Result<(), Cause> {
        let mut directory = BTreeMap::new();
        for (id, cell) in regions {
            if directory.insert(id, Arc::downgrade(&cell)).is_some() {
                return Err(Cause::Damaged);
            }
        }
        self.regions.set(directory).map_err(|_| Cause::Damaged)
    }
}
