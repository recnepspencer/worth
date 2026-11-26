//! `S2` single-slot fanout: one record whose version history and one hub
//! vertex whose adjacency list each exceed the residency budget stream
//! within it, page by page, and agree with the canonical meaning.
use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::super::canonical_read::{
    read_chunk_canonical, ChunkReadError, ChunkResidency, SlotSelection,
};
use super::super::consolidation::derive_bound_entry;
use super::super::slot_projection::{AdjacencyEdge, CanonicalChunkVisitor, SlotVersion};
use super::super::slot_range::{SlotDomain, SlotRange};
use super::member_store::{accountant, edge, seal, version, MemberStore, SlotFixture};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::residency::working_set::{ResidencyPoolKind, WorkingSetAccountant};

const BUDGET: u64 = 24 * 1024;
const HOT_VERSIONS: u64 = 6_000;
const HUB_EDGES: u64 = 8_000;

fn fixtures() -> Vec<SlotFixture> {
    let quiet = |slot| SlotFixture {
        slot,
        head: Some(version(1, slot)),
        history: vec![version(1, slot + 100)],
        adjacency: vec![edge(40, slot)],
    };
    vec![
        quiet(1),
        SlotFixture {
            slot: 3,
            head: Some(version(2, 3)),
            history: (0..HOT_VERSIONS).map(|v| version(2, v)).collect(),
            adjacency: Vec::new(),
        },
        quiet(5),
        SlotFixture {
            slot: 7,
            head: Some(version(3, 7)),
            history: Vec::new(),
            adjacency: (0..HUB_EDGES)
                .map(|e| edge(41 + (e % 3) as u32, e))
                .collect(),
        },
    ]
}

/// Folds what it visits into a digest and watches the pool's charge.
struct BoundedObserver {
    accountant: Arc<WorkingSetAccountant>,
    sequence: Sha256,
    versions: u64,
    edges: u64,
    peak_charged: u64,
}

impl BoundedObserver {
    fn observe_charge(&mut self) {
        let charged = self.accountant.charged_bytes(ResidencyPoolKind::Historical);
        self.peak_charged = self.peak_charged.max(charged);
    }
}

impl CanonicalChunkVisitor for BoundedObserver {
    type Error = std::convert::Infallible;

    fn slot_begin(&mut self, slot: u64, _head: Option<&SlotVersion>) -> Result<(), Self::Error> {
        self.sequence.update(slot.to_be_bytes());
        Ok(())
    }

    fn history_version(&mut self, version: &SlotVersion) -> Result<(), Self::Error> {
        self.sequence.update(&version.payload);
        self.versions += 1;
        self.observe_charge();
        Ok(())
    }

    fn adjacency_edge(&mut self, edge: &AdjacencyEdge) -> Result<(), Self::Error> {
        self.sequence
            .update(edge.relation.local_slot_value().to_be_bytes());
        self.edges += 1;
        self.observe_charge();
        Ok(())
    }

    fn slot_end(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn expected_sequence(fixtures: &[SlotFixture], only: Option<u64>) -> [u8; 32] {
    let mut sequence = Sha256::new();
    for slot in fixtures
        .iter()
        .filter(|slot| only.is_none_or(|only| only == slot.slot))
    {
        sequence.update(slot.slot.to_be_bytes());
        slot.history
            .iter()
            .for_each(|v| sequence.update(&v.payload));
        slot.adjacency
            .iter()
            .for_each(|e| sequence.update(e.relation.local_slot_value().to_be_bytes()));
    }
    sequence.finalize().into()
}

#[test]
fn long_history_and_hub_adjacency_stream_within_a_smaller_budget() {
    let store = MemberStore::default();
    let fixtures = fixtures();
    let range = SlotRange::new(0, 64).unwrap();
    let sealed = seal(&store, SlotDomain::Entity, 64, range, &fixtures);
    let history_bytes = HOT_VERSIONS * 48;
    let adjacency_bytes = HUB_EDGES * std::mem::size_of::<AdjacencyEdge>() as u64;
    assert!(history_bytes > 4 * BUDGET && adjacency_bytes > 4 * BUDGET);

    for selection in [
        SlotSelection::All,
        SlotSelection::One(3),
        SlotSelection::One(7),
    ] {
        let accountant = accountant(BUDGET, BUDGET, 4096);
        let residency = ChunkResidency::new(&accountant, ResidencyPoolKind::Historical);
        let mut observer = BoundedObserver {
            accountant: Arc::clone(&accountant),
            sequence: Sha256::new(),
            versions: 0,
            edges: 0,
            peak_charged: 0,
        };
        read_chunk_canonical(
            &residency,
            &store,
            SlotDomain::Entity,
            &sealed.binding,
            selection,
            &mut observer,
        )
        .unwrap();
        let only = match selection {
            SlotSelection::All => None,
            SlotSelection::One(slot) => Some(slot),
        };
        assert_eq!(
            <[u8; 32]>::from(observer.sequence.finalize()),
            expected_sequence(&fixtures, only)
        );
        assert!(
            observer.peak_charged <= BUDGET,
            "peak {} > {BUDGET}",
            observer.peak_charged
        );
        let pages = residency.continuation_observation();
        if selection != SlotSelection::One(7) {
            assert!(observer.versions >= HOT_VERSIONS);
        }
        assert!(
            pages.evictions > 0,
            "{selection:?} paged through more than the budget holds"
        );
    }

    let accountant = accountant(BUDGET, BUDGET, 4096);
    let residency = ChunkResidency::new(&accountant, ResidencyPoolKind::Historical);
    let derived =
        derive_bound_entry(&residency, &store, SlotDomain::Entity, &sealed.binding).unwrap();
    assert_eq!(
        derived, sealed.semantic,
        "cold re-derivation equals the sealed commitment"
    );
}

#[test]
fn a_budget_below_one_page_refuses_with_memory_exhausted() {
    let store = MemberStore::default();
    let fixtures = fixtures();
    let sealed = seal(
        &store,
        SlotDomain::Entity,
        64,
        SlotRange::new(0, 64).unwrap(),
        &fixtures,
    );
    let accountant = accountant(256, BUDGET, 4096);
    let residency = ChunkResidency::new(&accountant, ResidencyPoolKind::Historical);
    let mut observer = BoundedObserver {
        accountant: Arc::clone(&accountant),
        sequence: Sha256::new(),
        versions: 0,
        edges: 0,
        peak_charged: 0,
    };
    let refused = read_chunk_canonical(
        &residency,
        &store,
        SlotDomain::Entity,
        &sealed.binding,
        SlotSelection::All,
        &mut observer,
    );
    assert_eq!(refused, Err(ChunkReadError::Member(Cause::MemoryExhausted)));
    assert_eq!(accountant.charged_bytes(ResidencyPoolKind::Historical), 0);
}
