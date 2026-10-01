//! Versioned semantic inventory transcript shared by C.8 and Store's fresh
//! selected-media rewalk. Each family is consumed in key order, one entry at
//! a time; tree-block layout does not affect the resulting commitment.

use sha2::{Digest, Sha256};

use super::durable_root_entry;
use super::{
    CurrentPhysicalRecordPlacement, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    FreeSpaceKey, RecordFreeSpaceManifestEntry, RecordFreeSpaceRegion,
    RecordSegmentPageManifestEntry, SegmentPageKey,
};
use crate::{durable_artifact_checksum, PersistedRecordIdentity, PhysicalRecordFormatDeclaration};

const ROOT_DOMAIN: &[u8] = b"worth.store.physical.inventory.v1.root";
const FREE_HEADER_DOMAIN: &[u8] = b"worth.store.physical.inventory.v1.free-header";
const ROUTES_DOMAIN: &[u8] = b"worth.store.physical.inventory.v1.routes";
const SEGMENTS_DOMAIN: &[u8] = b"worth.store.physical.inventory.v1.segments";
const FREE_ENTRIES_DOMAIN: &[u8] = b"worth.store.physical.inventory.v1.free-entries";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalInventoryTranscriptDenial {
    RootFreeMismatch,
    OutOfOrder,
    BoundExceeded,
    CountMismatch,
}

/// Complete semantic inventory commitment, not an authority token by itself.
/// C.8 mints it only after source-to-result proof; Store independently streams
/// actual admitted media into the same schema before a Serving seal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalInventoryTranscriptV1 {
    root_sha256: [u8; 32],
    free_header_sha256: [u8; 32],
    route_count: u64,
    routes_sha256: [u8; 32],
    segment_count: u64,
    segments_sha256: [u8; 32],
    free_entry_count: u64,
    free_entries_sha256: [u8; 32],
}

impl PhysicalInventoryTranscriptV1 {
    pub fn matches_headers(
        self,
        root: &DurablePhysicalRootManifest,
        free: &DurableFreeSpaceManifestHeader,
        format: PhysicalRecordFormatDeclaration,
    ) -> bool {
        self.root_sha256 == framed_digest(ROOT_DOMAIN, &root.encode(format))
            && self.free_header_sha256 == framed_digest(FREE_HEADER_DOMAIN, &free.encode(format))
            && self.route_count == root.record_count()
            && self.free_entry_count == free.entry_count()
    }

    pub const fn root_sha256(self) -> [u8; 32] {
        self.root_sha256
    }
    pub const fn free_header_sha256(self) -> [u8; 32] {
        self.free_header_sha256
    }
    pub const fn route_count(self) -> u64 {
        self.route_count
    }
    pub const fn routes_sha256(self) -> [u8; 32] {
        self.routes_sha256
    }
    pub const fn segment_count(self) -> u64 {
        self.segment_count
    }
    pub const fn segments_sha256(self) -> [u8; 32] {
        self.segments_sha256
    }
    pub const fn free_entry_count(self) -> u64 {
        self.free_entry_count
    }
    pub const fn free_entries_sha256(self) -> [u8; 32] {
        self.free_entries_sha256
    }
}

pub struct PhysicalInventoryTranscriptBuilderV1 {
    root: DurablePhysicalRootManifest,
    free: DurableFreeSpaceManifestHeader,
    maximum_entries: u64,
    routes: Sha256,
    segments: Sha256,
    free_entries: Sha256,
    route_count: u64,
    segment_count: u64,
    free_entry_count: u64,
    last_route: Option<PersistedRecordIdentity>,
    last_segment: Option<SegmentPageKey>,
    last_free: Option<FreeSpaceKey>,
    root_sha256: [u8; 32],
    free_header_sha256: [u8; 32],
}

impl PhysicalInventoryTranscriptBuilderV1 {
    pub fn new(
        root: &DurablePhysicalRootManifest,
        free: &DurableFreeSpaceManifestHeader,
        format: PhysicalRecordFormatDeclaration,
        maximum_entries: u64,
    ) -> Result<Self, PhysicalInventoryTranscriptDenial> {
        Self::new_in_reserved(
            root,
            free,
            format,
            maximum_entries,
            Vec::with_capacity(root.encoded_frame_bytes()),
            Vec::with_capacity(free.encoded_frame_bytes()),
        )
    }

