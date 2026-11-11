//! The original edition spine is never copied into a close roster.
use super::*;
impl PartitionEdition {
    pub(in crate::runtime::state::subsystems::storage) fn exclude_source_capable_editions(
        &mut self,
    ) -> Result<(), ()> {
        Arc::get_mut(&mut self.partitions).ok_or(())?;
        Ok(())
    }
    pub(in crate::runtime::state::subsystems::storage) fn source_retirement_cell(
        &self,
        after: Option<PartitionId>,
        retire_native: bool,
    ) -> Option<(
        PartitionId,
        Arc<crate::storage::residency::RelationalRegionResidency>,
    )> {
        // PartitionId has a native u32 ordering. Advancing its scalar key
        // preserves exclusive cursor semantics without allocating a traversal.
        let first = match after {
            None => PartitionId(u32::MIN),
            Some(key) => PartitionId(key.as_u32().checked_add(1)?),
        };
        let next = self.partitions.lower_bound(&first);
        next.map(|(&id, region)| {
            if retire_native {
                region.retire_native_leases();
            }
            (id, region.cell().clone())
        })
    }
}
