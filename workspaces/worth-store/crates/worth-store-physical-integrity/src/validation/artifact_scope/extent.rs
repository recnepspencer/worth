use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::{
    durable_artifact_checksum, DurableExtentRecordPlacement, ExtentChunkCoordinate,
    PhysicalRecordFormatDeclaration,
};

use super::{PhysicalArtifactScope, PhysicalArtifactScopeIdentity};
use crate::localization::PhysicalByteRange;

impl PhysicalArtifactScope {
    pub const fn extent_manifest(
        store: StableStoreIdentity,
        record_format: PhysicalRecordFormatDeclaration,
        placement: DurableExtentRecordPlacement,
        range: PhysicalByteRange,
    ) -> Self {
        Self::new(
            store,
            PhysicalArtifactScopeIdentity::ExtentManifest {
                record_format,
                placement,
            },
            range,
        )
    }

    pub const fn extent_chunk(
        store: StableStoreIdentity,
        record_format: PhysicalRecordFormatDeclaration,
        coordinate: ExtentChunkCoordinate,
        range: PhysicalByteRange,
    ) -> Self {
        Self::new(
            store,
            PhysicalArtifactScopeIdentity::ExtentChunk {
                record_format,
                coordinate,
            },
            range,
        )
    }

    pub const fn extent_manifest_placement(self) -> Option<DurableExtentRecordPlacement> {
        match self.identity {
            PhysicalArtifactScopeIdentity::ExtentManifest { placement, .. } => Some(placement),
            _ => None,
        }
    }

    pub const fn extent_chunk_coordinate(self) -> Option<ExtentChunkCoordinate> {
        match self.identity {
            PhysicalArtifactScopeIdentity::ExtentChunk { coordinate, .. } => Some(coordinate),
            _ => None,
        }
    }

    pub(crate) const fn is_extent_manifest(self) -> bool {
        matches!(
            self.identity,
            PhysicalArtifactScopeIdentity::ExtentManifest { .. }
        )
    }

    pub(crate) const fn is_extent_chunk(self) -> bool {
        matches!(
            self.identity,
            PhysicalArtifactScopeIdentity::ExtentChunk { .. }
        )
    }

    pub(crate) fn exact_extent_scope_digest(self) -> u32 {
        let mut bytes = [0_u8; 103];
        bytes[..16].copy_from_slice(&self.store.bytes());
        bytes[16] = if self.is_extent_manifest() {
            1
        } else if self.is_extent_chunk() {
            2
        } else {
            panic!("extent scope digest requires an extent-family scope")
        };
        bytes[17..25].copy_from_slice(&self.range.offset().to_le_bytes());
        bytes[25..33].copy_from_slice(&self.range.length().to_le_bytes());
        bytes[33..43].copy_from_slice(&self.record_format().canonical_identity_bytes());

        let preimage_length = match self.identity {
            PhysicalArtifactScopeIdentity::ExtentManifest { placement, .. } => {
                encode_record_identity(&mut bytes[43..67], placement.record());
                bytes[67..75].copy_from_slice(&placement.extent().get().to_le_bytes());
                bytes[75..83].copy_from_slice(&placement.extent_generation().to_le_bytes());
                bytes[83..91].copy_from_slice(&placement.payload_bytes().to_le_bytes());
                91
            }
            PhysicalArtifactScopeIdentity::ExtentChunk { coordinate, .. } => {
                encode_record_identity(&mut bytes[43..67], coordinate.record());
                bytes[67..75]
                    .copy_from_slice(&coordinate.extent_cell().extent_id().get().to_le_bytes());
                bytes[75..83]
                    .copy_from_slice(&coordinate.extent_cell().generation().get().to_le_bytes());
                bytes[83..91].copy_from_slice(&coordinate.logical_bytes().to_le_bytes());
                bytes[91..99].copy_from_slice(&coordinate.logical_offset().to_le_bytes());
                bytes[99..103].copy_from_slice(&coordinate.ordinal().to_le_bytes());
                103
            }
            _ => unreachable!("extent-family predicate was checked above"),
        };
        durable_artifact_checksum(&bytes[..preimage_length])
    }
}

fn encode_record_identity(
    target: &mut [u8],
    record: worth_store_physical_format::PersistedRecordIdentity,
) {
    target[..16].copy_from_slice(&record.allocation_epoch());
    target[16..24].copy_from_slice(&record.ordinal().to_le_bytes());
}

