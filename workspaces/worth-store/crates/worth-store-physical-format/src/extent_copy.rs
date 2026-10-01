//! Root-independent, bounded extent-copy obligations carried by the existing WAL.
//! The WAL envelope supplies the intent LSN; payload bytes cannot invent it.
mod codec;
#[cfg(test)]
mod tests;

use crate::{
    DurableExtentManifest, DurableExtentRecordPlacement, ExtentArenaFrameLayout, ExtentArenaRange,
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalRecordFormatDeclaration,
    SelectedRecordRouteMetadata,
};

pub const EXTENT_COPY_DOMAIN: &[u8] = b"store.physical.extent-copy.v1";
pub const EXTENT_COPY_V2_DOMAIN: &[u8] = b"store.physical.extent-copy.v2";

/// Classifies either durable extent-copy wire generation before a caller
/// chooses the bounded version-aware decoder. This is not validity proof.
pub fn payload_is_extent_copy_any(bytes: &[u8]) -> bool {
    bytes.starts_with(EXTENT_COPY_DOMAIN) || bytes.starts_with(EXTENT_COPY_V2_DOMAIN)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalExtentCopyIntent {
    operation: [u8; 32],
    source_root: u64,
    source: DurableExtentRecordPlacement,
    destination: DurableExtentRecordPlacement,
    alignment: u64,
    maximum_frame_bytes: u32,
    chunk_count: u32,
    source_digest: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalExtentCopyResolutionKind {
    Cancelled,
    Published {
        root_generation: u64,
        publication_lsn: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalExtentCopyResolution {
    operation: [u8; 32],
    intent_digest: [u8; 32],
    intent_lsn: u64,
    kind: PhysicalExtentCopyResolutionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalExtentCopyRecord {
    Intent(PhysicalExtentCopyIntent),
    Resolved(PhysicalExtentCopyResolution),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalExtentCopyDenial {
    Domain,
    Length,
    Tag,
    Identity,
    Geometry,
}

impl PhysicalExtentCopyIntent {
    pub fn new(
        format: PhysicalRecordFormatDeclaration,
        operation: [u8; 32],
        source_root: u64,
        source: DurableExtentRecordPlacement,
        destination: ExtentArenaRange,
        alignment: u64,
        source_digest: [u8; 32],
    ) -> Option<Self> {
        Self::new_with_target_tier(
            format,
            operation,
            source_root,
            source,
            destination,
            alignment,
            source_digest,
            source.tier_class(),
        )
    }

    /// Encodes a requested destination tier; the Store owner must authenticate
    /// the selected source and durable arena epoch before effectful reservation.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_target_tier(
        format: PhysicalRecordFormatDeclaration,
        operation: [u8; 32],
        source_root: u64,
        source: DurableExtentRecordPlacement,
        destination: ExtentArenaRange,
        alignment: u64,
        source_digest: [u8; 32],
        target_tier: crate::PhysicalTierClass,
    ) -> Option<Self> {
        if operation == [0; 32]
            || source_root == 0
            || source.arena_range().arena() == destination.arena()
        {
            return None;
        }
        let maximum_frame_bytes = format.page_size().bytes();
        let overhead =
            crate::DURABLE_EXTENT_FRAME_HEADER_BYTES + crate::EXTENT_CHUNK_METADATA_BYTES;
        let payload = u64::from(maximum_frame_bytes).checked_sub(overhead as u64)?;
        let chunk_count = u32::try_from(source.payload_bytes().div_ceil(payload)).ok()?;
        let layout = ExtentArenaFrameLayout::new(format, alignment)?;
        if !layout.admits(source.arena_range(), chunk_count)
            || !layout.admits(destination, chunk_count)
        {
            return None;
        }
        let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(source.extent())
            .with_extent_generation(
                PhysicalGeneration::from_raw(source.extent_generation().checked_add(1)?).ok()?,
            );
        let destination_metadata =
            SelectedRecordRouteMetadata::new(source.content_class(), target_tier)?;
        let destination = DurableExtentRecordPlacement::new_selected(
            source.record(),
            cell,
            source.payload_bytes(),
            destination,
            destination_metadata,
        )?;
        let value = Self {
            operation,
            source_root,
            source,
            destination,
            alignment,
            maximum_frame_bytes,
            chunk_count,
            source_digest,
        };
        value.source_manifest(format)?;
        Some(value)
    }

    pub const fn operation(self) -> [u8; 32] {
        self.operation
    }
    /// The source-root protection basis, never a final publication generation.
    pub const fn source_root(self) -> u64 {
        self.source_root
    }
    pub const fn source(self) -> DurableExtentRecordPlacement {
        self.source
    }
    pub const fn destination(self) -> DurableExtentRecordPlacement {
        self.destination
    }
    pub const fn alignment(self) -> u64 {
        self.alignment
    }
    pub const fn maximum_frame_bytes(self) -> u32 {
        self.maximum_frame_bytes
    }
    pub const fn chunk_count(self) -> u32 {
        self.chunk_count
    }
    pub const fn source_digest(self) -> [u8; 32] {
        self.source_digest
    }

    pub fn source_manifest(
        self,
        format: PhysicalRecordFormatDeclaration,
    ) -> Option<DurableExtentManifest> {
        self.manifest(format, self.source)
    }
    pub fn destination_manifest(
        self,
        format: PhysicalRecordFormatDeclaration,
    ) -> Option<DurableExtentManifest> {
        self.manifest(format, self.destination)
    }
    fn manifest(
        self,
        format: PhysicalRecordFormatDeclaration,
        placement: DurableExtentRecordPlacement,
    ) -> Option<DurableExtentManifest> {
        DurableExtentManifest::new(
            format,
            placement.record(),
            placement.extent_cell(),
            placement.payload_bytes(),
            self.maximum_frame_bytes,
            self.chunk_count,
            self.alignment,
        )
    }
}

impl PhysicalExtentCopyResolution {
    pub fn new(
        operation: [u8; 32],
        intent_digest: [u8; 32],
        intent_lsn: u64,
        kind: PhysicalExtentCopyResolutionKind,
    ) -> Option<Self> {
        if operation == [0; 32] || intent_lsn == 0 {
            return None;
        }
        if let PhysicalExtentCopyResolutionKind::Published {
            root_generation,
            publication_lsn,
        } = kind
        {
            if root_generation == 0 || publication_lsn <= intent_lsn {
                return None;
            }
        }
        Some(Self {
            operation,
            intent_digest,
            intent_lsn,
            kind,
        })
    }
    pub const fn operation(self) -> [u8; 32] {
        self.operation
    }
    pub const fn intent_digest(self) -> [u8; 32] {
        self.intent_digest
    }
    pub const fn intent_lsn(self) -> u64 {
        self.intent_lsn
    }
    pub const fn kind(self) -> PhysicalExtentCopyResolutionKind {
        self.kind
    }
}
