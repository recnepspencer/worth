use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
};

pub(super) fn selected_absence_identity(
    root: &DurablePhysicalRootManifest,
    placements: &[CurrentPhysicalRecordPlacement],
    format: PhysicalRecordFormatDeclaration,
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth.store.recovery.selected-root-absence.v1");
    digest.update(root.encode(format));
    for placement in placements {
        digest.update(placement.record().allocation_epoch());
        digest.update(placement.record().ordinal().to_le_bytes());
        match placement {
            CurrentPhysicalRecordPlacement::Inline(inline) => {
                digest.update([1]);
                digest.update(inline.segment().get().to_le_bytes());
                digest.update(inline.segment_generation().to_le_bytes());
                digest.update(inline.page().get().to_le_bytes());
                digest.update(inline.page_generation().to_le_bytes());
                digest.update(inline.slot().get().to_le_bytes());
                digest.update(inline.slot_generation().to_le_bytes());
            }
            CurrentPhysicalRecordPlacement::Extent(extent) => {
                digest.update([2]);
                digest.update(extent.extent().get().to_le_bytes());
                digest.update(extent.extent_generation().to_le_bytes());
            }
        }
    }
    digest.finalize().into()
}
