//! Both mirror directories retain their own original leases and Head symbols.
use super::*;
use crate::runtime::RuntimeSymbolTable;
use crate::storage::residency::RelationalRegionResidency;
use std::sync::Arc;

impl StorageSubsystem {
    pub(crate) fn preflight_source_retirement(&self) -> Result<(), ()> {
        self.partitions
            .try_write()
            .ok_or(())?
            .edition
            .exclude_source_capable_editions()
    }
    pub(crate) fn source_retirement_edition_cell(
        &self,
        after: Option<PartitionId>,
        retire_native: bool,
    ) -> Result<Option<(PartitionId, Arc<RelationalRegionResidency>)>, ()> {
        Ok(self
            .partitions
            .try_read()
            .ok_or(())?
            .edition
            .source_retirement_cell(after, retire_native))
    }
    pub(crate) fn source_retirement_target(
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
            MirrorRegion::AwaitingResidentTarget(original) => {
                if retire_native {
                    original.retire_native_leases();
                }
                (id, original.cell().clone(), None)
            }
            MirrorRegion::Resident(original) => {
                let (cell, symbols) = original.source_retirement_binding(retire_native)?;
                (id, cell, symbols)
            }
        }))
    }
}
