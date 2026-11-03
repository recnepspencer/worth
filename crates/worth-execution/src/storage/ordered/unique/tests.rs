use super::*;
use crate::{CancellationToken, SerialMemoryBudget, SerialRequest};
use std::{
    alloc::Layout,
    collections::{BTreeMap, BTreeSet},
};

mod aliases;

struct Fixture {
    budget: SerialMemoryBudget,
    serial: SerialRequest,
}
impl Fixture {
    fn new() -> Self {
        let budget = SerialMemoryBudget::new(Some(1_000_000));
        let serial = SerialRequest::from_memory(budget.clone(), CancellationToken::new(), None);
        Self { budget, serial }
    }
    fn request(&self) -> ExecutionRequest<'_, '_> {
        ExecutionRequest::serial(&self.serial)
    }
    fn cell(&self, key: u64) -> ExecutionOrderedMapCell<u64, u64> {
        ExecutionOrderedMapCell::prepare_for_request(key, key * 7, self.request()).unwrap()
    }
    fn map(&self, keys: &[u64]) -> ExecutionOrderedMap<u64, u64> {
        let mut map = ExecutionOrderedMap::default();
        for &key in keys {
            map.insert_vacant_prepared(self.cell(key)).unwrap();
        }
        map
    }
}

fn arc_bill<T>() -> u64 {
    Layout::new::<[usize; 2]>()
        .extend(Layout::new::<T>())
        .unwrap()
        .0
        .pad_to_align()
        .size() as u64
}
fn cell_bill() -> u64 {
    arc_bill::<node::Entry<u64, u64>>() + arc_bill::<node::Node<u64, u64>>()
}
fn identities(map: &ExecutionOrderedMap<u64, u64>) -> BTreeSet<(u8, usize)> {
    fn visit(root: &node::Link<u64, u64>, ids: &mut BTreeSet<(u8, usize)>) {
        if let Some(node) = root {
            assert!(ids.insert((0, Arc::as_ptr(node) as usize)));
            assert!(ids.insert((1, Arc::as_ptr(&node.entry) as usize)));
            visit(&node.left, ids);
            visit(&node.right, ids);
        }
    }
    let mut ids = BTreeSet::new();
    visit(&map.root, &mut ids);
    ids
}
fn cell_identities(cell: &ExecutionOrderedMapCell<u64, u64>) -> [(u8, usize); 2] {
    assert_eq!(cell.node.bytes, Some(cell_bill()));
    singleton_identities(cell)
}
fn singleton_identities(cell: &ExecutionOrderedMapCell<u64, u64>) -> [(u8, usize); 2] {
    assert!(cell.node.left.is_none() && cell.node.right.is_none());
    assert_eq!((cell.node.count, cell.node.height), (1, 1));
    [
        (0, Arc::as_ptr(&cell.node) as usize),
        (1, Arc::as_ptr(&cell.node.entry) as usize),
    ]
}
fn assert_map(map: &ExecutionOrderedMap<u64, u64>, expected: &BTreeMap<u64, u64>) {
    fn topology(root: &node::Link<u64, u64>) -> (u8, usize) {
        let Some(node) = root else { return (0, 0) };
        let (left_height, left_count) = topology(&node.left);
        let (right_height, right_count) = topology(&node.right);
        let count = left_count + right_count + 1;
        assert!(left_height.abs_diff(right_height) <= 1);
        assert_eq!(node.height, left_height.max(right_height) + 1);
        assert_eq!(node.count, count);
        assert_eq!(node.bytes, Some(count as u64 * cell_bill()));
        (node.height, count)
    }
    assert_eq!(topology(&map.root).1, expected.len());
    assert_eq!(map.len(), expected.len());
    assert_eq!(
        map.iter()
            .map(|(k, v)| (*k, *v))
            .collect::<BTreeMap<_, _>>(),
        *expected
    );
    for (rank, (key, value)) in expected.iter().enumerate() {
        assert_eq!(map.rank_of(key), Some(rank));
        assert_eq!(map.entry_at(rank), Some((key, value)));
    }
    assert_eq!(map.entry_at(expected.len()), None);
    assert_eq!(
        map.accounted_bytes(),
        Some(expected.len() as u64 * cell_bill())
    );
}

