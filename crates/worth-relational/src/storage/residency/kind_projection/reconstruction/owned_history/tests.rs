//! Pure projection and suffix mutation; no reconstructed authority or provider.
use super::*;
use crate::identity::data::{EntityId, VersionId};
use crate::storage::data::RecordLifecycleState;
use crate::storage::substrate::RecordHistoryOpening;
mod fixture;

#[test]
fn head_prefixes_rebuild_owned_history_after_endpoint_edits_and_generation_reuse() {
    let (mut partial, mut oracle, relation, opening) = fixture::partitions();
    assert_owned_history(&mut partial, &mut oracle, &opening);
    let target = EntityId::new(relation.partition_id, 0, 1);
    for partition in [&mut partial, &mut oracle] {
        fixture::update_target(partition, relation, target, VersionId(1_000));
    }
    assert_owned_history(&mut partial, &mut oracle, &opening);
    for partition in [&mut partial, &mut oracle] {
        fixture::reuse(partition, relation);
    }
    assert_owned_history(&mut partial, &mut oracle, &opening);
}

fn assert_owned_history(
    partial: &mut PartitionState,
    full: &mut PartitionState,
    opening: &RecordHistoryOpening,
) {
    let mut actual = BTreeMap::new();
    for slot in partial.relation_arena.occupied_slots_iter() {
        project_opening(partial, slot, opening, &mut actual, &mut |_, _| Ok(())).unwrap();
        project_suffix(partial, slot, &mut actual, &mut |_, _| Ok(())).unwrap();
    }
    let lane = crate::storage::substrate::maintenance::ReconstructionLane::oracle();
    let expected = super::super::canonical_history(&lane, full).unwrap();
    assert_eq!(actual.values().cloned().collect::<Vec<_>>(), expected);
    super::super::installation::install(
        &lane,
        partial,
        &actual,
        AdjacencyKindBasis::Historical,
        None,
        &mut |_, _| Ok(()),
        &mut |_, _| Ok(None),
    )
    .unwrap();
    crate::storage::residency::rebuild_region_history_kind_buckets(&lane, full).unwrap();
    assert_eq!(
        crate::storage::residency::RelationalRegionKindCommitment::capture(&lane, partial).unwrap(),
        crate::storage::residency::RelationalRegionKindCommitment::capture(&lane, full).unwrap()
    );
}
