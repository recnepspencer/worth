use super::*;
use crate::storage::substrate::{StorageAllocationObservation, StorageAllocationVisitor};

#[derive(Default)]
struct Allocations(u64);
impl StorageAllocationVisitor for Allocations {
    fn visit(&mut self, allocation: StorageAllocationObservation) -> bool {
        self.0 += allocation.bytes;
        true
    }
}

#[test]
fn insertion_quote_covers_original_shared_path_and_rotation_allocations() {
    for keys in [[3, 2, 1, 4, 5], [3, 1, 2, 5, 4]] {
        let mut map = SharedMap::<u64, u64>::new();
        for key in keys {
            let original = map.clone();
            let quote = map.insertion_allocation_bytes(&key).unwrap();
            map.insert(key, key + 10);
            let mut new = Allocations::default();
            map.visit_new_allocations(
                &original,
                &mut new,
                &mut crate::storage::substrate::visit_new_inline_value,
            );
            assert_eq!(
                new.0, quote,
                "the original retained root coexists with every admitted new owner"
            );
            assert_eq!(original.get(&key), None);
            assert_eq!(map.get(&key), Some(&(key + 10)));
        }
        let original = map.clone();
        let quote = map.insertion_allocation_bytes(&3).unwrap();
        map.insert(3, 99);
        let mut new = Allocations::default();
        map.visit_new_allocations(
            &original,
            &mut new,
            &mut crate::storage::substrate::visit_new_inline_value,
        );
        assert_eq!(new.0, quote);
        assert_eq!(original.get(&3), Some(&13));
        assert_eq!(map.get(&3), Some(&99));
    }
}
