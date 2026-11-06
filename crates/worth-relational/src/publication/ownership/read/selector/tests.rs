//! Lifecycle corruption is distinct from legitimate absent materialization.
use super::*;
use crate::storage::{
    overlay::PartitionState,
    substrate::{
        EntityArena, EntityExtra, RelationArena, RelationEndpoints, RelationExtra, SlotInit,
    },
};
use std::collections::BTreeMap;
fn image() -> (
    BTreeMap<crate::identity::data::PartitionId, PartitionState>,
    EntityId,
    RelationId,
) {
    let partition = crate::identity::data::PartitionId(1);
    let mut state = PartitionState {
        partition_id: partition,
        adjacency_policy: crate::config::data::AdjacencyPolicy {
            backend: crate::config::data::AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        },
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(1),
        relation_arena: RelationArena::with_capacity(1),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    };
    let (slot, generation, _) = state.entity_arena.push_slot(SlotInit {
        partition_id: partition,
        kind_id: KindId(1),
        version_id: crate::identity::data::VersionId(1),
        extra: EntityExtra::default(),
    });
    let entity = EntityId::new(partition, slot as u64, generation);
    let (slot, generation, _) = state.relation_arena.push_slot(SlotInit {
        partition_id: partition,
        kind_id: KindId(2),
        version_id: crate::identity::data::VersionId(1),
        extra: RelationExtra {
            endpoints: Some(RelationEndpoints {
                source: entity,
                target: entity,
            }),
            authoritative_aspect_state: None,
        },
    });
    (
        BTreeMap::from([(partition, state)]),
        entity,
        RelationId::new(partition, slot as u64, generation),
    )
}
fn damage(error: TransactionCommitError) {
    use crate::{
        execution::RelationalExecutionDenialCause,
        storage::data::RelationalStorageUnavailableCause,
        transactions::data::CommitExecutionDenialKind,
    };
    assert!(
        matches!(error,TransactionCommitError::Execution{denial,..}
        if matches!(denial.kind,CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::StorageUnavailable(f)) if f.cause==RelationalStorageUnavailableCause::Damaged)),
        "{error:?}"
    );
}
#[test]
fn live_metadata_loss_cannot_be_healed_from_the_candidate_after_image() {
    let (healthy, entity, relation_id) = image();
    let mut damaged = healthy.clone();
    let state = damaged.get_mut(&entity.partition_id).unwrap();
    state
        .entity_arena
        .metadata_history
        .set(0, Default::default());
    state
        .relation_arena
        .metadata_history
        .set(0, Default::default());
    assert!(state.entity_arena.get(&entity).unwrap().is_live());
    assert!(state.relation_arena.get(&relation_id).unwrap().is_live());
    let before = InvariantStateView::new(&damaged, crate::identity::data::VersionId(1));
    let after = InvariantStateView::new(&healthy, crate::identity::data::VersionId(1));
    damage(entity_kind(&before, Some(&after), entity).unwrap_err());
    damage(relation(&before, Some(&after), relation_id).unwrap_err());
    damage(
        entity_kind(
            &before,
            Some(&after),
            EntityId::new(entity.partition_id, 8, 1),
        )
        .unwrap_err(),
    );
    damage(
        relation(
            &before,
            Some(&after),
            RelationId::new(entity.partition_id, 8, 1),
        )
        .unwrap_err(),
    );
}
#[test]
fn suspended_selectors_use_only_restored_candidate_metadata() {
    let (healthy, entity, relation_id) = image();
    let mut suspended = healthy.clone();
    let state = suspended.get_mut(&entity.partition_id).unwrap();
    state
        .entity_arena
        .suspend_materialization(entity.slot_index())
        .unwrap();
    state
        .relation_arena
        .suspend_materialization(relation_id.slot_index())
        .unwrap();
    let before = InvariantStateView::new(&suspended, crate::identity::data::VersionId(1));
    let after = InvariantStateView::new(&healthy, crate::identity::data::VersionId(1));
    assert!(before.entity_metadata(entity).is_none());
    assert!(before.relation_metadata(relation_id).is_none());
    assert_eq!(
        entity_kind(&before, Some(&after), entity).unwrap(),
        Some(KindId(1))
    );
    let edge = relation(&before, Some(&after), relation_id).unwrap();
    assert_eq!((edge.source, edge.target), (entity, entity));
    for error in [
        entity_kind(&before, None, entity).unwrap_err(),
        relation(&before, None, relation_id).unwrap_err(),
    ] {
        assert!(
            matches!(&error,TransactionCommitError::Conflict{error:conflict,..}
            if matches!(&conflict.class,crate::transactions::data::ConflictClass::MutationStateInconsistency{evidence:crate::transactions::data::MutationStateInconsistencyEvidence::MaterializationTransition{..},..})),
            "{error:?}"
        );
    }
}
