//! A component partitioner edited one step at a time equals one built fresh
//! from the same items and edges after every step, and a step leaves every
//! island it does not touch with its identity and its members.

use std::collections::{BTreeMap, BTreeSet};

use worth_execution::{ComponentPartitioner, PartitionItemId};
use worth_foundational::PartitionIdentity;

type Islands = BTreeMap<PartitionIdentity, BTreeSet<u64>>;

/// Every island by its identity, read through `route` and checked against
/// `members`.
fn islands(partitioner: &ComponentPartitioner, items: &BTreeSet<u64>) -> Islands {
    let mut islands = Islands::new();
    for item in items {
        let identity = partitioner
            .route(PartitionItemId(*item))
            .expect("every item routes");
        islands.entry(identity).or_default().insert(*item);
    }
    for (identity, members) in &islands {
        let held = partitioner
            .members(*identity)
            .expect("every routed island has members")
            .iter()
            .map(|item| item.0)
            .collect::<BTreeSet<_>>();
        assert_eq!(&held, members, "members and routes agree");
    }
    islands
}

fn fresh(items: &BTreeSet<u64>, edges: &BTreeSet<(u64, u64)>) -> ComponentPartitioner {
    let mut partitioner = ComponentPartitioner::new();
    for item in items {
        partitioner.upsert_item(PartitionItemId(*item));
    }
    for (a, b) in edges {
        partitioner
            .add_edge(PartitionItemId(*a), PartitionItemId(*b))
            .expect("both ends are items");
    }
    partitioner
}

#[derive(Clone, Copy, Debug)]
enum Step {
    AddItem(u64),
    AddEdge(u64, u64),
    RemoveEdge(u64, u64),
    RemoveItem(u64),
}

/// The items whose islands `step` touches, before it runs.
fn touched(step: Step) -> Vec<u64> {
    match step {
        Step::AddItem(_) => Vec::new(),
        Step::AddEdge(a, b) | Step::RemoveEdge(a, b) => vec![a, b],
        Step::RemoveItem(item) => vec![item],
    }
}

#[test]
fn every_retained_partitioner_equals_a_fresh_one_and_untouched_islands_keep_identity() {
    for seed in 1..=8_u64 {
        let mut state = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
        let mut next = move |bound: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % bound
        };
        let mut retained = ComponentPartitioner::new();
        let mut items = BTreeSet::<u64>::new();
        let mut edges = BTreeSet::<(u64, u64)>::new();
        for _ in 0..400 {
            let present = items.iter().copied().collect::<Vec<u64>>();
            let pick = |roll: u64| present[usize::try_from(roll).unwrap()];
            let count = present.len() as u64;
            let step = match next(4) {
                0 => Step::AddItem(next(24)),
                1 if count >= 2 => {
                    let (a, b) = (pick(next(count)), pick(next(count)));
                    if a == b {
                        Step::AddItem(next(24))
                    } else {
                        Step::AddEdge(a.min(b), a.max(b))
                    }
                }
                2 if !edges.is_empty() => {
                    let edge = *edges
                        .iter()
                        .nth(usize::try_from(next(edges.len() as u64)).unwrap())
                        .unwrap();
                    Step::RemoveEdge(edge.0, edge.1)
                }
                3 if count >= 1 => Step::RemoveItem(pick(next(count))),
                _ => Step::AddItem(next(24)),
            };
            let before = islands(&retained, &items);
            let touched = touched(step)
                .into_iter()
                .filter_map(|item| retained.route(PartitionItemId(item)))
                .collect::<BTreeSet<_>>();
            match step {
                Step::AddItem(item) => {
                    retained.upsert_item(PartitionItemId(item));
                    items.insert(item);
                }
                Step::AddEdge(a, b) => {
                    retained
                        .add_edge(PartitionItemId(a), PartitionItemId(b))
                        .unwrap();
                    edges.insert((a, b));
                }
                Step::RemoveEdge(a, b) => {
                    retained
                        .remove_edge(PartitionItemId(a), PartitionItemId(b))
                        .unwrap();
                    edges.remove(&(a, b));
                }
                Step::RemoveItem(item) => {
                    retained.remove_item(PartitionItemId(item));
                    items.remove(&item);
                    edges.retain(|(a, b)| *a != item && *b != item);
                    assert!(retained.route(PartitionItemId(item)).is_none());
                }
            }
            let after = islands(&retained, &items);
            assert_eq!(
                after,
                islands(&fresh(&items, &edges), &items),
                "seed {seed}: {step:?} leaves what a fresh build leaves"
            );
            for (identity, members) in before.iter().filter(|(id, _)| !touched.contains(id)) {
                assert_eq!(
                    after.get(identity),
                    Some(members),
                    "seed {seed}: {step:?} leaves island {identity:?} as it was"
                );
            }
        }
    }
}
