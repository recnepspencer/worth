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

#[cfg(test)]
mod tests {
    use super::super::*;
    use worth_foundational::facade::ChargedBytes;
    use worth_relational::facade::identity::PartitionId;

    #[test]
    fn native_identity_describes_its_shared_key_without_changing_custody() {
        fn accepts_resident<T: ChargedBytes>(value: &T) -> u64 {
            value.additional_charged_bytes()
        }
        for length in [0, 1, 8, 33, 1_024] {
            // Owner-local storage evidence; this constructs no reader/grant.
            let identity = WorthQueryInvariantEntityIdentity::<(), ()> {
                entity_id: EntityId::new(PartitionId::main(), 3, 1),
                kind: KindId(7),
                entity: Arc::from("x".repeat(length)),
                authority_identity: 19,
                _marker: PhantomData,
            };
            let weak = Arc::downgrade(&identity.entity);
            let clone = identity.clone();
            assert!(Arc::ptr_eq(&identity.entity, &clone.entity));
            assert_eq!(
                accepts_resident(&identity),
                ((2 * size_of::<usize>() + length).next_multiple_of(align_of::<usize>())) as u64
            );
            assert_eq!(accepts_resident(&identity), accepts_resident(&clone));
            assert_eq!(clone.authority_identity, 19);
            drop(identity);
            assert!(weak.upgrade().is_some());
            drop(clone);
            assert!(weak.upgrade().is_none());
        }
    }
}
