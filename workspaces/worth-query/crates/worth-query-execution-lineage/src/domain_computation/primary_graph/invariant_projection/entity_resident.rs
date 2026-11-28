//! Resident description of the native identity's shared entity key.
use super::WorthQueryInvariantEntityIdentity;
use worth_foundational::facade::ChargedBytes;

impl<Schema, Entity> ChargedBytes for WorthQueryInvariantEntityIdentity<Schema, Entity> {
    /// Count this held path's complete Arc allocation. Other held paths may
    /// describe the same allocation; this makes no global deduplication claim.
    fn additional_charged_bytes(&self) -> u64 {
        entity_key_charged_bytes(self.entity.len())
    }
}

fn entity_key_charged_bytes(length: usize) -> u64 {
    std::alloc::Layout::array::<u8>(length)
        .ok()
        .and_then(|payload| std::alloc::Layout::new::<[usize; 2]>().extend(payload).ok())
        .and_then(|(layout, _)| u64::try_from(layout.pad_to_align().size()).ok())
        .unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::entity_key_charged_bytes;

    #[test]
    fn shared_entity_key_charges_arc_header_payload_and_padding() {
        for length in [0, 1, 8, 33, 1_024] {
            assert_eq!(
                entity_key_charged_bytes(length),
                (2 * size_of::<usize>() + length).next_multiple_of(align_of::<usize>()) as u64,
            );
        }
    }
}
