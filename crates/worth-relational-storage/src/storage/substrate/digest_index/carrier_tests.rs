//! Evaluation only: union actual retained owners, excluding transient test Vecs/maps.
use super::*;
use crate::storage::substrate::{
    SharedMap, StorageAllocation, StorageAllocationObservation, StorageAllocationVisitor,
    StoredValue,
};
use serde::Serialize;
use std::collections::BTreeMap;

type Old<K> = SharedMap<K, StorageAllocation<DigestIndex>>;
type New<K> = SharedMap<K, DigestIndex>;
#[derive(Default)]
struct Owners(BTreeMap<u64, u64>);
impl StorageAllocationVisitor for Owners {
    fn visit(&mut self, allocation: StorageAllocationObservation) -> bool {
        match self.0.insert(allocation.id, allocation.bytes) {
            Some(bytes) => {
                assert_eq!(bytes, allocation.bytes);
                false
            }
            None => true,
        }
    }
}
impl Owners {
    fn bytes(&self) -> u64 {
        self.0.values().sum()
    }
    fn uniform_size(&self) -> Option<u64> {
        let first = self.0.values().next().copied()?;
        assert!(self.0.values().all(|bytes| *bytes == first));
        Some(first)
    }
}
#[derive(Default)]
struct Union {
    nodes: Owners,
    headers: Owners,
    children: Owners,
}
fn collect<K: Ord + Copy, V: StoredValue<Value = DigestIndex>>(roots: &[SharedMap<K, V>]) -> Union {
    let mut union = Union::default();
    for root in roots {
        root.visit_allocations(false, &mut union.nodes, &mut |value, header, _nodes| {
            if header.visit(&mut union.headers) {
                value.visit_allocations(&mut union.children);
            }
        });
    }
    union
}
#[derive(Serialize)]
struct Measurement<'a> {
    axis: &'a str,
    scenario: &'a str,
    retained_roots: usize,
    old_map_nodes: usize,
    new_map_nodes: usize,
    old_map_node_bytes: Option<u64>,
    new_map_node_bytes: Option<u64>,
    old_unique_header_arcs: usize,
    header_arc_bytes: u64,
    shared_child_owners: usize,
    shared_child_bytes: u64,
    old_total_bytes: u64,
    new_total_bytes: u64,
    new_minus_old_bytes: i128,
}
fn report<K: Ord + Copy>(axis: &str, scenario: &str, old: &[Old<K>], new: &[New<K>]) {
    assert_eq!(old.len(), new.len());
    let retained_roots = old.len();
    let old = collect(old);
    let new = collect(new);
    assert_eq!(old.nodes.0.len(), new.nodes.0.len());
    assert_eq!(
        old.children.0, new.children.0,
        "both representations hold the SAME real payload producer allocations"
    );
    assert!(new.headers.0.is_empty());
    let header_arc = StorageAllocation::<DigestIndex>::layout_bytes();
    if let Some(bytes) = old.headers.uniform_size() {
        assert_eq!(bytes, header_arc);
    }
    let old_node = old.nodes.uniform_size();
    let new_node = new.nodes.uniform_size();
    if let (Some(old), Some(new)) = (old_node, new_node) {
        assert_eq!(
            new - old,
            (std::mem::size_of::<DigestIndex>()
                - std::mem::size_of::<StorageAllocation<DigestIndex>>()) as u64
        );
    }
    let old_total = old.nodes.bytes() + old.headers.bytes() + old.children.bytes();
    let new_total = new.nodes.bytes() + new.children.bytes();
    let measurement = Measurement {
        axis,
        scenario,
        retained_roots,
        old_map_nodes: old.nodes.0.len(),
        new_map_nodes: new.nodes.0.len(),
        old_map_node_bytes: old_node,
        new_map_node_bytes: new_node,
        old_unique_header_arcs: old.headers.0.len(),
        header_arc_bytes: header_arc,
        shared_child_owners: old.children.0.len(),
        shared_child_bytes: old.children.bytes(),
        old_total_bytes: old_total,
        new_total_bytes: new_total,
        new_minus_old_bytes: new_total as i128 - old_total as i128,
    };
    eprintln!(
        "DIGEST_HEADER_MEASUREMENT {}",
        serde_json::to_string(&measurement).unwrap()
    );
}
fn value(slot: usize, version: usize) -> [u8; 32] {
    let mut value = [0u8; 32];
    value[..8].copy_from_slice(&u64::try_from(slot).unwrap().to_be_bytes());
    value[8..16].copy_from_slice(&u64::try_from(version).unwrap().to_be_bytes());
    value
}
fn payload(slot: usize) -> DigestIndex {
    let mut payload = DigestIndex::default();
    payload.set(0, Some(value(slot, 0))).unwrap();
    payload
}
fn dense<K: Ord + Copy>(key: fn(usize) -> K) -> (Old<K>, New<K>) {
    let values = (0..64)
        .map(|slot| (key(slot), payload(slot)))
        .collect::<Vec<_>>();
    (
        Old::from_sorted_unique(values.clone()),
        New::from_sorted_unique(values),
    )
}
fn axis<K: Ord + Copy>(name: &str, key: fn(usize) -> K) {
    report(
        name,
        "empty",
        &[Old::<K>::default()],
        &[New::<K>::default()],
    );
    let mut old = Old::default();
    let mut new = New::default();
    old.insert(key(0), DigestIndex::default());
    new.insert(key(0), DigestIndex::default());
    report(name, "singleton_default", &[old], &[new]);
    let first = payload(0);
    let mut old = Old::default();
    let mut new = New::default();
    old.insert(key(0), first.clone());
    new.insert(key(0), first);
    report(name, "singleton_populated", &[old], &[new]);
    let first = payload(0);
    let frontier = first.append_frontier(Some(value(0, 0))).unwrap();
    let opened = DigestIndex::from_append_frontier(&frontier).unwrap();
    let mut old = Old::default();
    let mut new = New::default();
    old.insert(key(0), opened.clone());
    new.insert(key(0), opened);
    report(name, "singleton_frontier", &[old], &[new]);
    let (old, new) = dense(key);
    report(name, "dense_64", &[old], &[new]);
    let mut old = Old::default();
    let mut new = New::default();
    let mut old_roots = Vec::new();
    let mut new_roots = Vec::new();
    for slot in 0..64 {
        let next = payload(slot);
        old.insert(key(slot), next.clone());
        new.insert(key(slot), next);
        old_roots.push(old.clone());
        new_roots.push(new.clone());
    }
    drop((old, new));
    report(name, "retained_append_64", &old_roots, &new_roots);
    let (mut old, mut new) = dense(key);
    let mut old_roots = vec![old.clone()];
    let mut new_roots = vec![new.clone()];
    for version in 1..=128 {
        assert!(old
            .get(&key(63))
            .unwrap()
            .same_storage(new.get(&key(63)).unwrap()));
        let mut next = old.get(&key(63)).unwrap().clone();
        next.set(version as u128, Some(value(63, version))).unwrap();
        old.insert(key(63), next.clone());
        new.insert(key(63), next);
        old_roots.push(old.clone());
        new_roots.push(new.clone());
    }
    drop((old, new));
    report(name, "retained_hot_history_128", &old_roots, &new_roots);
}
#[test]
fn inline_digest_headers_measure_empty_singleton_dense_and_retained_owners() {
    // These are the actual keys of RecordContentIndex.histories and
    // AdjacencyContentIndex.memberships. Neither is a fixed-capacity column.
    axis("record_history_u64", |slot| u64::try_from(slot).unwrap());
    axis("membership_usize", |slot| slot);
}
#[test]
fn inline_digest_headers_preserve_child_identity_and_clone_isolation() {
    let full = payload(0);
    let frontier = full.append_frontier(Some(value(0, 0))).unwrap();
    let mut changed = DigestIndex::from_append_frontier(&frontier).unwrap();
    let old = changed.clone();
    assert!(old.same_storage(&changed));
    let before = old.digest();
    changed.set(1, Some(value(0, 1))).unwrap();
    assert_eq!(old.digest(), before);
    assert_ne!(changed.digest(), before);
    assert_ne!(
        old.root.as_ref().map(StorageAllocation::id),
        changed.root.as_ref().map(StorageAllocation::id)
    );
    assert_ne!(
        old.history_last_value.as_ref().map(StorageAllocation::id),
        changed
            .history_last_value
            .as_ref()
            .map(StorageAllocation::id)
    );
    drop(changed);
    assert_eq!(old.append_frontier(None).unwrap(), frontier);
}
#[test]
fn inline_digest_headers_preserve_commitment_and_frontier_wire() {
    #[derive(Serialize)]
    struct LegacyOpening {
        siblings: Vec<crate::storage::substrate::HistoryFrontierSibling>,
        last: Option<(u128, worth_foundational::CanonicalDigestId)>,
    }
    let source = payload(4);
    let opening = source.append_frontier(Some(value(4, 0))).unwrap();
    let expected = LegacyOpening {
        siblings: opening.siblings.clone(),
        last: opening.last,
    };
    let restored = DigestIndex::from_append_frontier(&opening).unwrap();
    let mut old = Old::<u64>::default();
    let mut new = New::<u64>::default();
    old.insert(4, restored.clone());
    new.insert(4, restored);
    for index in [old.get(&4).unwrap(), new.get(&4).unwrap()] {
        assert_eq!(index.digest(), source.digest());
        let opening = index.append_frontier(None).unwrap();
        assert_eq!(
            rmp_serde::to_vec(&opening).unwrap(),
            rmp_serde::to_vec(&expected).unwrap()
        );
        assert_eq!(
            rmp_serde::to_vec_named(&opening).unwrap(),
            rmp_serde::to_vec_named(&expected).unwrap()
        );
    }
}
