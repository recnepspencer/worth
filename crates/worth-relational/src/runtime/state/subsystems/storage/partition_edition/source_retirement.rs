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
        use std::ops::Bound::{Excluded, Unbounded};
        let next = match after {
            None => self.partitions.first_key_value(),
            Some(key) => self.partitions.range((Excluded(key), Unbounded)).next(),
        };
        next.map(|(&id, region)| {
            if retire_native {
                region.retire_native_leases();
            }
            (id, region.cell().clone())
        })
    }
}
