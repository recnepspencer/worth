//! The actual fixed slot survives exhausted allowance and owns its refund.
use super::*;
use std::alloc::Layout;
use worth_execution::{
    CancellationSource, ExecutionMemoryReservation, ExecutionRequest, LeaseDenial, MapKernelStop,
    MemoryLimitDenial, MemoryLimitLevel, SerialMemoryBudget, SerialRequest,
};

fn independent_bill() -> u64 {
    Layout::new::<Option<CheckpointResult>>()
        .extend(Layout::new::<
            Option<worth_store_relational::PreparedRelationalSectionClose>,
        >())
        .unwrap()
        .0
        .extend(Layout::new::<Option<ExecutionMemoryReservation>>())
        .unwrap()
        .0
        .pad_to_align()
        .size() as u64
}
fn native_preparation_bill() -> u64 {
    let budget = SerialMemoryBudget::new(Some(64 << 10));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let original = worth_store_relational::PreparedRelationalSectionClose::prepare(
        &ExecutionRequest::serial(&host),
    )
    .unwrap();
    let bill = budget.charged_memory_bytes();
    assert!(
        bill > 0,
        "the actual Native owner funds its fixed frame and reversible outer storage"
    );
    drop(original);
    assert_eq!(budget.charged_memory_bytes(), 0);
    bill
}
fn controller() -> u64 {
    let budget = SerialMemoryBudget::new(Some(64 << 10));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let mut observed = None;
    ExecutionRequest::serial(&host)
        .run_native_with_checkpoint(|_| {
            observed = Some(budget.charged_memory_bytes());
        })
        .unwrap();
    assert_eq!(budget.charged_memory_bytes(), 0);
    observed.unwrap()
}

#[test]
fn prepaid_close_storage_retains_original_backings_through_exhaustion_and_public_moveout() {
    let ceiling = 64 << 10;
    let budget = SerialMemoryBudget::new(Some(ceiling));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let execution = ExecutionRequest::serial(&host);
    let mut slot = HomeCloseCheckpointResult::prepare(&execution).unwrap();
    let bill = independent_bill() + native_preparation_bill();
    assert!(slot.is_none());
    assert_eq!(budget.charged_memory_bytes(), bill);
    let original = slot.backing.as_deref().unwrap() as *const Backing;
    execution
        .run_native_with_checkpoint(|_| {
            let remaining = ceiling - budget.charged_memory_bytes();
            // This is the actual original Serial scope, not a replacement policy.
            let exhausted = ExecutionMemoryReservation::reserve_in_scope(None, remaining).unwrap();
            assert_eq!(budget.charged_memory_bytes(), ceiling);
            slot.store(Err(
                crate::ApplicationHomeCheckpointDenial::WorldSectionRequired,
            ));
            assert!(matches!(
                slot.result(),
                Err(crate::ApplicationHomeCheckpointDenial::WorldSectionRequired)
            ));
            slot.store(Err(
                crate::ApplicationHomeCheckpointDenial::HomePreparationIncomplete,
            ));
            assert!(matches!(
                slot.result(),
                Err(crate::ApplicationHomeCheckpointDenial::HomePreparationIncomplete)
            ));
            assert!(std::ptr::eq(original, slot.backing.as_deref().unwrap()));
            assert_eq!(budget.charged_memory_bytes(), ceiling);
            drop(exhausted);
        })
        .unwrap();
    assert_eq!(budget.charged_memory_bytes(), bill);
    let result = slot.into_result();
    assert!(matches!(
        result,
        Err(crate::ApplicationHomeCheckpointDenial::HomePreparationIncomplete)
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
    let mut dropped = HomeCloseCheckpointResult::prepare(&execution).unwrap();
    dropped.store(Err(
        crate::ApplicationHomeCheckpointDenial::WorldSectionRequired,
    ));
    assert_eq!(budget.charged_memory_bytes(), bill);
    drop(dropped);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn prepaid_slot_denial_targets_actual_backing_and_preserves_cancellation() {
    let bill = independent_bill();
    let budget = SerialMemoryBudget::new(Some(controller() + bill - 1));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&host);
    let failure = match HomeCloseCheckpointResult::prepare(&execution) {
        Err(cause) => cause,
        Ok(_) => panic!("one-short original budget admitted the actual slot"),
    };
    assert!(
        matches!(failure,
            WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(MemoryLimitDenial::Capacity {
                requested, admitted, level: MemoryLimitLevel::Policy { ancestor: 0 },
            })) if requested == bill && admitted == bill - 1
        ),
        "{failure:?}"
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
    cancellation.cancel();
    assert!(matches!(
        HomeCloseCheckpointResult::prepare(&execution),
        Err(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
}
