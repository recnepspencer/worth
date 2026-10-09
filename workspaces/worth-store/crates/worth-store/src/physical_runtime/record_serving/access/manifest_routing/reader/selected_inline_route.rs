use worth_store_buffer_pool::OperationAllocationGrant;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, PageGenerationCell, PersistedRecordIdentity,
    PhysicalRecordSlot, SelectedRecordRouteMetadata,
};

use super::{ManifestDiscoveryCounterSnapshot, ManifestLookupFailure, ManifestReader};

impl ManifestReader<'_> {
    pub(in crate::physical_runtime::record_serving) fn require_selected_inline_metadata(
        &self,
        allocation: &OperationAllocationGrant,
        record: PersistedRecordIdentity,
        page: PageGenerationCell,
        slot: PhysicalRecordSlot,
        slot_generation: u64,
        payload_bytes: u64,
    ) -> Result<SelectedRecordRouteMetadata, ManifestLookupFailure> {
        self.selected_inline_metadata_if_routed(
            allocation,
            record,
            page,
            slot,
            slot_generation,
            payload_bytes,
        )?
        .ok_or(ManifestLookupFailure::Damaged)
    }

    /// A retired route may leave an old physical slot in a reusable page.
    /// Its bytes are not remapped into the new selected root.
    pub(in crate::physical_runtime::record_serving) fn selected_inline_metadata_if_routed(
        &self,
        allocation: &OperationAllocationGrant,
        record: PersistedRecordIdentity,
        page: PageGenerationCell,
        slot: PhysicalRecordSlot,
        slot_generation: u64,
        payload_bytes: u64,
    ) -> Result<Option<SelectedRecordRouteMetadata>, ManifestLookupFailure> {
        let mut counters = ManifestDiscoveryCounterSnapshot::default();
        let source = match self.locate(allocation, record, &mut counters)? {
            None => return Ok(None),
            Some(CurrentPhysicalRecordPlacement::Inline(source)) => source,
            Some(_) => return Err(ManifestLookupFailure::Damaged),
        };
        if source.page_cell() != page
            || source.slot() != slot
            || source.slot_generation() != slot_generation
            || source.payload_bytes() != payload_bytes
        {
            return Err(ManifestLookupFailure::Damaged);
        }
        Ok(Some(source.route_metadata()))
    }
}