#[cfg(test)]
mod tests {
    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };
    use worth_store_physical_format::{
        DurableExtentRecordPlacement, ExtentChunkCoordinate, PersistedRecordIdentity,
        PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageSizeClass,
        PhysicalRecordFormatDeclaration, RecordExtentGenerationCell,
    };

    use super::*;

    #[test]
    fn manifest_scope_digest_changes_for_every_manifest_identity_axis() {
        let baseline = PhysicalArtifactScope::extent_manifest(
            store(7),
            format(PhysicalPageSizeClass::KiB16),
            placement(record(0x22, 7), cell(4, 5), 16_277),
            range(8_192, 104),
        );
        let variants = [
            PhysicalArtifactScope::extent_manifest(
                store(8),
                baseline.record_format(),
                placement(record(0x22, 7), cell(4, 5), 16_277),
                baseline.byte_range(),
            ),
            PhysicalArtifactScope::extent_manifest(
                store(7),
                format(PhysicalPageSizeClass::KiB32),
                placement(record(0x22, 7), cell(4, 5), 16_277),
                baseline.byte_range(),
            ),
            manifest_scope(record(0x33, 8), cell(4, 5), 16_277, baseline.byte_range()),
            manifest_scope(record(0x22, 7), cell(8, 5), 16_277, baseline.byte_range()),
            manifest_scope(record(0x22, 7), cell(4, 6), 16_277, baseline.byte_range()),
            manifest_scope(record(0x22, 7), cell(4, 5), 16_279, baseline.byte_range()),
            manifest_scope(record(0x22, 7), cell(4, 5), 16_277, range(12_288, 104)),
        ];

        for variant in variants {
            assert_ne!(
                baseline.exact_extent_scope_digest(),
                variant.exact_extent_scope_digest()
            );
        }
    }

    #[test]
    fn chunk_scope_digest_changes_for_every_chunk_identity_axis() {
        let baseline = chunk_scope(
            format(PhysicalPageSizeClass::KiB16),
            coordinate(record(0x22, 7), cell(4, 5), 16_277, 16_272, 2),
            range(16_384, 117),
        );
        let variants = [
            PhysicalArtifactScope::extent_chunk(
                store(8),
                baseline.record_format(),
                baseline.extent_chunk_coordinate().unwrap(),
                baseline.byte_range(),
            ),
            chunk_scope(
                format(PhysicalPageSizeClass::KiB32),
                baseline.extent_chunk_coordinate().unwrap(),
                baseline.byte_range(),
            ),
            chunk_variant(baseline, record(0x33, 8), cell(4, 5), 16_277, 16_272, 2),
            chunk_variant(baseline, record(0x22, 7), cell(8, 5), 16_277, 16_272, 2),
            chunk_variant(baseline, record(0x22, 7), cell(4, 6), 16_277, 16_272, 2),
            chunk_variant(baseline, record(0x22, 7), cell(4, 5), 16_279, 16_272, 2),
            chunk_variant(baseline, record(0x22, 7), cell(4, 5), 16_277, 0, 2),
            chunk_variant(baseline, record(0x22, 7), cell(4, 5), 16_277, 16_272, 3),
            chunk_scope(
                baseline.record_format(),
                baseline.extent_chunk_coordinate().unwrap(),
                range(20_480, 117),
            ),
        ];

        for variant in variants {
            assert_ne!(
                baseline.exact_extent_scope_digest(),
                variant.exact_extent_scope_digest()
            );
        }
    }

    fn manifest_scope(
        record: PersistedRecordIdentity,
        extent: RecordExtentGenerationCell,
        payload_bytes: u64,
        range: PhysicalByteRange,
    ) -> PhysicalArtifactScope {
        PhysicalArtifactScope::extent_manifest(
            store(7),
            format(PhysicalPageSizeClass::KiB16),
            placement(record, extent, payload_bytes),
            range,
        )
    }

    fn chunk_scope(
        format: PhysicalRecordFormatDeclaration,
        coordinate: ExtentChunkCoordinate,
        range: PhysicalByteRange,
    ) -> PhysicalArtifactScope {
        PhysicalArtifactScope::extent_chunk(store(7), format, coordinate, range)
    }

    fn chunk_variant(
        baseline: PhysicalArtifactScope,
        record: PersistedRecordIdentity,
        extent: RecordExtentGenerationCell,
        logical_bytes: u64,
        logical_offset: u64,
        ordinal: u32,
    ) -> PhysicalArtifactScope {
        chunk_scope(
            baseline.record_format(),
            coordinate(record, extent, logical_bytes, logical_offset, ordinal),
            baseline.byte_range(),
        )
    }

    fn placement(
        record: PersistedRecordIdentity,
        extent: RecordExtentGenerationCell,
        payload_bytes: u64,
    ) -> DurableExtentRecordPlacement {
        DurableExtentRecordPlacement::new(record, extent, payload_bytes).unwrap()
    }

    fn coordinate(
        record: PersistedRecordIdentity,
        extent: RecordExtentGenerationCell,
        logical_bytes: u64,
        logical_offset: u64,
        ordinal: u32,
    ) -> ExtentChunkCoordinate {
        ExtentChunkCoordinate::new(record, extent, logical_bytes, logical_offset, ordinal).unwrap()
    }

    fn record(byte: u8, ordinal: u64) -> PersistedRecordIdentity {
        PersistedRecordIdentity::new([byte; 16], ordinal).unwrap()
    }

    fn cell(extent: u64, generation: u64) -> RecordExtentGenerationCell {
        PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(PhysicalExtentId::from_raw(extent).unwrap())
            .with_extent_generation(PhysicalGeneration::from_raw(generation).unwrap())
    }

    fn format(page_size: PhysicalPageSizeClass) -> PhysicalRecordFormatDeclaration {
        PhysicalRecordFormatDeclaration::builder()
            .page_size(page_size)
            .admit()
            .unwrap()
    }

    fn store(byte: u8) -> StableStoreIdentity {
        StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([byte; 16]).unwrap(),
        )
        .published_identity()
    }

    fn range(offset: u64, length: u64) -> PhysicalByteRange {
        PhysicalByteRange::new(offset, length).unwrap()
    }
}
