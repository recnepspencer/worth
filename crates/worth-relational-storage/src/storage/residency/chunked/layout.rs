//! The fixed chunk geometry of one region format.

/// Partition boundaries and per-slot spill bounds. One layout is fixed per
/// format; every producer and reader of that format uses the same values, so
/// chunk boundaries never depend on content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkLayout {
    /// Slots per chunk; chunk `k` covers `[k * chunk_slots, (k + 1) * chunk_slots)`.
    pub chunk_slots: u64,
    /// Encoded bytes of one slot's history (or adjacency list) kept inline in
    /// the chunk image; older entries spill into continuation pages.
    pub slot_inline_budget: usize,
    /// Encoded entry bytes carried by one continuation page.
    pub continuation_page_bytes: usize,
    /// Entries carried by one paged-directory page.
    pub directory_page_entries: usize,
    /// Encoded bytes of one chunk image member.
    pub max_chunk_image_bytes: usize,
}

impl ChunkLayout {
    /// The `REG8` geometry.
    pub const REG8: Self = Self {
        chunk_slots: 4096,
        slot_inline_budget: 1024,
        continuation_page_bytes: 4096,
        directory_page_entries: 64,
        max_chunk_image_bytes: 16 * 1024 * 1024,
    };

    /// Largest encoded payload of one slot version. A version must fit one
    /// continuation page and the inline budget, so spilling always progresses.
    pub const fn max_version_payload_bytes(self) -> usize {
        if self.slot_inline_budget < self.continuation_page_bytes {
            self.slot_inline_budget
        } else {
            self.continuation_page_bytes
        }
    }
}
