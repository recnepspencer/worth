// A retained subset pays before cloning and uses a stable two-pass predicate.
#[allow(unused_imports)]
use crate::keyed_kept_admission::ordinary_context::*;
use std::cell::Cell;
use {
    worth_execution::KeyedItem, worth_execution::KeyedPartitioner, worth_execution::PartitionItemId,
};

#[test]
fn kept_refuses_before_key_clones_and_rechecks_the_stable_predicate_only_after_admission() {
    let clones = std::rc::Rc::new(std::cell::Cell::new(0));
    let mut keyed = worth_execution::KeyedPartitioner::new();
    for id in 0..12 {
        keyed
            .upsert(
                KeyedItem {
                    item: PartitionItemId(id),
                    key: Key((id % 3) as u8, std::rc::Rc::clone(&clones), vec![0; 100]),
                    partition: worth_foundational::PartitionIdentity::new(id % 3),
                },
                |_| Ok::<(), ()>(()),
            )
            .unwrap();
    }
    clones.set(0);
    let calls = std::cell::Cell::new(0);
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
    assert!(matches!(
        refused,
        Err(worth_execution::KeyedEditDenial::Admission(_))
    ));
    assert_eq!(clones.get(), 0);
    assert_eq!(
        calls.get(),
        12,
        "a refusal visits each retained member only to count its bound"
    );
    let calls: [Cell<usize>; 12] = std::array::from_fn(|_| std::cell::Cell::new(0));
    let admitted = std::cell::Cell::new(false);
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
