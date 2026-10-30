use super::*;
use worth_execution::ExecutionAllocationPolicy::SystemAllocation;

#[test]
fn reports_actual_burns_in_both_classes_and_seals_owned_prefixes() {
    let owner = RecordIdentitySubsystem::default();
    let observer = owner.observe_reservations(8, SystemAllocation).unwrap();
    let first = observer.report(SystemAllocation).unwrap();
    assert_eq!(first.records().len(), 0);
    for class in [
        RecordAllocationClass::Entity,
        RecordAllocationClass::Relation,
    ] {
        drop(owner.reserve(class, PartitionId(7)).unwrap());
    }
    let burned = observer.report(SystemAllocation).unwrap();
    assert_eq!(
        burned.records().collect::<Vec<_>>(),
        [
            RecordRef::Entity(EntityId::new(PartitionId(7), 0, 1)),
            RecordRef::Relation(RelationId::new(PartitionId(7), 0, 1)),
        ]
    );
    assert_eq!(
        first.records().len(),
        0,
        "a sealed report cannot acquire later records"
    );
    let (mut success, _, _) = owner
        .reserve(RecordAllocationClass::Entity, PartitionId(7))
        .unwrap();
    success.consume();
    assert_eq!(
        observer.report(SystemAllocation).unwrap().records().last(),
        Some(RecordRef::Entity(EntityId::new(PartitionId(7), 1, 1)))
    );
}
#[test]
fn scopes_are_runtime_bound_nested_unwind_safe_and_fail_closed_on_capacity() {
    let owner = RecordIdentitySubsystem::default();
    let other = RecordIdentitySubsystem::default();
    let outer = owner.observe_reservations(8, SystemAllocation).unwrap();
    let limited = owner.observe_reservations(0, SystemAllocation).unwrap();
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _inner = owner.observe_reservations(1, SystemAllocation).unwrap();
        drop(
            other
                .reserve(RecordAllocationClass::Entity, PartitionId(7))
                .unwrap(),
        );
        drop(
            owner
                .reserve(RecordAllocationClass::Relation, PartitionId(7))
                .unwrap(),
        );
        panic!("unwind observer");
    }));
    assert_eq!(outer.report(SystemAllocation).unwrap().records().len(), 1);
    assert!(matches!(
        limited.report(SystemAllocation),
        Err(RecordReservationObservationDenial::CapacityExceeded { maximum_records: 0 })
    ));
    drop(limited);
    drop(
        owner
            .reserve(RecordAllocationClass::Entity, PartitionId(7))
            .unwrap(),
    );
    assert_eq!(outer.report(SystemAllocation).unwrap().records().len(), 2);
    drop(outer);
    OBSERVERS.with_borrow(|observers| assert!(observers.is_empty()));
}

fn funded_lease(
    cancellation: worth_execution::CancellationToken,
) -> worth_execution::ExecutionResourceLease<'static> {
    use worth_foundational::facade::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    worth_execution::ExecutionResourceLease::serial(worth_execution::LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(1_048_576), None),
        ),
        cancellation,
        deadline: None,
    })
    .unwrap()
}
#[test]
fn recording_interruption_retains_its_original_cause_instead_of_certifying_a_prefix() {
    let owner = RecordIdentitySubsystem::default();
    let cancellation = worth_execution::CancellationSource::new();
    let lease = funded_lease(cancellation.token());
    let observer = owner
        .observe_reservations(2, ExecutionAllocationPolicy::Execution(&lease))
        .unwrap();
    cancellation.cancel();
    drop(
        owner
            .reserve(RecordAllocationClass::Entity, PartitionId(7))
            .unwrap(),
    );
    assert!(matches!(observer.report(SystemAllocation),
        Err(RecordReservationObservationDenial::Allocation(ref cause))
            if cause.classify() == worth_execution::ExecutionAllocationDenialClass::Request(worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled))));
}
#[test]
fn bounded_observation_and_sealed_reports_hold_exactly_their_payload_charge() {
    for count in [1, 8, 64, 512] {
        let lease = funded_lease(worth_execution::CancellationToken::new());
        let allocation = ExecutionAllocationPolicy::Execution(&lease);
        let owner = RecordIdentitySubsystem::default();
        let before = lease.charged_memory_bytes();
        let observer = owner.observe_reservations(count, allocation).unwrap();
        let payload = u64::try_from(count * RECORD_BYTES).unwrap();
        assert_eq!(lease.charged_memory_bytes() - before, payload);
        for _ in 0..count {
            drop(
                owner
                    .reserve(RecordAllocationClass::Entity, PartitionId(7))
                    .unwrap(),
            );
        }
        assert_eq!(
            lease.charged_memory_bytes() - before,
            payload,
            "recording does not grow backing"
        );
        let report = observer.report(allocation).unwrap();
        assert_eq!(report.records().len(), count);
        assert_eq!(lease.charged_memory_bytes() - before, 2 * payload);
        drop(report);
        drop(observer);
        assert_eq!(
            lease.charged_memory_bytes(),
            before,
            "all diagnostic payload custody releases"
        );
    }
}

#[test]
fn reports_canonical_reservations_as_well_as_live_choices() {
    let owner = RecordIdentitySubsystem::default();
    let observation = owner.observe_reservations(1, SystemAllocation).unwrap();
    let mut reservation = owner
        .reserve_exact(
            0,
            RecordAllocationClass::Entity,
            PartitionId(7),
            0,
            1,
            RecordAllocationOrigin::AppendFrontier,
        )
        .unwrap();
    assert_eq!(
        observation
            .report(SystemAllocation)
            .unwrap()
            .records()
            .collect::<Vec<_>>(),
        [RecordRef::Entity(EntityId::new(PartitionId(7), 0, 1))]
    );
    reservation.consume();
    assert_eq!(
        observation
            .report(SystemAllocation)
            .unwrap()
            .records()
            .len(),
        1,
        "consuming custody is not another reservation"
    );
}
