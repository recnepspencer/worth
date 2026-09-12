use worth_store::physical_runtime::stability::stable_physical_read_receipt_for_certification_test;
use worth_store_io_scheduler::{
    admit_background_pacing, blob_ingest_background_capacity_for_certification_test,
    foreground_reservation::admitted_point_read_reservation_for_certification_test,
    verification_deferred_background_capacity_for_certification_test,
    verification_denied_background_capacity_for_certification_test,
    verification_throttled_background_capacity_for_certification_test,
    verification_zero_admitted_throttle_background_capacity_for_certification_test,
    BackgroundIdleCapacityLeaseRequest, BackgroundPacingOutcome, BackgroundResourceBudget,
    QueueSlot,
};

use crate::{BlobStreamingReadAdmission, BlobStreamingReadDenial};

#[test]
fn verification_pressure_outcomes_drive_read_facade() {
    let yielded = BlobStreamingReadAdmission::from_stable_physical_read(
        stable_physical_read_receipt_for_certification_test(12),
        admitted_point_read_reservation_for_certification_test(),
        yield_verification_pressure(),
    )
    .expect_err("foreground pressure yield should deny blob read admission");
    assert!(matches!(
        yielded,
        BlobStreamingReadDenial::VerificationPressureYielded { counters }
            if counters.pressure_yield_denials() == 1
    ));

    let deferred = pressure_denial(deferred_verification_pressure());
    assert!(matches!(
        deferred,
        BlobStreamingReadDenial::VerificationPressureDeferred { counters }
            if counters.pressure_deferred_denials() == 1
    ));

    let denied = pressure_denial(denied_verification_pressure());
    assert!(matches!(
        denied,
        BlobStreamingReadDenial::VerificationPressureDenied { counters, .. }
            if counters.pressure_denied_denials() == 1
    ));

    let violation = pressure_denial(violation_verification_pressure());
    assert!(matches!(
        violation,
        BlobStreamingReadDenial::VerificationPressureViolation { counters }
            if counters.pressure_violations() == 1
    ));

    let class_mismatch = pressure_denial(blob_ingest_pressure());
    assert!(matches!(
        class_mismatch,
        BlobStreamingReadDenial::VerificationPressureClassMismatch { .. }
    ));

    let admission = BlobStreamingReadAdmission::from_stable_physical_read(
        stable_physical_read_receipt_for_certification_test(12),
        admitted_point_read_reservation_for_certification_test(),
        throttled_verification_pressure(),
    )
    .expect("throttled verification pressure with admitted budget should pace but admit read");
    assert_eq!(admission.pressure_counters().pressure_throttles(), 1);

    let zero_admitted = pressure_denial(zero_admitted_throttled_verification_pressure());
    assert!(matches!(
        zero_admitted,
        BlobStreamingReadDenial::VerificationPressureThrottledWithoutAdmittedCapacity { counters }
            if counters.pressure_throttles() == 1
    ));
}

fn pressure_denial(outcome: BackgroundPacingOutcome) -> BlobStreamingReadDenial {
    BlobStreamingReadAdmission::from_stable_physical_read(
        stable_physical_read_receipt_for_certification_test(12),
        admitted_point_read_reservation_for_certification_test(),
        outcome,
    )
    .expect_err("pressure outcome should deny read admission")
}

fn throttled_verification_pressure() -> BackgroundPacingOutcome {
    admit_background_pacing(BackgroundIdleCapacityLeaseRequest::new(
        verification_throttled_background_capacity_for_certification_test(
            read_pressure_budget(),
            one_slot_budget(),
        ),
    ))
}

fn zero_admitted_throttled_verification_pressure() -> BackgroundPacingOutcome {
    admit_background_pacing(BackgroundIdleCapacityLeaseRequest::new(
        verification_zero_admitted_throttle_background_capacity_for_certification_test(
            read_pressure_budget(),
        ),
    ))
}

fn yield_verification_pressure() -> BackgroundPacingOutcome {
    admit_background_pacing(
        BackgroundIdleCapacityLeaseRequest::new(
            verification_throttled_background_capacity_for_certification_test(
                read_pressure_budget(),
                read_pressure_budget(),
            ),
        )
        .with_foreground_pressure_events(1),
    )
}

fn deferred_verification_pressure() -> BackgroundPacingOutcome {
    admit_background_pacing(BackgroundIdleCapacityLeaseRequest::new(
        verification_deferred_background_capacity_for_certification_test(read_pressure_budget()),
    ))
}

fn denied_verification_pressure() -> BackgroundPacingOutcome {
    admit_background_pacing(BackgroundIdleCapacityLeaseRequest::new(
        verification_denied_background_capacity_for_certification_test(
            three_slot_budget(),
            one_slot_budget(),
            one_slot_budget(),
        ),
    ))
}

fn violation_verification_pressure() -> BackgroundPacingOutcome {
    admit_background_pacing(
        BackgroundIdleCapacityLeaseRequest::new(
            verification_throttled_background_capacity_for_certification_test(
                read_pressure_budget(),
                read_pressure_budget(),
            ),
        )
        .with_foreground_pressure_events(1)
        .with_late_yield(),
    )
}

fn blob_ingest_pressure() -> BackgroundPacingOutcome {
    admit_background_pacing(BackgroundIdleCapacityLeaseRequest::new(
        blob_ingest_background_capacity_for_certification_test(read_pressure_budget()),
    ))
}

fn read_pressure_budget() -> BackgroundResourceBudget {
    BackgroundResourceBudget::new().with_queue_slots(QueueSlot::new(2).unwrap())
}

fn one_slot_budget() -> BackgroundResourceBudget {
    BackgroundResourceBudget::new().with_queue_slots(QueueSlot::new(1).unwrap())
}

fn three_slot_budget() -> BackgroundResourceBudget {
    BackgroundResourceBudget::new().with_queue_slots(QueueSlot::new(3).unwrap())
}
