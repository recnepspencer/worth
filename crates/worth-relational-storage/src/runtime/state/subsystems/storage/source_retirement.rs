//! Both mirror directories retain their own original leases and Head symbols.
use super::*;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::storage::residency::RelationalRegionResidency;
use std::sync::Arc;

impl StorageSubsystem {
    #[allow(
        clippy::result_unit_err,
        reason = "The close owner treats lock contention and a retained source edition as the same retry"
    )]
    pub fn preflight_source_retirement(&self) -> Result<(), ()> {
        self.partitions
            .try_write()
            .ok_or(())?
            .edition
            .exclude_source_capable_editions()
    }
    #[allow(
        clippy::result_unit_err,
        reason = "The close owner treats a missed read lock as one retry outcome"
    )]
    pub fn source_retirement_edition_cell(
        &self,
        after: Option<PartitionId>,
    ) -> Result<Option<(PartitionId, Arc<RelationalRegionResidency>)>, ()> {
        Ok(self
            .partitions
            .try_read()
            .ok_or(())?
            .edition
            .source_retirement_cell(after))
    }
    #[allow(
        clippy::result_unit_err,
        reason = "The close owner treats unavailable custody as one retry outcome"
    )]
    pub fn source_retirement_target(
        &self,
        after: Option<PartitionId>,
        retire_native: bool,
    ) -> Result<
        Option<(
            PartitionId,
            Arc<RelationalRegionResidency>,
            Option<RuntimeSymbolTable>,
        )>,
        (),
    > {
        use std::ops::Bound::{Excluded, Unbounded};
        let mirror = self.partitions.try_read().ok_or(())?;
        let next = match after {
            None => mirror.regions.first_key_value(),
            Some(key) => mirror.regions.range((Excluded(key), Unbounded)).next(),
        };
        let Some((&id, region)) = next else {
            return Ok(None);
        };
        Ok(Some(match region {
            // A retention holds no leases; only the resident target's edit
            // lease carries a native count to retire.
            MirrorRegion::AwaitingResidentTarget(original) => (id, original.cell().clone(), None),
            MirrorRegion::Resident(original) => {
                let (cell, symbols) = original.source_retirement_binding(retire_native)?;
                (id, cell, symbols)
            }
        }))
    }
}
