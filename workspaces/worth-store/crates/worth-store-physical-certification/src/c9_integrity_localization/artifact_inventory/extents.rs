use super::ArtifactInventory;
use worth_store_physical_format::*;
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

pub(super) fn collect(inventory: &mut ArtifactInventory, placement: DurableExtentRecordPlacement) {
    let file = RecordArtifactFile::ExtentManifest {
        extent: placement.extent().get(),
        generation: placement.extent_generation(),
    };
    let scope = PhysicalArtifactScope::extent_manifest(
        inventory.store,
        inventory.format,
        placement,
        inventory.record_range(file),
    );
    inventory.push_record(file, "extent_manifest", scope);
    let (manifest, _) = DurableExtentManifest::decode(&inventory.record_bytes(file)).unwrap();
    let data = RecordArtifactFile::Extent {
        extent: placement.extent().get(),
        generation: placement.extent_generation(),
    };
    let bytes = inventory.record_bytes(data);
    let mut physical_offset = 0;
    for ordinal in 0..manifest.chunk_count() {
        let length = 48 + super::u32_at(&bytes, physical_offset + 24) as usize;
        let coordinate = ExtentChunkCoordinate::new(
            placement.record(),
            placement.extent_cell(),
            placement.payload_bytes(),
            ordinal as u64 * manifest.chunk_payload_capacity() as u64,
            ordinal + 1,
        )
        .unwrap();
        inventory.push_record(
            data,
            "extent_chunk",
            PhysicalArtifactScope::extent_chunk(
                inventory.store,
                inventory.format,
                coordinate,
                PhysicalByteRange::new(physical_offset as u64, length as u64).unwrap(),
            ),
        );
        physical_offset += length;
    }
    assert_eq!(physical_offset, bytes.len());
}
