//! Pure cursor logic. Original-source authenticity belongs to the real producer.

use super::*;
use crate::identity::data::{EntityId, RelationId};

#[test]
fn borrowed_original_allocations_preserve_exact_ids_origins_and_consumption() {
    let entity = RecordRef::Entity(EntityId::new(PartitionId(7), 4, 9));
    let relation = RecordRef::Relation(RelationId::new(PartitionId(11), 1000, 1));
    let evidence = [
        CanonicalRecordAllocation::with_origin(
            0,
            entity.clone(),
            RecordAllocationOrigin::Reclaimed {
                prior_generation: 8,
            },
        ),
        CanonicalRecordAllocation::with_origin(
            1,
            relation.clone(),
            RecordAllocationOrigin::AppendFrontier,
        ),
    ];
    let mut cursor = OriginalRecordAllocationCursor::new(&evidence);
    assert!(std::ptr::eq(cursor.evidence, evidence.as_slice()));
    // A reclaimed original may skip unpublished generation burns; there is no
    // target-arena/high-water inference. Append may begin after burned slots.
    let first = cursor
        .reserve(RecordAllocationClass::Entity, PartitionId(7))
        .unwrap();
    assert_eq!((first.slot, first.generation), (4, 9));
    cursor.record(entity.clone()).unwrap();
    let second = cursor
        .reserve(RecordAllocationClass::Relation, PartitionId(11))
        .unwrap();
    assert_eq!((second.slot, second.generation), (1000, 1));
    cursor.record(relation.clone()).unwrap();
    cursor.finish_mutation().unwrap();
    assert_eq!(evidence[0].record(), &entity);
    assert_eq!(evidence[1].record(), &relation);
    assert_eq!(cursor.reserved, 2);
    assert_eq!(cursor.recorded, 2);
    assert_eq!(
        std::mem::size_of::<OriginalRecordAllocationCursor<'_>>(),
        std::mem::size_of::<&[CanonicalRecordAllocation]>() + 2 * std::mem::size_of::<usize>()
    );
}

#[test]
fn original_allocations_refuse_wrong_targets_records_ordinals_and_origin_generations() {
    let record = RecordRef::Entity(EntityId::new(PartitionId(7), 4, 9));
    let evidence = [CanonicalRecordAllocation::with_origin(
        0,
        record.clone(),
        RecordAllocationOrigin::Reclaimed {
            prior_generation: 8,
        },
    )];
    let mut cursor = OriginalRecordAllocationCursor::new(&evidence);
    assert!(matches!(
        cursor.reserve(RecordAllocationClass::Relation, PartitionId(7)),
        Err(RecordAllocationDenial::ReplayTargetMismatch { ordinal: 0, .. })
    ));
    assert!(matches!(
        cursor.reserve(RecordAllocationClass::Entity, PartitionId(8)),
        Err(RecordAllocationDenial::ReplayTargetMismatch { ordinal: 0, .. })
    ));
    assert_eq!((cursor.reserved, cursor.recorded), (0, 0));
    cursor
        .reserve(RecordAllocationClass::Entity, PartitionId(7))
        .unwrap();
    let wrong = RecordRef::Entity(EntityId::new(PartitionId(7), 4, 8));
    assert_eq!(
        cursor.record(wrong.clone()),
        Err(RecordAllocationDenial::ReplayRecordMismatch {
            ordinal: 0,
            expected: record.clone(),
            observed: wrong,
        })
    );
    assert_eq!((cursor.reserved, cursor.recorded), (1, 0));
    cursor.record(record.clone()).unwrap();
    cursor.finish_mutation().unwrap();

    let wrong_ordinal = [CanonicalRecordAllocation::with_origin(
        3,
        record.clone(),
        RecordAllocationOrigin::Reclaimed {
            prior_generation: 8,
        },
    )];
    let mut cursor = OriginalRecordAllocationCursor::new(&wrong_ordinal);
    assert_eq!(
        cursor
            .reserve(RecordAllocationClass::Entity, PartitionId(7))
            .unwrap_err(),
        RecordAllocationDenial::ReplayEvidenceUnexpected {
            expected_ordinal: 0,
            observed_ordinal: 3
        }
    );
    assert_eq!((cursor.reserved, cursor.recorded), (0, 0));
    for (origin, generation, expected) in [
        (RecordAllocationOrigin::AppendFrontier, 0, 1),
        (RecordAllocationOrigin::AppendFrontier, 9, 1),
        (
            RecordAllocationOrigin::Reclaimed {
                prior_generation: 8,
            },
            8,
            9,
        ),
    ] {
        let bad = [CanonicalRecordAllocation::with_origin(
            0,
            RecordRef::Entity(EntityId::new(PartitionId(7), 4, generation)),
            origin,
        )];
        let mut cursor = OriginalRecordAllocationCursor::new(&bad);
        assert_eq!(
            cursor
                .reserve(RecordAllocationClass::Entity, PartitionId(7))
                .unwrap_err(),
            RecordAllocationDenial::ReplayGenerationMismatch {
                ordinal: 0,
                class: RecordAllocationClass::Entity,
                partition_id: PartitionId(7),
                slot: 4,
                expected_generation: expected,
                observed_generation: generation,
            }
        );
        assert_eq!((cursor.reserved, cursor.recorded), (0, 0));
    }
}

#[test]
fn original_allocations_require_every_reserved_record_and_abandon_without_cleanup() {
    let record = RecordRef::Entity(EntityId::new(PartitionId(7), 100, 1));
    let evidence = [CanonicalRecordAllocation::with_origin(
        0,
        record.clone(),
        RecordAllocationOrigin::AppendFrontier,
    )];
    {
        let mut cursor = OriginalRecordAllocationCursor::new(&evidence);
        assert_eq!(
            cursor.record(record.clone()),
            Err(RecordAllocationDenial::ReplayEvidenceMissing { ordinal: 0 })
        );
        cursor
            .reserve(RecordAllocationClass::Entity, PartitionId(7))
            .unwrap();
        assert_eq!(
            cursor.finish_mutation(),
            Err(RecordAllocationDenial::ReplayEvidenceRemaining { remaining: 1 })
        );
        // Abandonment has no live reservation, lock, reusable slot or allocator
        // authority to touch. The source body still owns the original evidence.
    }
    let mut retry = OriginalRecordAllocationCursor::new(&evidence);
    retry
        .reserve(RecordAllocationClass::Entity, PartitionId(7))
        .unwrap();
    retry.record(record.clone()).unwrap();
    retry.finish_mutation().unwrap();
    assert_eq!(
        retry
            .reserve(RecordAllocationClass::Entity, PartitionId(7))
            .unwrap_err(),
        RecordAllocationDenial::ReplayEvidenceMissing { ordinal: 1 }
    );
    assert_eq!(evidence[0].record(), &record);
    let mut empty = OriginalRecordAllocationCursor::new(&[]);
    empty.finish_mutation().unwrap();
    assert_eq!(
        empty
            .reserve(RecordAllocationClass::Entity, PartitionId(7))
            .unwrap_err(),
        RecordAllocationDenial::ReplayEvidenceMissing { ordinal: 0 }
    );
}