#[test]
fn prepared_cells_and_rotations_preserve_original_owners_and_grants() {
    // These cover LL/RR/LR/RL insertion and deletion rotations, then a larger
    // differently ordered walk whose every intermediate AVL metadata is checked.
    for keys in [
        vec![3, 2, 1],
        vec![1, 2, 3],
        vec![3, 1, 2],
        vec![1, 3, 2],
        (0..64).map(|i| (i * 37) % 64).collect(),
    ] {
        let fixture = Fixture::new();
        let mut cells = Vec::with_capacity(keys.len());
        let mut removed = Vec::with_capacity(keys.len());
        let mut all_ids = BTreeSet::new();
        for key in &keys {
            let cell = fixture.cell(*key);
            all_ids.extend(cell_identities(&cell));
            cells.push(cell);
        }
        let held = keys.len() as u64 * cell_bill();
        assert_eq!(fixture.budget.charged_memory_bytes(), held);
        let full = fixture.budget.reserve(1_000_000 - held).unwrap();
        let mut map = ExecutionOrderedMap::default();
        let mut expected = BTreeMap::new();
        let mut installed_ids = BTreeSet::new();
        for cell in cells {
            installed_ids.extend(cell_identities(&cell));
            expected.insert(*cell.key(), *cell.value());
            map.insert_vacant_prepared(cell).unwrap();
            assert_map(&map, &expected);
            assert_eq!(identities(&map), installed_ids);
            assert_eq!(fixture.budget.charged_memory_bytes(), 1_000_000);
        }
        for key in &keys {
            assert_eq!(map.set_existing_unique(key, key * 11), Ok(Some(key * 7)));
            expected.insert(*key, key * 11);
        }
        assert_map(&map, &expected);
        assert_eq!(identities(&map), all_ids);
        let mut removed_ids = BTreeSet::new();
        for key in keys.iter().rev() {
            let mut selected = map.root.as_ref().unwrap();
            while selected.entry.key != *key {
                selected = if *key < selected.entry.key {
                    selected.left.as_ref().unwrap()
                } else {
                    selected.right.as_ref().unwrap()
                };
            }
            let original_pair = [
                (0, Arc::as_ptr(selected) as usize),
                (1, Arc::as_ptr(&selected.entry) as usize),
            ];
            let cell = map.remove_retaining_unique(key).unwrap().unwrap();
            assert_eq!((*cell.key(), *cell.value()), (*key, key * 11));
            assert_eq!(cell_identities(&cell), original_pair);
            removed_ids.extend(cell_identities(&cell));
            removed.push(cell);
            expected.remove(key);
            assert_map(&map, &expected);
            assert_eq!(
                identities(&map)
                    .union(&removed_ids)
                    .copied()
                    .collect::<BTreeSet<_>>(),
                all_ids
            );
            assert_eq!(fixture.budget.charged_memory_bytes(), 1_000_000);
        }
        assert!(map.remove_retaining_unique(&99).unwrap().is_none());
        drop(map);
        drop(full);
        assert_eq!(fixture.budget.charged_memory_bytes(), held);
        // Each detached original node/entry still owns its exact reservations.
        for (index, cell) in removed.into_iter().enumerate() {
            drop(cell);
            assert_eq!(
                fixture.budget.charged_memory_bytes(),
                held - (index as u64 + 1) * cell_bill()
            );
        }
    }
}

#[derive(Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CopyWithoutClone(u64);
#[expect(
    clippy::non_canonical_clone_impl,
    reason = "This adversarial key panics on Clone to prove the owner uses Copy"
)]
impl Clone for CopyWithoutClone {
    fn clone(&self) -> Self {
        panic!("Copy-only map owners must never invoke custom Clone")
    }
}
#[test]
fn unique_edits_never_invoke_copy_clone() {
    let fixture = Fixture::new();
    let mut map = ExecutionOrderedMap::default();
    for key in [4, 2, 6, 1, 3, 5, 7] {
        let cell = ExecutionOrderedMapCell::prepare_for_request(
            CopyWithoutClone(key),
            CopyWithoutClone(key * 3),
            fixture.request(),
        )
        .unwrap();
        map.insert_vacant_prepared(cell).unwrap();
    }
    assert_eq!(
        map.set_existing_unique(&CopyWithoutClone(4), CopyWithoutClone(91)),
        Ok(Some(CopyWithoutClone(12)))
    );
    let removed = map
        .remove_retaining_unique(&CopyWithoutClone(4))
        .unwrap()
        .unwrap();
    assert_eq!(removed.value(), &CopyWithoutClone(91));
    assert_eq!(
        map.keys().map(|k| k.0).collect::<Vec<_>>(),
        [1, 2, 3, 5, 6, 7]
    );
    drop(removed);
    drop(map);
    assert_eq!(fixture.budget.charged_memory_bytes(), 0);
}

#[test]
fn ordinary_preparation_keeps_its_explicit_allocation_contract() {
    let cell = ExecutionOrderedMapCell::prepare_in(
        5u64,
        9u64,
        ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    // Outside a run, SystemAllocation retains measured layout bookkeeping;
    // that Some value does not claim an execution reservation.
    assert_eq!(cell.node.bytes, Some(cell_bill()));
    let ids = singleton_identities(&cell);
    let mut map = ExecutionOrderedMap::default();
    map.insert_vacant_prepared(cell).unwrap();
    assert_eq!(map.set_existing_unique(&5, 10), Ok(Some(9)));
    let cell = map.remove_retaining_unique(&5).unwrap().unwrap();
    assert_eq!(singleton_identities(&cell), ids);
    assert_eq!(cell.node.bytes, Some(cell_bill()));
    assert_eq!(*cell.value(), 10);
}

#[test]
fn prepared_cell_second_reservation_refuses_with_original_cause_and_refund() {
    let fixture = Fixture::new();
    let request = fixture.request();
    let entry_bill = arc_bill::<node::Entry<u64, u64>>();
    let node_bill = arc_bill::<node::Node<u64, u64>>();
    let mut before_node_reservation = 0;
    // Observe the public nested controller at the same second-reservation cut,
    // with its already admitted entry ticket alive. No private kernel constant.
    request
        .run_native_with_checkpoint(|_| {
            let entry = request.reserve_retained_memory(entry_bill).unwrap();
            request
                .run_native_with_checkpoint(|_| {
                    before_node_reservation = fixture.budget.charged_memory_bytes();
                })
                .unwrap();
            drop(entry);
        })
        .unwrap();
    assert!(before_node_reservation >= entry_bill);
    assert_eq!(fixture.budget.charged_memory_bytes(), 0);
    let budget = SerialMemoryBudget::new(Some(before_node_reservation + node_bill - 1));
    let serial = SerialRequest::from_memory(budget.clone(), CancellationToken::new(), None);
    let denial =
        ExecutionOrderedMapCell::prepare_for_request(1u64, 7u64, ExecutionRequest::serial(&serial))
            .unwrap_err();
    assert!(matches!(denial, ExecutionOrderedMapDenial::Request(
        crate::WorkCeilingDenial::Admission(crate::LeaseDenial::MemoryExhausted(
            crate::MemoryLimitDenial::Capacity { requested, .. }
        ))
    ) if requested == node_bill));
    assert_eq!(budget.charged_memory_bytes(), 0);
}
