//! Where one chunk of an extent lies in its arena range and how long its
//! frame is. Every reader of an extent chunk derives both here, from the
//! manifest that declares the extent and the layout that places its frames.

use super::ExtentArenaFrameLayout;
use crate::{
    DurableExtentManifest, ExtentChunkCoordinate, DURABLE_EXTENT_FRAME_HEADER_BYTES,
    EXTENT_CHUNK_METADATA_BYTES,
};

/// Chunk `ordinal` of an extent: the coordinate its frame carries, the frame's
/// offset within the extent's arena range, and the frame's exact length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtentChunkFrame {
    coordinate: ExtentChunkCoordinate,
    offset: u64,
    payload_bytes: u32,
}

impl ExtentChunkFrame {
    /// Chunk `ordinal` of `manifest`, placed by `layout`. `None` for an
    /// ordinal outside the manifest's chunks, or a placement past every
    /// offset.
    pub fn of(
        manifest: DurableExtentManifest,
        layout: ExtentArenaFrameLayout,
        ordinal: u32,
    ) -> Option<Self> {
        if ordinal > manifest.chunk_count() {
            return None;
        }
        let capacity = manifest.chunk_payload_capacity();
        let logical_offset = u64::from(ordinal.checked_sub(1)?).checked_mul(u64::from(capacity))?;
        let coordinate = ExtentChunkCoordinate::new(
            manifest.record(),
            manifest.extent_cell(),
            manifest.logical_bytes(),
            logical_offset,
            ordinal,
        )?;
        let remaining = manifest.logical_bytes() - logical_offset;
        let payload_bytes = u32::try_from(remaining.min(u64::from(capacity))).ok()?;
        Some(Self {
            coordinate,
            offset: layout.chunk_offset(ordinal)?,
            payload_bytes,
        })
    }

    pub const fn coordinate(self) -> ExtentChunkCoordinate {
        self.coordinate
    }

    /// The frame's offset from the start of the extent's arena range.
    pub const fn offset(self) -> u64 {
        self.offset
    }

    /// The chunk's payload bytes, after its frame header and metadata.
    pub const fn payload_bytes(self) -> u32 {
        self.payload_bytes
    }

    /// The frame's exact length: header, metadata and payload. A manifest's
    /// payload capacity leaves room for both, so it never passes a page.
    pub const fn length(self) -> u32 {
        (DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES) as u32
            + self.payload_bytes
    }
}
