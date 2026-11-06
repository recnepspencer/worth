use std::marker::PhantomData;
use std::sync::Arc;
use worth_foundational::facade::ChargedBytes;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantEntityIdentity;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::PartitionId;

#[test]
fn native_identity_describes_its_shared_key_without_changing_custody() {
    fn accepts_resident<T: ChargedBytes>(value: &T) -> u64 {
        value.additional_charged_bytes()
    }
    for length in [0, 1, 8, 33, 1_024] {
        // Owner-local storage evidence; this constructs no reader/grant.
        let identity = WorthQueryInvariantEntityIdentity::<(), ()>::new_for_test(
            EntityId::new(PartitionId::main(), 3, 1),
            KindId(7),
            Arc::from("x".repeat(length)),
            19,
            PhantomData,
        );
        let weak = Arc::downgrade(identity.entity_for_test());
        let clone = identity.clone();
        assert!(Arc::ptr_eq(
            identity.entity_for_test(),
            clone.entity_for_test()
        ));
        assert_eq!(
            accepts_resident(&identity),
            ((2 * size_of::<usize>() + length).next_multiple_of(align_of::<usize>())) as u64
        );
        assert_eq!(accepts_resident(&identity), accepts_resident(&clone));
        assert_eq!((*clone.authority_identity_for_test()), 19);
        drop(identity);
        assert!(weak.upgrade().is_some());
        drop(clone);
        assert!(weak.upgrade().is_none());
    }
}