    /// Canonical transcript initialization using caller-admitted backing only.
    /// Caller buffers grant no truth: this owner encodes both headers itself.
    pub fn new_in_reserved(
        root: &DurablePhysicalRootManifest,
        free: &DurableFreeSpaceManifestHeader,
        format: PhysicalRecordFormatDeclaration,
        maximum_entries: u64,
        root_frame: Vec<u8>,
        free_frame: Vec<u8>,
    ) -> Result<Self, PhysicalInventoryTranscriptDenial> {
        let free_bytes = free
            .encode_in_reserved(format, free_frame)
            .ok_or(PhysicalInventoryTranscriptDenial::BoundExceeded)?;
        if maximum_entries == 0
            || root.generation() != free.generation()
            || root.tree_identity() != free.tree_identity()
            || root.node_capacity() != free.node_capacity()
            || root.free_space_root() != free.root()
            || root.free_space_checksum() != durable_artifact_checksum(&free_bytes)
            || root.record_count() > maximum_entries
            || free.entry_count() > maximum_entries
        {
            return Err(PhysicalInventoryTranscriptDenial::RootFreeMismatch);
        }
        let root_bytes = root
            .encode_in_reserved(format, root_frame)
            .ok_or(PhysicalInventoryTranscriptDenial::BoundExceeded)?;
        let root_sha256 = framed_digest(ROOT_DOMAIN, &root_bytes);
        let free_header_sha256 = framed_digest(FREE_HEADER_DOMAIN, &free_bytes);
        Ok(Self {
            root: root.clone(),
            free: free.clone(),
            maximum_entries,
            routes: domain_hasher(ROUTES_DOMAIN),
            segments: domain_hasher(SEGMENTS_DOMAIN),
            free_entries: domain_hasher(FREE_ENTRIES_DOMAIN),
            route_count: 0,
            segment_count: 0,
            free_entry_count: 0,
            last_route: None,
            last_segment: None,
            last_free: None,
            root_sha256,
            free_header_sha256,
        })
    }

    pub fn include_route(
        &mut self,
        placement: CurrentPhysicalRecordPlacement,
    ) -> Result<(), PhysicalInventoryTranscriptDenial> {
        let key = placement.record();
        if self.last_route.is_some_and(|prior| prior >= key) {
            return Err(PhysicalInventoryTranscriptDenial::OutOfOrder);
        }
        self.route_count = charged(self.route_count, self.maximum_entries)?;
        let mut bytes = [0_u8; 88];
        durable_root_entry::encode_entry(&mut bytes, placement);
        self.routes.update(bytes);
        self.last_route = Some(key);
        Ok(())
    }

    pub fn include_segment(
        &mut self,
        entry: RecordSegmentPageManifestEntry,
    ) -> Result<(), PhysicalInventoryTranscriptDenial> {
        let key = SegmentPageKey::from(entry);
        if self.last_segment.is_some_and(|prior| prior >= key) {
            return Err(PhysicalInventoryTranscriptDenial::OutOfOrder);
        }
        self.segment_count = charged(self.segment_count, self.maximum_entries)?;
        let mut bytes = [0_u8; 40];
        bytes[..8].copy_from_slice(&entry.page_cell().segment_id().get().to_le_bytes());
        bytes[8..16].copy_from_slice(&entry.page().get().to_le_bytes());
        bytes[16..24].copy_from_slice(&entry.page_generation().to_le_bytes());
        bytes[24..32].copy_from_slice(&entry.data_generation().to_le_bytes());
        bytes[32..36].copy_from_slice(&entry.data_page_count().to_le_bytes());
        bytes[36..40].copy_from_slice(&entry.frame_index().to_le_bytes());
        self.segments.update(bytes);
        self.last_segment = Some(key);
        Ok(())
    }

    pub fn include_free(
        &mut self,
        entry: RecordFreeSpaceManifestEntry,
    ) -> Result<(), PhysicalInventoryTranscriptDenial> {
        let key = FreeSpaceKey::from(entry);
        if self.last_free.is_some_and(|prior| prior >= key) {
            return Err(PhysicalInventoryTranscriptDenial::OutOfOrder);
        }
        self.free_entry_count = charged(self.free_entry_count, self.maximum_entries)?;
        let mut bytes = [0_u8; 40];
        bytes[0] = entry.class() as u8;
        bytes[8..16].copy_from_slice(&entry.owner().to_le_bytes());
        let (start, length) = match entry.region() {
            RecordFreeSpaceRegion::Inline(value) => {
                (value.first_unallocated(), value.unallocated_count())
            }
            RecordFreeSpaceRegion::Arena(range) => (range.offset(), range.length()),
        };
        bytes[16..24].copy_from_slice(&start.to_le_bytes());
        bytes[24..32].copy_from_slice(&length.to_le_bytes());
        bytes[32..40].copy_from_slice(&entry.generation().to_le_bytes());
        self.free_entries.update(bytes);
        self.last_free = Some(key);
        Ok(())
    }

