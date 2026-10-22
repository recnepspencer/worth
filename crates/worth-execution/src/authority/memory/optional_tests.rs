use super::*;

#[test]
fn absent_serial_ceiling_keeps_shared_charges_and_last_holder_release() {
    let budget = SerialMemoryBudget::new(None);
    let observer = budget.clone();
    let mut held = budget.reserve(23).unwrap();
    let sibling = observer.reserve(7).unwrap();
    assert_eq!(budget.limit(), None);
    assert_eq!(observer.charged_memory_bytes(), 30);
    drop(budget);
    held.resize(31).unwrap();
    assert_eq!(observer.charged_memory_bytes(), 38);
    drop(sibling);
    assert_eq!(observer.charged_memory_bytes(), 31);
    drop(held);
    assert_eq!(observer.charged_memory_bytes(), 0);
}

#[test]
fn absent_child_ceiling_still_charges_finite_serial_parent_atomically() {
    let parent = SerialMemoryBudget::new(Some(19));
    let child = SerialMemoryBudget::new(None).within_parent(&parent);
    let mut held = child.reserve(13).unwrap();
    let sibling = parent.reserve(5).unwrap();
    assert_eq!(
        held.resize(15),
        Err(MemoryLimitDenial::Capacity {
            requested: 15,
            admitted: 14,
            level: MemoryLimitLevel::Policy { ancestor: 1 },
        })
    );
    assert_eq!(held.bytes(), 13);
    assert_eq!(parent.charged_memory_bytes(), 18);
    assert_eq!(child.charged_memory_bytes(), 13);
    drop(sibling);
    held.resize(19).unwrap();
    drop(held);
    assert_eq!(parent.charged_memory_bytes(), 0);
    assert_eq!(child.charged_memory_bytes(), 0);
}

#[test]
fn absent_serial_ceiling_refuses_counter_overflow_without_losing_the_hold() {
    // Ledger reservations do not allocate payload backing. This is a pure representability court.
    let budget = SerialMemoryBudget::new(None);
    let held = budget.reserve(u64::MAX).unwrap();
    assert_eq!(
        budget.reserve(1).unwrap_err(),
        MemoryLimitDenial::CounterOverflow {
            requested: 1,
            charged: u64::MAX,
            level: MemoryLimitLevel::Policy { ancestor: 0 },
        }
    );
    assert_eq!(budget.charged_memory_bytes(), u64::MAX);
    assert_eq!(held.bytes(), u64::MAX);
    drop(held);
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(budget.reserve(1).unwrap().bytes(), 1);
}

#[test]
fn absent_request_ceiling_survives_original_serial_policy_and_cancel() {
    use worth_foundational::{DeterminismContract, ExecutionBudget, ExecutionPosture};
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, None),
    );
    let cancellation = crate::CancellationSource::new();
    let serial = crate::SerialRequest::from_policy(&policy, cancellation.token(), None);
    let request = crate::ExecutionRequest::serial(&serial);
    assert_eq!(request.memory_limit(), None);
    let held = request
        .run(crate::ExecutionWorkCeiling::new(None), |lease| {
            ExecutionMemoryReservation::reserve_in_scope(lease, 29).unwrap()
        })
        .unwrap()
        .0;
    assert_eq!(
        serial
            .memory()
            .expect("explicit accounted request")
            .charged_memory_bytes(),
        29
    );
    cancellation.cancel();
    assert_eq!(
        request.consult(),
        Err(crate::WorkCeilingDenial::Stopped(
            crate::MapKernelStop::Cancelled
        ))
    );
    drop(held);
    assert_eq!(
        serial
            .memory()
            .expect("explicit accounted request")
            .charged_memory_bytes(),
        0
    );
}
