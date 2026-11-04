//! Resident description of the native identity's shared entity key.
use super::WorthQueryInvariantEntityIdentity;
use worth_foundational::facade::ChargedBytes;

impl<Schema, Entity> ChargedBytes for WorthQueryInvariantEntityIdentity<Schema, Entity> {
    /// Count this held path's complete Arc allocation. Other held paths may
    /// describe the same allocation; this makes no global deduplication claim.
    fn additional_charged_bytes(&self) -> u64 {
        std::alloc::Layout::array::<u8>(self.entity.len())
            .ok()
            .and_then(|payload| std::alloc::Layout::new::<[usize; 2]>().extend(payload).ok())
            .and_then(|(layout, _)| u64::try_from(layout.pad_to_align().size()).ok())
            .unwrap_or(u64::MAX)
    }
}
