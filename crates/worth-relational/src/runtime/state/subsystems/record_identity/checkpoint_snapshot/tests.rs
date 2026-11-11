//! The actual allocator's unpublished burns and pending custody survive capture.
use super::*;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

#[test]
fn funded_identity_snapshot_preserves_actual_pending_burns_wire_and_last_alias() {
    let owner = RecordIdentitySubsystem::default();
    let mut pending = owner.begin_allocations();
    let allocation = pending
        .reserve(RecordAllocationClass::Entity, PartitionId(7))
        .unwrap();
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let image = owner.checkpoint_identity_for_request(&request).unwrap();
    assert_eq!(image, owner.checkpoint_identity());
    assert_eq!(image.pending_reservations.len(), 1);
    assert_eq!(image.pending_reservations[0].slot, allocation.slot as u64);
    assert_eq!(
        image.generation_high_water[0].generation,
        allocation.generation
    );
    assert_eq!(
        image.append_frontiers[0].next_slot,
        allocation.slot as u64 + 1
    );
    #[derive(serde::Serialize)]
    struct Legacy<'a> {
        schema_version: u16,
        generation_high_water: &'a [DurableRecordGenerationHighWater],
        reusable_slots: &'a [DurableReusableRecordSlot],
        append_frontiers: &'a [DurableRecordSlotFrontier],
        pending_reservations: &'a [DurablePendingRecordReservation],
    }
    let legacy = Legacy {
        schema_version: 2,
        generation_high_water: &image.generation_high_water,
        reusable_slots: &image.reusable_slots,
        append_frontiers: &image.append_frontiers,
        pending_reservations: &image.pending_reservations,
    };
    assert_eq!(
        rmp_serde::to_vec(&image).unwrap(),
        rmp_serde::to_vec(&legacy).unwrap()
    );
    assert_eq!(
        rmp_serde::to_vec_named(&image).unwrap(),
        rmp_serde::to_vec_named(&legacy).unwrap()
    );
    let decoded: DurableRecordIdentityState =
        rmp_serde::from_slice(&rmp_serde::to_vec(&image).unwrap()).unwrap();
    assert_eq!(image, decoded);
    let charged = budget.charged_memory_bytes();
    assert!(charged > 0);
    drop(pending);
    let after_abandonment = owner.checkpoint_identity();
    assert!(after_abandonment.pending_reservations.is_empty());
    assert_eq!(after_abandonment.append_frontiers, image.append_frontiers);
    assert_eq!(
        after_abandonment.generation_high_water,
        image.generation_high_water
    );
    assert_eq!(image.pending_reservations.len(), 1);
    let alias = image.clone();
    drop(image);
    drop(owner);
    assert_eq!(budget.charged_memory_bytes(), charged);
    assert_eq!(alias.pending_reservations.len(), 1);
    drop(alias);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn identity_snapshot_refuses_actual_backing_and_original_cancellation_before_copy() {
    let owner = RecordIdentitySubsystem::default();
    let mut pending = owner.begin_allocations();
    pending
        .reserve(RecordAllocationClass::Entity, PartitionId(7))
        .unwrap();
    let calibration = SerialMemoryBudget::new(Some(64 * 1024));
    let host =
        SerialRequest::from_memory(calibration.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let mut controller = None;
    request
        .run_native_with_checkpoint(|_| controller = Some(calibration.charged_memory_bytes()))
        .unwrap();
    let minimum = size_of::<DurableRecordGenerationHighWater>()
        + size_of::<DurableRecordSlotFrontier>()
        + size_of::<DurablePendingRecordReservation>();
    let budget = SerialMemoryBudget::new(Some(controller.unwrap() + minimum as u64 - 1));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    assert!(matches!(owner.checkpoint_identity_for_request(&request),
        Err(RelationalRecoveryStop::Request(crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
            requested, admitted, ancestor: 0,
        })) if requested >= minimum as u64 && admitted < minimum as u64));
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(owner.checkpoint_identity().pending_reservations.len(), 1);
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    cancellation.cancel();
    assert!(matches!(
        owner.checkpoint_identity_for_request(&ExecutionRequest::serial(&host)),
        Err(RelationalRecoveryStop::Request(
            crate::execution::RelationalExecutionDenialCause::Cancelled
        ))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(owner.checkpoint_identity().pending_reservations.len(), 1);
}
