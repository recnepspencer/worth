//! Native slot lifecycle distinguishes this candidate's retirement from prior absence.
use worth_relational_storage::internal_port::storage::substrate::record_arena::arena::slot_lifecycle::RecordArenaFixturePort as _;
use worth_relational_storage::internal_port::storage::substrate::record_arena::arena::slot_lifecycle::metadata::RecordArenaFixturePort as _;
use super::*;
use crate::contracts::config::data::policies::{AdjacencyBackend, AdjacencyPolicy};
use crate::contracts::identity::data::primitives::{PartitionId, VersionId};
use worth_relational_storage::internal_port::storage::overlay::partition::PartitionState;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;
use worth_relational_storage::internal_port::storage::substrate::record_arena::arena::{EntityArena, RelationArena, SlotInit};
use worth_relational_storage::internal_port::storage::substrate::record_arena::kinds::{EntityRecordKind, RecordKind};
use std::collections::BTreeMap;
#[test]
fn native_endpoint_classification_distinguishes_current_retirement_from_prior_absence() {
    let policy = AdjacencyPolicy {
        backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
        small_degree_inline_capacity: 4,
    };
    let partition = PartitionId::main();
    let mut before = PartitionState {
        partition_id: partition,
        adjacency_policy: policy.clone(),
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(3),
        relation_arena: RelationArena::with_capacity(0),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    };
    let mut entities = Vec::new();
    for _ in 0..3 {
        let (slot, generation, _) = before.entity_arena.push_slot_for_test(SlotInit {
            partition_id: partition,
            kind_id: KindId(1),
            version_id: VersionId(1),
            extra: EntityRecordKind::empty_extra(),
        });
        entities.push(EntityId::new(partition, slot as u64, generation));
    }
    before
        .entity_arena
        .retire_record_for_test(entities[1].slot_index(), VersionId(2))
        .unwrap();
    let mut after = before.clone();
    after
        .entity_arena
        .retire_record_for_test(entities[0].slot_index(), VersionId(3))
        .unwrap();
    let before =
        WorkingState::working_state_for_test(BTreeMap::from([(partition, before)]), policy.clone());
    let after = WorkingState::working_state_for_test(BTreeMap::from([(partition, after)]), policy);
    let before = InvariantStateView::new(&before, VersionId(2));
    let after = InvariantStateView::new(&after, VersionId(3));
    use super::retirement::{classify, EndpointRetirement};
    assert!(matches!(
        classify(&before, &after, entities[0]),
        EndpointRetirement::RetiredByCandidate
    ));
    assert!(
        matches!(
            classify(&before, &after, entities[1]),
            EndpointRetirement::Unavailable
        ),
        "a prior retirement remains unavailable"
    );
    assert!(
        matches!(
            classify(&before, &after, entities[2]),
            EndpointRetirement::Current
        ),
        "a live endpoint remains readable"
    );
}
