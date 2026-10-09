use super::ArtifactInventory;
use worth_store_physical_format::*;
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

pub(super) fn collect(inventory: &mut ArtifactInventory, placement: DurableExtentRecordPlacement) {
    let arena = placement.arena_range();
    let file = RecordArtifactFile::ExtentArena {
        arena: arena.arena().get(),
    };
    let bytes = inventory.record_bytes(file);
    let start = usize::try_from(arena.offset()).unwrap();
    let end = usize::try_from(arena.end()).unwrap();
    let manifest_end = start + EXTENT_ARENA_MANIFEST_FRAME_BYTES;
    assert!(
        manifest_end <= end && manifest_end <= bytes.len(),
        "selected arena must contain its framed manifest"
    );
    let (manifest, _) = DurableExtentManifest::decode(&bytes[start..manifest_end]).unwrap();
    assert_eq!(manifest.record(), placement.record());
    assert_eq!(manifest.extent_cell(), placement.extent_cell());
    assert_eq!(manifest.logical_bytes(), placement.payload_bytes());
    let layout = ExtentArenaFrameLayout::new(inventory.format, manifest.alignment()).unwrap();
    assert!(layout.admits(arena, manifest.chunk_count()));

    inventory.push_record(
        file,
        "extent_manifest",
        PhysicalArtifactScope::extent_manifest(
            inventory.store,
            inventory.format,
            placement,
            PhysicalByteRange::new(arena.offset(), EXTENT_ARENA_MANIFEST_FRAME_BYTES as u64)
                .unwrap(),
        ),
    );
    for ordinal in 1..=manifest.chunk_count() {
        let offset =
            usize::try_from(arena.offset() + layout.chunk_offset(ordinal).unwrap()).unwrap();
        assert!(
            offset + 28 <= end && offset + 28 <= bytes.len(),
            "selected chunk header must exist on media"
        );
        let length = 48 + super::u32_at(&bytes, offset + 24) as usize;
        assert!(
            length <= usize::try_from(layout.chunk_stride()).unwrap()
                && offset + length <= end
                && offset + length <= bytes.len(),
            "selected chunk frame must stay within its arena range and durable bytes"
        );
        let coordinate = ExtentChunkCoordinate::new(
            placement.record(),
            placement.extent_cell(),
            placement.payload_bytes(),
            u64::from(ordinal - 1) * u64::from(manifest.chunk_payload_capacity()),
            ordinal,
        )
        .unwrap();
        inventory.push_record(
            file,
            "extent_chunk",
            PhysicalArtifactScope::extent_chunk(
                inventory.store,
                inventory.format,
                coordinate,
                PhysicalByteRange::new(offset as u64, length as u64).unwrap(),
                arena,
            ),
        );
    }
}
