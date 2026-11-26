//! Carried roster delta walk against a copy-on-write routing model (`G7`,
//! mutant `M4`). The model rewrites touched paths exactly like the C.5
//! planner (`manifest_routing/planner/rewrite.rs`): unchanged children keep
//! their reference, rewritten blocks take the successor generation, leaves
//! and branch levels re-chunk at a fixed capacity, empty leaves vanish.
//! The oracle roster comes from the record payloads alone, not the tree.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::carried::{
    carried_in_range, walk_changed_since, RoutingDeltaVisitor, RoutingNode, RoutingSource,
};

const CAPACITY: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ModelRef {
    generation: u64,
    block: u64,
    first: u64,
    last: u64,
}

#[derive(Debug, Clone)]
enum ModelBlock {
    Leaf(Vec<u64>),
    Branch(Vec<ModelRef>),
}

/// Copy-on-write routing tree plus the payload of every live record:
/// `Some(attempt)` for a drop-set manifest, `None` for any other record.
#[derive(Default)]
struct ModelStore {
    blocks: HashMap<(u64, u64), ModelBlock>,
    root: Option<ModelRef>,
    generation: u64,
    next_block: u64,
    payloads: BTreeMap<u64, Option<u64>>,
    reads: u64,
}

type Pin = (u64, u64);

impl ModelStore {
    fn commit(&mut self, upserts: &[(u64, Option<u64>)], drops: &[u64]) {
        self.generation += 1;
        let ups = upserts
            .iter()
            .map(|(record, _)| *record)
            .collect::<BTreeSet<_>>();
        let drops = drops.iter().copied().collect::<BTreeSet<_>>();
        self.payloads.extend(upserts.iter().copied());
        drops.iter().for_each(|record| {
            self.payloads.remove(record);
        });
        let mut level = match self.root {
            None => self.write_leaves(ups.into_iter().collect()),
            Some(root) if ups.is_empty() && drops.is_empty() => vec![root],
            Some(root) => self.rewrite(root, &ups, &drops),
        };
        while level.len() > 1 {
            level = self.write_branches(level);
        }
        self.root = level.first().copied();
    }

    fn rewrite(
        &mut self,
        at: ModelRef,
        ups: &BTreeSet<u64>,
        drops: &BTreeSet<u64>,
    ) -> Vec<ModelRef> {
        match self.blocks[&(at.generation, at.block)].clone() {
            ModelBlock::Leaf(records) => {
                let mut merged = records.into_iter().collect::<BTreeSet<_>>();
                merged.extend(ups.iter().copied());
                drops.iter().for_each(|record| {
                    merged.remove(record);
                });
                self.write_leaves(merged.into_iter().collect())
            }
            ModelBlock::Branch(children) => {
                let mut assigned = vec![(BTreeSet::new(), BTreeSet::new()); children.len()];
                let child_of = |record: &u64| {
                    children
                        .partition_point(|child| child.last < *record)
                        .min(children.len() - 1)
                };
                ups.iter()
                    .for_each(|r| _ = assigned[child_of(r)].0.insert(*r));
                drops
                    .iter()
                    .for_each(|r| _ = assigned[child_of(r)].1.insert(*r));
                let mut rewritten = Vec::new();
                for (child, (child_ups, child_drops)) in children.iter().zip(assigned) {
                    if child_ups.is_empty() && child_drops.is_empty() {
                        rewritten.push(*child);
                    } else {
                        rewritten.extend(self.rewrite(*child, &child_ups, &child_drops));
                    }
                }
                self.write_branches(rewritten)
            }
        }
    }

    fn write_leaves(&mut self, records: Vec<u64>) -> Vec<ModelRef> {
        records
            .chunks(CAPACITY)
            .map(|chunk| {
                self.stage(
                    chunk[0],
                    chunk[chunk.len() - 1],
                    ModelBlock::Leaf(chunk.to_vec()),
                )
            })
            .collect()
    }

