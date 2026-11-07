//! Private normalization and adoption decisions use values, never runtime roots.
use crate::config::data::AdjacencyPolicy;
use crate::identity::data::{KindId, PartitionId, RelationId};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::PartitionState;
use crate::storage::partition::{AdjacencySet, HistoricalKindCustody, SparseAdjacencyTable};
use crate::storage::residency::{kind_admission, kind_commitment, kind_projection};
use crate::storage::substrate::{EntityArena, RelationArena};
use std::collections::BTreeSet;
use worth_execution::SerialMemoryBudget;

fn peer_partition() -> PartitionState {
    let policy = AdjacencyPolicy {
        backend: crate::config::data::AdjacencyBackend::InlineSmallDegreeAdjacency,
        small_degree_inline_capacity: 4,
    };
    let mut partition = PartitionState {
        partition_id: PartitionId(7),
        adjacency_policy: policy.clone(),
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(0),
        relation_arena: RelationArena::with_capacity(0),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    };
    let relation = RelationId::new(PartitionId(11), 3, 1);
    for (slot, table) in [
        (2, &mut partition.adjacency),
        (5, &mut partition.reverse_adjacency),
    ] {
        let mut set = AdjacencySet::new(&policy);
        set.insert(KindId(9), relation);
        *table.ensure(slot, &policy) = set;
    }
    partition
}

fn normalize(
    partition: &mut PartitionState,
    budget: &SerialMemoryBudget,
) -> Result<super::super::RelationalRegionKindCommitment, Cause> {
    // Actual reservation values fund this fixture without driving SDK work.
    kind_admission::normalize_value(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        partition,
        &mut |_, _| Ok(()),
        &mut |prior, bytes| {
            let custody = super::retained_admission::admit(prior, bytes, &mut |bytes| {
                budget.reserve(bytes).map(Some).map_err(|denial| {
                    crate::execution::denial::lease_denial(
                        worth_execution::LeaseDenial::MemoryExhausted(denial),
                    )
                })
            })
            .expect("positive value fixture funding must admit");
            Ok(custody)
        },
    )
}

#[test]
fn custody_admission_grows_the_same_prior_allocation_without_a_fresh_ticket() {
    let budget = SerialMemoryBudget::new(Some(1 << 20));
    let mut calls = 0;
    let mut reserve = |bytes| {
        calls += 1;
        budget.reserve(bytes).map(Some).map_err(|denial| {
            crate::execution::denial::lease_denial(worth_execution::LeaseDenial::MemoryExhausted(
                denial,
            ))
        })
    };
    let first = super::retained_admission::admit(None, 64, &mut reserve)
        .unwrap()
        .unwrap();
    let grown = super::retained_admission::admit(Some(&first), 32, &mut reserve)
        .unwrap()
        .unwrap();
    assert_eq!(grown.id(), first.id());
    assert_eq!(calls, 1);
    let expected = HistoricalKindCustody::layout_bytes() + 96;
    assert_eq!(grown.lock().unwrap().bytes(), expected);
    assert_eq!(budget.charged_memory_bytes(), expected);
    drop(first);
    assert_eq!(budget.charged_memory_bytes(), expected);
    drop(grown);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn normalization_preserves_literal_peer_claims_and_refuses_conflicting_kinds() {
    let mut partition = peer_partition();
    let budget = SerialMemoryBudget::new(Some(1 << 20));
    let commitment = normalize(&mut partition, &budget).unwrap();
    let claims = kind_projection::canonical_history(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        &partition,
    )
    .unwrap();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].relation, RelationId::new(PartitionId(11), 3, 1));
    assert_eq!(claims[0].kind, KindId(9));
    assert_eq!(claims[0].outgoing, BTreeSet::from([2]));
    assert_eq!(claims[0].incoming, BTreeSet::from([5]));
    assert_eq!(commitment.head, commitment.history);
    partition
        .reverse_adjacency
        .get_mut(5)
        .unwrap()
        .index_historical_kind(KindId(10), RelationId::new(PartitionId(11), 3, 1));
    assert_eq!(
        normalize(&mut partition, &budget),
        Err(Cause::IntegrityMismatch)
    );
    assert_eq!(
        kind_commitment::digest(
            &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
            &[]
        )
        .unwrap(),
        [
            124, 141, 64, 135, 143, 11, 198, 95, 76, 148, 197, 94, 181, 204, 245, 140, 200, 172,
            222, 42, 127, 107, 104, 209, 141, 141, 153, 139, 97, 85, 161, 41
        ]
    );
}

#[test]
fn historical_kind_funding_follows_clones_partial_adoption_and_replacement() {
    let budget = SerialMemoryBudget::new(Some(1 << 20));
    let mut first = peer_partition();
    normalize(&mut first, &budget).unwrap();
    let funded = budget.charged_memory_bytes();
    assert!(funded > 0);
    let mut second = first.clone();
    assert_eq!(budget.charged_memory_bytes(), funded);
    normalize(&mut second, &budget).unwrap();
    let both = budget.charged_memory_bytes();
    assert_eq!(
        both,
        2 * funded,
        "replacement owns fresh, equal-layout tickets"
    );
    drop(first);
    assert_eq!(budget.charged_memory_bytes(), funded);
    let held = second
        .adjacency
        .get(2)
        .unwrap()
        .historical_kind_custody()
        .unwrap()
        .lock()
        .unwrap()
        .bytes();
    let mut adopted = SparseAdjacencyTable::default();
    adopted.merge_slots_from(&second.adjacency, &BTreeSet::from([2]));
    drop(second);
    assert_eq!(budget.charged_memory_bytes(), held);
    drop(adopted);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn normalization_refuses_before_mutation_when_value_admission_refuses() {
    let mut partition = peer_partition();
    let before = kind_projection::canonical_history(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        &partition,
    )
    .unwrap();
    let mut calls = Vec::new();
    assert_eq!(
        kind_admission::normalize_value(
            &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
            &mut partition,
            &mut |work, bytes| {
                calls.push((work, bytes));
                Err(Cause::MemoryExhausted)
            },
            &mut |_, _| panic!("refused scratch may not acquire result custody")
        ),
        Err(Cause::MemoryExhausted)
    );
    assert_eq!(calls, vec![(0, 65_536)]);
    assert_eq!(
        kind_projection::canonical_history(
            &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
            &partition
        )
        .unwrap(),
        before
    );
}

#[test]
fn digest_work_refusal_keeps_the_exact_value_checkpoint_cause() {
    let mut partition = peer_partition();
    partition.adjacency = SparseAdjacencyTable::default();
    partition.reverse_adjacency = SparseAdjacencyTable::default();
    let mut calls = Vec::new();
    let mut work = 0_u64;
    assert_eq!(
        kind_admission::normalize_value(
            &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
            &mut partition,
            &mut |units, bytes| {
                calls.push((units, bytes));
                if units > work {
                    return Err(Cause::SourceUnavailable);
                }
                work -= units;
                Ok(())
            },
            &mut |_, _| panic!("empty partition has no retained entries")
        ),
        Err(Cause::SourceUnavailable)
    );
    // No graph traversal precedes the one-byte empty MessagePack digest write.
    assert_eq!(calls, vec![(0, 65_536), (1, 0)]);
}
