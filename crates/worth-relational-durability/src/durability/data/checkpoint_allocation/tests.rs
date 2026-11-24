use super::*;
use CheckpointReservationAdvanceDenial as D;

fn record(class: RecordAllocationClass, slot: u64, generation: u32) -> CheckpointRecordIdentity {
    CheckpointRecordIdentity {
        class,
        partition_id: 4,
        slot,
        generation,
    }
}
fn frontier(class: RecordAllocationClass, next_slot: u64) -> CheckpointRecordFrontier {
    CheckpointRecordFrontier {
        class,
        partition_id: 4,
        next_slot,
    }
}

#[test]
fn exact_report_permits_only_contiguous_appends_and_single_generation_steps() {
    for class in [
        RecordAllocationClass::Entity,
        RecordAllocationClass::Relation,
    ] {
        let old_frontiers = [frontier(class, 1)];
        let old_generations = [record(class, 0, 1)];
        let before = CheckpointAllocatorInspection {
            frontiers: &old_frontiers,
            generations: &old_generations,
        };
        let new_frontiers = [frontier(class, 2)];
        let new_generations = [record(class, 0, 2), record(class, 1, 1)];
        let after = CheckpointAllocatorInspection {
            frontiers: &new_frontiers,
            generations: &new_generations,
        };
        assert_eq!(
            before.verify_reported_advance(&after, new_generations),
            Ok(())
        );
        assert_eq!(
            before.verify_reported_advance(&after, []),
            Err(D::FrontiersDiffer)
        );
        let gap = record(class, 3, 1);
        assert_eq!(
            before.verify_reported_advance(&after, [gap]),
            Err(D::UnreservedAppendGap {
                reservation: gap,
                frontier: 1
            })
        );
        for generation in [1, 3] {
            let bad = record(class, 0, generation);
            assert_eq!(
                before.verify_reported_advance(&after, [bad]),
                Err(D::GenerationStep {
                    reservation: bad,
                    expected: 2
                })
            );
        }
    }
}

#[test]
fn regressions_disappearing_coordinates_and_duplicates_are_not_advances() {
    let class = RecordAllocationClass::Entity;
    let frontiers = [frontier(class, 2)];
    let generations = [record(class, 0, 1), record(class, 1, 1)];
    let before = CheckpointAllocatorInspection {
        frontiers: &frontiers,
        generations: &generations,
    };
    assert_eq!(before.verify_reported_advance(&before, []), Ok(()));
    for rows in [
        vec![],
        vec![frontier(class, 1)],
        vec![frontier(class, 3)],
        vec![frontier(class, 2), frontier(class, 2)],
    ] {
        let after = CheckpointAllocatorInspection {
            frontiers: &rows,
            ..before
        };
        assert!(
            before.verify_reported_advance(&after, []).is_err(),
            "{rows:?}"
        );
        assert!(
            after.verify_reported_advance(&before, []).is_err(),
            "{rows:?}"
        );
    }
    for rows in [
        vec![],
        vec![record(class, 0, 0), record(class, 1, 1)],
        vec![record(class, 0, 2), record(class, 1, 1)],
        vec![record(class, 0, 1), record(class, 0, 1)],
    ] {
        let after = CheckpointAllocatorInspection {
            generations: &rows,
            ..before
        };
        assert!(
            before.verify_reported_advance(&after, []).is_err(),
            "{rows:?}"
        );
        assert!(
            after.verify_reported_advance(&before, []).is_err(),
            "{rows:?}"
        );
    }
}

#[test]
fn overflow_and_duplicate_reports_refuse_without_a_partial_success() {
    let class = RecordAllocationClass::Relation;
    let empty = CheckpointAllocatorInspection {
        frontiers: &[],
        generations: &[],
    };
    assert_eq!(
        empty.verify_reported_advance(&empty, [record(class, u64::MAX, 1)]),
        Err(D::SlotOverflow)
    );
    let frontiers = [frontier(class, 1)];
    let generations = [record(class, 0, u32::MAX)];
    let before = CheckpointAllocatorInspection {
        frontiers: &frontiers,
        generations: &generations,
    };
    assert_eq!(
        before.verify_reported_advance(&before, [record(class, 0, 1)]),
        Err(D::GenerationOverflow)
    );
    let duplicate = record(class, 0, 1);
    assert_eq!(
        empty.verify_reported_advance(&empty, [duplicate, duplicate]),
        Err(D::GenerationStep {
            reservation: duplicate,
            expected: 2
        })
    );
}