    pub fn finish(
        mut self,
    ) -> Result<PhysicalInventoryTranscriptV1, PhysicalInventoryTranscriptDenial> {
        if self.route_count != self.root.record_count()
            || self.free_entry_count != self.free.entry_count()
            || (self.root.segment_root().is_some() != (self.segment_count > 0))
            || (self.free.root().is_some() != (self.free_entry_count > 0))
        {
            return Err(PhysicalInventoryTranscriptDenial::CountMismatch);
        }
        self.routes.update(self.route_count.to_le_bytes());
        self.segments.update(self.segment_count.to_le_bytes());
        self.free_entries
            .update(self.free_entry_count.to_le_bytes());
        Ok(PhysicalInventoryTranscriptV1 {
            root_sha256: self.root_sha256,
            free_header_sha256: self.free_header_sha256,
            route_count: self.route_count,
            routes_sha256: self.routes.finalize().into(),
            segment_count: self.segment_count,
            segments_sha256: self.segments.finalize().into(),
            free_entry_count: self.free_entry_count,
            free_entries_sha256: self.free_entries.finalize().into(),
        })
    }
}

fn charged(current: u64, maximum: u64) -> Result<u64, PhysicalInventoryTranscriptDenial> {
    current
        .checked_add(1)
        .filter(|count| *count <= maximum)
        .ok_or(PhysicalInventoryTranscriptDenial::BoundExceeded)
}

fn domain_hasher(domain: &[u8]) -> Sha256 {
    let mut hasher = Sha256::new();
    hasher.update((domain.len() as u64).to_le_bytes());
    hasher.update(domain);
    hasher
}

fn framed_digest(domain: &[u8], bytes: &[u8]) -> [u8; 32] {
    let mut hasher = domain_hasher(domain);
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        durable_artifact_checksum, CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement,
        ExtentArenaId, ExtentArenaRange, PersistedRecordIdentity, PhysicalExtentId,
        PhysicalGeneration, PhysicalGenerationAuthority, PhysicalRecordFormatDeclaration,
        PhysicalRootRoutingBlock,
    };

    fn fixture() -> (
        PhysicalRecordFormatDeclaration,
        DurablePhysicalRootManifest,
        DurableFreeSpaceManifestHeader,
        CurrentPhysicalRecordPlacement,
    ) {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let free =
            DurableFreeSpaceManifestHeader::new(1, 1, 2, 4, 0, 1, 1, 2, 2, 65536, 4096, 1, None)
                .unwrap();
        let record = PersistedRecordIdentity::new([3; 16], 1).unwrap();
        let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(PhysicalExtentId::from_raw(1).unwrap())
            .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
        let placement = CurrentPhysicalRecordPlacement::Extent(
            DurableExtentRecordPlacement::legacy_unknown(
                record,
                extent,
                3,
                ExtentArenaRange::new(ExtentArenaId::new(1).unwrap(), 0, 20480).unwrap(),
            )
            .unwrap(),
        );
        let routing = PhysicalRootRoutingBlock::leaf(1, 1, 1, vec![placement], 2).unwrap();
        let reference = routing.reference(durable_artifact_checksum(&routing.encode(format)));
        let root = DurablePhysicalRootManifest::builder(
            1,
            1,
            2,
            durable_artifact_checksum(&free.encode(format)),
        )
        .record_count(1)
        .next_block(2)
        .routing_root(Some(reference))
        .admit()
        .unwrap();
        (format, root, free, placement)
    }

    #[test]
    fn same_count_route_substitution_changes_semantic_transcript() {
        let (format, root, free, original) = fixture();
        let mut original_builder =
            PhysicalInventoryTranscriptBuilderV1::new(&root, &free, format, 2).unwrap();
        original_builder.include_route(original).unwrap();
        let original_transcript = original_builder.finish().unwrap();
        let CurrentPhysicalRecordPlacement::Extent(extent) = original else {
            unreachable!()
        };
        let changed = CurrentPhysicalRecordPlacement::Extent(
            DurableExtentRecordPlacement::legacy_unknown(
                extent.record(),
                extent.extent_cell(),
                extent.payload_bytes(),
                ExtentArenaRange::new(ExtentArenaId::new(1).unwrap(), 4096, 20480).unwrap(),
            )
            .unwrap(),
        );
        let mut changed_builder =
            PhysicalInventoryTranscriptBuilderV1::new(&root, &free, format, 2).unwrap();
        changed_builder.include_route(changed).unwrap();
        let changed_transcript = changed_builder.finish().unwrap();
        assert_eq!(
            original_transcript.route_count(),
            changed_transcript.route_count()
        );
        assert_ne!(
            original_transcript.routes_sha256(),
            changed_transcript.routes_sha256()
        );
    }

    #[test]
    fn duplicate_route_and_missing_count_fail_closed() {
        let (format, root, free, placement) = fixture();
        let mut duplicate =
            PhysicalInventoryTranscriptBuilderV1::new(&root, &free, format, 2).unwrap();
        duplicate.include_route(placement).unwrap();
        assert_eq!(
            duplicate.include_route(placement),
            Err(PhysicalInventoryTranscriptDenial::OutOfOrder)
        );
        let missing = PhysicalInventoryTranscriptBuilderV1::new(&root, &free, format, 2).unwrap();
        assert_eq!(
            missing.finish(),
            Err(PhysicalInventoryTranscriptDenial::CountMismatch)
        );
    }
}
