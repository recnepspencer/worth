//! Canonical transcript for one bounded rooted inventory snapshot.

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest,
    PhysicalInventoryTranscriptBuilderV1, PhysicalInventoryTranscriptV1,
    PhysicalRecordFormatDeclaration,
};

use crate::progression::RecoverySelectedSourceInventory;

pub(super) fn inventory_transcript(
    root: &DurablePhysicalRootManifest,
    inventory: &RecoverySelectedSourceInventory,
    routes: &[CurrentPhysicalRecordPlacement],
    format: PhysicalRecordFormatDeclaration,
    maximum_entries: u64,
) -> Option<PhysicalInventoryTranscriptV1> {
    let mut transcript = PhysicalInventoryTranscriptBuilderV1::new(
        root,
        &inventory.free_space,
        format,
        maximum_entries,
    )
    .ok()?;
    for route in routes {
        transcript.include_route(*route).ok()?;
    }
    for page in inventory.segment_pages.values() {
        transcript.include_segment(page.entry).ok()?;
    }
    for free in &inventory.free_entries {
        transcript.include_free(*free).ok()?;
    }
    transcript.finish().ok()
}
