use crate::{ExtentArenaRange, PhysicalRecordFormatDeclaration};

/// Aligned placement of the existing self-identifying extent manifest and chunks.
/// Alignment is supplied by qualified media and persisted in the manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtentArenaFrameLayout {
    alignment: u64,
    manifest_stride: u64,
    chunk_stride: u64,
}

impl ExtentArenaFrameLayout {
    pub fn new(format: PhysicalRecordFormatDeclaration, alignment: u64) -> Option<Self> {
        if !alignment.is_power_of_two() {
            return None;
        }
        Some(Self {
            alignment,
            manifest_stride: align_up(super::EXTENT_ARENA_MANIFEST_FRAME_BYTES as u64, alignment)?,
            chunk_stride: align_up(u64::from(format.page_size().bytes()), alignment)?,
        })
    }

    pub const fn alignment(self) -> u64 {
        self.alignment
    }
    pub const fn manifest_stride(self) -> u64 {
        self.manifest_stride
    }
    pub const fn chunk_stride(self) -> u64 {
        self.chunk_stride
    }
    pub fn allocated_bytes(self, chunk_count: u32) -> Option<u64> {
        if chunk_count == 0 {
            return None;
        }
        self.manifest_stride
            .checked_add(self.chunk_stride.checked_mul(u64::from(chunk_count))?)
    }
    pub fn chunk_offset(self, ordinal: u32) -> Option<u64> {
        self.manifest_stride.checked_add(
            self.chunk_stride
                .checked_mul(u64::from(ordinal.checked_sub(1)?))?,
        )
    }
    pub fn admits(self, range: ExtentArenaRange, chunk_count: u32) -> bool {
        range.offset().is_multiple_of(self.alignment)
            && self.allocated_bytes(chunk_count) == Some(range.length())
    }
}

fn align_up(bytes: u64, alignment: u64) -> Option<u64> {
    bytes
        .checked_add(alignment - 1)
        .map(|value| value & !(alignment - 1))
}