    fn write_branches(&mut self, children: Vec<ModelRef>) -> Vec<ModelRef> {
        children
            .chunks(CAPACITY)
            .map(|chunk| {
                let (first, last) = (chunk[0].first, chunk[chunk.len() - 1].last);
                self.stage(first, last, ModelBlock::Branch(chunk.to_vec()))
            })
            .collect()
    }

    fn stage(&mut self, first: u64, last: u64, block: ModelBlock) -> ModelRef {
        self.next_block += 1;
        let reference = ModelRef {
            generation: self.generation,
            block: self.next_block,
            first,
            last,
        };
        self.blocks
            .insert((reference.generation, reference.block), block);
        reference
    }

    /// The canonical roster: every live manifest record, ascending.
    fn oracle_roster(&self) -> Vec<Pin> {
        self.payloads
            .iter()
            .filter_map(|(record, attempt)| attempt.map(|attempt| (*record, attempt)))
            .collect()
    }

    /// Delta walk from a roster carried at generation `floor`.
    fn advance(&mut self, carried: &[Pin], floor: u64) -> DeltaOutcome {
        self.reads = 0;
        let payloads = self.payloads.clone();
        let mut roster = ModelRoster {
            carried,
            payloads: &payloads,
            pins: Vec::new(),
            changed: 0,
        };
        let root = self.root;
        walk_changed_since(self, root, floor, &mut roster).unwrap();
        DeltaOutcome {
            pins: roster.pins,
            changed: roster.changed,
            reads: self.reads,
        }
    }
}

struct DeltaOutcome {
    pins: Vec<Pin>,
    changed: u64,
    reads: u64,
}

impl RoutingSource for ModelStore {
    type Ref = ModelRef;
    type Key = u64;
    type Error = std::convert::Infallible;

    fn generation(&self, reference: ModelRef) -> u64 {
        reference.generation
    }

    fn range(&self, reference: ModelRef) -> (u64, u64) {
        (reference.first, reference.last)
    }

    fn read(&mut self, reference: ModelRef) -> Result<RoutingNode<ModelRef, u64>, Self::Error> {
        self.reads += 1;
        Ok(
            match self.blocks[&(reference.generation, reference.block)].clone() {
                ModelBlock::Leaf(records) => RoutingNode::Leaf(records),
                ModelBlock::Branch(children) => RoutingNode::Branch(children),
            },
        )
    }
}

struct ModelRoster<'a> {
    carried: &'a [Pin],
    payloads: &'a BTreeMap<u64, Option<u64>>,
    pins: Vec<Pin>,
    changed: u64,
}

impl RoutingDeltaVisitor<u64> for ModelRoster<'_> {
    type Error = std::convert::Infallible;

    fn retained(&mut self, first: u64, last: u64) -> Result<(), Self::Error> {
        self.pins
            .extend_from_slice(carried_in_range(self.carried, first, last, |pin| pin.0));
        Ok(())
    }

    fn changed(&mut self, record: u64) -> Result<(), Self::Error> {
        self.changed += 1;
        if let Some(Some(attempt)) = self.payloads.get(&record) {
            self.pins.push((record, *attempt));
        }
        Ok(())
    }
}

/// Deterministic xorshift so the commit stream is reproducible.
struct Stream(u64);

impl Stream {
    fn next(&mut self, bound: u64) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 % bound
    }
}

