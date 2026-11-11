//! Pure slot transitions: source versions are real arena records, not runtime roots.
use super::*;
use crate::{identity::data::EntityId, storage::substrate::EntityExtra};
fn source() -> (RecordArena<EntityRecordKind>, EntityId) {
    let mut arena = RecordArena::with_capacity(1);
    let id = EntityId::new(PartitionId::main(), 7, 3);
    arena
        .write_reserved_slot(
            SlotInit {
                partition_id: id.partition_id,
                kind_id: KindId(8),
                version_id: VersionId(11),
                extra: EntityExtra::from_unbudgeted_parts(
                    Some(crate::identity::data::StructuralFingerprint::new(
                        crate::symbols::data::Symbol(1),
                        717,
                    )),
                    None,
                    None,
                ),
            },
            id.slot_index(),
            id.generation_value(),
        )
        .unwrap();
    (arena, id)
}
#[test]
fn historical_slot_selection_keeps_the_issued_identity_and_birth_with_fresh_provenance() {
    let (original, id) = source();
    for posture in [
        None,
        Some(RecordLifecycleState::Live),
        Some(RecordLifecycleState::DeletedRetained),
        Some(RecordLifecycleState::MaterializationUnavailable),
    ] {
        let mut current = if posture.is_some() {
            original.clone()
        } else {
            RecordArena::with_capacity(1)
        };
        if let Some(posture) = posture {
            if posture == RecordLifecycleState::DeletedRetained {
                current.retire(id.slot_index(), VersionId(12)).unwrap();
            }
            if posture == RecordLifecycleState::MaterializationUnavailable {
                current.suspend_materialization(id.slot_index()).unwrap();
            }
        }
        current
            .select_retained_version_using(None, &id, original.get(&id).unwrap(), VersionId(19))
            .unwrap();
        let row = current.get(&id).unwrap();
        assert!(row.is_live());
        assert!(current.field_revisions_at(id.slot_index()).is_some());
        assert_eq!(row.generation(), 3);
        assert_eq!(row.kind_id(), Some(KindId(8)));
        assert_eq!(
            row.extra().structural_fingerprint,
            Some(crate::identity::data::StructuralFingerprint::new(
                crate::symbols::data::Symbol(1),
                717
            ))
        );
        assert_eq!(
            current
                .metadata_history_at(id.slot_index())
                .unwrap()
                .get(current.metadata_history_at(id.slot_index()).unwrap().len() - 1)
                .unwrap()
                .effective_at,
            VersionId(19)
        );
        assert_eq!(
            current.created_at[current.physical_index(id.slot_index()).unwrap()],
            VersionId(11)
        );
        assert_eq!(current.live_bitset.set_slots().collect::<Vec<_>>(), vec![7]);
    }
}
#[test]
fn historical_slot_selection_refuses_a_reused_generation_or_a_nonlive_source() {
    let (original, id) = source();
    let mut current = original.clone();
    let physical = current.physical_index(id.slot_index()).unwrap();
    current.generations.set(physical, 4);
    assert!(current
        .select_retained_version_using(None, &id, original.get(&id).unwrap(), VersionId(19))
        .is_err());
    assert_eq!(current.generations[physical], 4);
    assert_eq!(
        current
            .metadata_history_at(id.slot_index())
            .unwrap()
            .get(0)
            .unwrap()
            .effective_at,
        VersionId(11),
    );
    let mut current = original.clone();
    let mut retired = original.clone();
    retired.retire(id.slot_index(), VersionId(12)).unwrap();
    assert!(current
        .select_retained_version_using(None, &id, retired.get(&id).unwrap(), VersionId(19))
        .is_err());
    assert_eq!(
        current
            .metadata_history_at(id.slot_index())
            .unwrap()
            .get(0)
            .unwrap()
            .effective_at,
        VersionId(11)
    );
}

#[test]
fn selecting_another_record_with_equal_generation_and_kind_cannot_copy_its_payload() {
    let (mut original, id) = source();
    let foreign = EntityId::new(id.partition_id, 8, id.generation_value());
    original
        .write_reserved_slot(
            SlotInit {
                partition_id: id.partition_id,
                kind_id: KindId(8),
                version_id: VersionId(11),
                extra: EntityExtra::default(),
            },
            foreign.slot_index(),
            foreign.generation_value(),
        )
        .unwrap();
    let mut current = original.clone();
    assert!(current
        .select_retained_version_using(None, &id, original.get(&foreign).unwrap(), VersionId(19))
        .is_err());
    assert_eq!(
        current.get(&id).unwrap().extra().structural_fingerprint,
        original.get(&id).unwrap().extra().structural_fingerprint
    );
    assert_eq!(
        current.metadata_history_at(id.slot_index()).unwrap().len(),
        1
    );
}

#[test]
fn selecting_a_version_refuses_a_conflicting_record_birth() {
    let (original, id) = source();
    let mut current = original.clone();
    let physical = current.physical_index(id.slot_index()).unwrap();
    current.created_at.set(physical, VersionId(12));
    assert!(current
        .select_retained_version_using(None, &id, original.get(&id).unwrap(), VersionId(19))
        .is_err());
    assert_eq!(
        current.metadata_history_at(id.slot_index()).unwrap().len(),
        1
    );
    assert_eq!(current.created_at[physical], VersionId(12));
}
