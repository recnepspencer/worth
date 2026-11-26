//! Cold record directory pages of a readmitted cell (`G7`).
use super::*;
use crate::storage::data::record_directory_pages::{
    PagedRecordDirectory, RECORD_DIRECTORY_PAGE_BYTES,
};
use crate::storage::residency::chunked::cold_member_port::ColdMemberSource;
use crate::storage::residency::chunked::member_io::ChunkMemberSource;
use crate::storage::residency::record_directory::{RecordDirectoryPages, RegionRecordDirectory};
use crate::storage::residency::working_set::{
    ResidencyBudget, ResidencyPoolKind, WorkingSetAccountant,
};

impl RelationalRegionResidency {
    /// Opens the leaf's paged record directory without reading any page.
    /// Pages fault through this cell's working-set pool from the cell's own
    /// retained member custody. The directory holds the cell weakly: the root
    /// that retains the directory retains the cell, so the directory never
    /// extends cold custody past the cell's lifetime or Home close. An unbound
    /// cell faults through an unbudgeted accountant.
    pub fn record_directory(
        self: &Arc<Self>,
        descriptor: Arc<PagedRecordDirectory>,
    ) -> Arc<RegionRecordDirectory> {
        let (accountant, pool) = match self.working_set.get() {
            Some(binding) => (binding.accountant.clone(), binding.pool),
            None => (unbudgeted(), ResidencyPoolKind::Head),
        };
        let pages = RecordDirectoryPages::new(
            descriptor.pages.clone(),
            Arc::new(CellDirectoryPages(Arc::downgrade(self))),
            accountant,
            pool,
        );
        RegionRecordDirectory::from_paged(self.partition_id, descriptor, pages)
    }
}

fn unbudgeted() -> Arc<WorkingSetAccountant> {
    WorkingSetAccountant::new(ResidencyBudget {
        head_working_set_bytes: u64::MAX,
        historical_working_set_bytes: u64::MAX,
        max_in_flight_fill_bytes: u64::MAX,
        streaming_window_bytes: u32::MAX,
    })
}

/// Reads one page under a serving retain of the cell's member custody.
struct CellDirectoryPages(std::sync::Weak<RelationalRegionResidency>);

impl ChunkMemberSource for CellDirectoryPages {
    fn read_range(
        &self,
        member: RelationalDurableMemberLocator,
        offset: u64,
        target: &mut [u8],
    ) -> Result<usize, Cause> {
        let cell = self.0.upgrade().ok_or(Cause::SourceUnavailable)?;
        cell.check_damage().map_err(|failure| failure.cause)?;
        let custody = cell
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .physical
            .as_ref()
            .and_then(|physical| physical.try_retain_serving())
            .ok_or(Cause::SourceUnavailable)?;
        ColdMemberSource::with_custody(custody, RECORD_DIRECTORY_PAGE_BYTES as u64)
            .read_range(member, offset, target)
    }
}