#[test]
fn carried_roster_delta_equals_full_rebuild_across_additions_overwrites_and_removals() {
    let mut store = ModelStore::default();
    let mut stream = Stream(0x9e37_79b9_7f4a_7c15);
    let (mut carried, mut carried_generation) = (Vec::new(), 0);
    let mut attempt = 0;
    for commit in 0..240_u64 {
        let mut upserts = Vec::new();
        for _ in 0..stream.next(12) {
            attempt += 1;
            let manifest = stream.next(3) == 0;
            upserts.push((stream.next(1_500), manifest.then_some(attempt)));
        }
        let live = store.payloads.keys().copied().collect::<Vec<_>>();
        let drops = (0..stream.next(6).min(live.len() as u64))
            .map(|_| live[stream.next(live.len() as u64) as usize])
            .filter(|record| upserts.iter().all(|(upserted, _)| upserted != record))
            .collect::<BTreeSet<_>>();
        store.commit(&upserts, &drops.into_iter().collect::<Vec<_>>());
        if commit % 7 == 0 || commit % 11 == 0 {
            let delta = store.advance(&carried, carried_generation);
            assert_eq!(
                delta.pins,
                store.oracle_roster(),
                "checkpoint after commit {commit}"
            );
            (carried, carried_generation) = (delta.pins, store.generation);
        }
    }
    let last = store.advance(&carried, carried_generation);
    assert_eq!(last.pins, store.oracle_roster(), "final checkpoint");
    let (carried, carried_generation) = (last.pins, store.generation);
    assert!(
        !carried.is_empty(),
        "the stream exercised live manifest pins"
    );
    let idle = store.advance(&carried, carried_generation);
    assert_eq!(
        (idle.pins, idle.reads),
        (carried, 0),
        "an unchanged root reads no block"
    );
}

#[test]
fn carried_roster_cutover_work_tracks_touched_routes_not_total_routes() {
    let measure = |routes: u64| {
        let mut store = ModelStore::default();
        let initial = (1..=routes)
            .map(|ordinal| (ordinal * 2, (ordinal % 37 == 0).then_some(ordinal)))
            .collect::<Vec<_>>();
        store.commit(&initial, &[]);
        let carried = store.oracle_roster();
        let base = store.generation;
        let spot = |eighth: u64| (routes * eighth / 8) * 2;
        let touched = [
            (spot(1), Some(1_000_001)),
            (spot(3), None),
            (spot(5) + 1, Some(1_000_002)),
            (spot(7) + 1, None),
        ];
        store.commit(&touched, &[spot(2), spot(6)]);
        let delta = store.advance(&carried, base);
        assert_eq!(delta.pins, store.oracle_roster(), "{routes} routes");
        let full = store.advance(&[], 0);
        assert_eq!(full.pins, store.oracle_roster());
        (delta.reads, delta.changed, full.reads)
    };
    let (reads_1x, changed_1x, full_1x) = measure(2_000);
    let (reads_10x, changed_10x, full_10x) = measure(20_000);
    let touched_paths = 6;
    assert!(
        full_10x >= 9 * full_1x,
        "the full rebuild is linear: {full_1x} -> {full_10x}"
    );
    assert!(
        reads_10x <= reads_1x + touched_paths,
        "delta reads grow only by one level per touched path: {reads_1x} -> {reads_10x}"
    );
    let changed_bound = touched_paths * 2 * CAPACITY as u64;
    assert!(changed_1x <= changed_bound && changed_10x <= changed_bound);
    assert!(
        reads_10x * 20 < full_10x,
        "delta {reads_10x} vs full {full_10x}"
    );
}

#[test]
fn carried_roster_carries_to_rejects_different_root_at_equal_generation() {
    use worth_store_physical_format::{
        PhysicalGeneration, PhysicalGenerationAuthority, PhysicalRootReference,
    };

    use super::carried::CarriedManifestRoster;

    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let cell = |root_ref: u64, generation: u64| {
        authority
            .root_publication_cell(PhysicalRootReference::from_raw(root_ref).unwrap())
            .with_root_publication_generation(PhysicalGeneration::from_raw(generation).unwrap())
    };
    let carried = CarriedManifestRoster {
        root: cell(2, 2),
        tree_identity: 7,
        pins: Vec::new(),
    };
    assert!(carried.carries_to(cell(2, 2), 7), "same root at same gen");
    assert!(carried.carries_to(cell(3, 3), 7), "strictly newer gen");
    assert!(
        !carried.carries_to(cell(99, 2), 7),
        "different root at equal generation must fall back to full scan"
    );
    assert!(!carried.carries_to(cell(1, 1), 7), "older gen");
    assert!(!carried.carries_to(cell(3, 3), 8), "different tree");
}
