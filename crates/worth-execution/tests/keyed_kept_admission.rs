//! A retained subset pays before cloning and uses a stable two-pass predicate.
use std::{cell::Cell, cmp::Ordering, rc::Rc};
use worth_execution::{
    ChargedBytes, KeyedEditDenial, KeyedItem, KeyedPartitioner, PartitionItemId,
};
use worth_foundational::PartitionIdentity;

struct Key(u8, Rc<Cell<usize>>, Vec<u8>);
impl Clone for Key {
    fn clone(&self) -> Self {
        self.1.set(self.1.get() + 1);
        Self(self.0, Rc::clone(&self.1), self.2.clone())
    }
}
impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for Key {}
impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Key {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
impl ChargedBytes for Key {
    fn additional_charged_bytes(&self) -> u64 {
        self.2.len() as u64
    }
}

#[test]
fn kept_refuses_before_key_clones_and_rechecks_the_stable_predicate_only_after_admission() {
    let clones = Rc::new(Cell::new(0));
    let mut keyed = KeyedPartitioner::new();
    for id in 0..12 {
        keyed
            .upsert(
                KeyedItem {
                    item: PartitionItemId(id),
                    key: Key((id % 3) as u8, Rc::clone(&clones), vec![0; 100]),
                    partition: PartitionIdentity::new(id % 3),
                },
                |_| Ok::<(), ()>(()),
            )
            .unwrap();
    }
    clones.set(0);
    let calls = Cell::new(0);
    let refused = keyed.kept(
        |_| {
            calls.set(calls.get() + 1);
            true
        },
        |bound| {
            assert_eq!(clones.get(), 0, "bound computation clones no retained key");
            Err(bound)
        },
    );
    assert!(matches!(refused, Err(KeyedEditDenial::Admission(_))));
    assert_eq!(clones.get(), 0);
    assert_eq!(
        calls.get(),
        12,
        "a refusal visits each retained member only to count its bound"
    );
    let calls: [Cell<usize>; 12] = std::array::from_fn(|_| Cell::new(0));
    let admitted = Cell::new(false);
    let mut offered = 0;
    let kept = keyed
        .kept(
            |item| {
                let count = &calls[item.0 as usize];
                if count.get() == 1 {
                    assert!(admitted.get(), "construction follows admission");
                }
                count.set(count.get() + 1);
                item.0 % 2 == 0
            },
            |bound| {
                assert_eq!(clones.get(), 0);
                offered = bound;
                admitted.set(true);
                Ok::<(), ()>(())
            },
        )
        .unwrap();
    assert!(
        calls.iter().all(|count| count.get() == 2),
        "stable keep is counted before admission and evaluated again for the admitted copy"
    );
    assert_eq!(kept.charged_bytes(), Some(offered));
    assert_eq!(
        offered,
        KeyedPartitioner::<Key>::retained_bytes(6, 3, (6 + 2 * 3) * 100).unwrap()
    );
}
