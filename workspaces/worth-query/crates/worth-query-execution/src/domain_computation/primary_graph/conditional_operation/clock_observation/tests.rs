use super::*;

struct TestClock;

fn granular_installation(
) -> crate::domain_computation::primary_graph::WorthQueryGranularInvalidationInstallation {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    world.application.granular_invalidation_installation()
}

fn receipt() -> ErasedClockObservationReceipt {
    ErasedClockObservationReceipt {
        sequence: 7,
        observed_coordinate: 41,
        due_wake_count: 2,
        due_work_remaining: true,
        authoritative_commit_count: 2,
        authoritative_work_remaining: true,
        retained_due_wake_count: 3,
        retained_eligible_wake_count: 1,
        retained_suppressed_wake_count: 1,
        retained_deferred_wake_count: 0,
        retained_failed_wake_count: 1,
        committed_operation_count: 2,
        already_committed_operation_count: 1,
        failed_operation_count: 3,
        indeterminate_operation_count: 1,
        snapshot_capacity_backpressure: Some(17),
        retention_capacity_backpressure: false,
        execution_provenance: Vec::new(),
        granular_invalidations: Vec::new(),
    }
}

#[test]
fn typed_receipt_preserves_query_owned_clock_and_work_evidence() {
    let installation = granular_installation();
    let mut receipt = receipt().typed::<TestClock>(installation.clone(), None);

    assert_eq!(receipt.sequence(), 7);
    assert_eq!(receipt.observed_time().nanoseconds(), 41);
    assert_eq!(receipt.due_wake_count(), 2);
    assert!(receipt.due_work_remaining());
    assert_eq!(receipt.authoritative_commit_count(), 2);
    assert!(receipt.authoritative_work_remaining());
    assert_eq!(receipt.retained_due_wake_count(), 3);
    assert_eq!(receipt.retained_eligible_wake_count(), 1);
    assert_eq!(receipt.retained_suppressed_wake_count(), 1);
    assert_eq!(receipt.retained_deferred_wake_count(), 0);
    assert_eq!(receipt.retained_failed_wake_count(), 1);
    assert_eq!(receipt.committed_operation_count(), 2);
    assert_eq!(receipt.already_committed_operation_count(), 1);
    assert_eq!(receipt.failed_operation_count(), 3);
    assert_eq!(receipt.indeterminate_operation_count(), 1);
    assert_eq!(receipt.snapshot_capacity_backpressure(), Some(17));
    let granular = receipt.take_granular_invalidation_batch();
    assert!(installation.admits_batch(&granular));
    assert!(granular.is_empty());
    assert_eq!(granular.observation().direct_truth_delivery_count(), 0);
    assert_eq!(granular.observation().signal_performed_delivery_count(), 0);
}

#[test]
fn erased_postures_map_without_lower_runtime_evidence() {
    assert!(matches!(
        ErasedClockObservationOutcome::Accepted(receipt())
            .typed::<TestClock>(granular_installation(), None),
        WorthQueryConditionalClockObservationOutcome::Accepted(_)
    ));
    assert!(matches!(
        ErasedClockObservationOutcome::Duplicate(receipt())
            .typed::<TestClock>(granular_installation(), None),
        WorthQueryConditionalClockObservationOutcome::Duplicate(_)
    ));
    assert!(matches!(
        ErasedClockObservationOutcome::Stale.typed::<TestClock>(granular_installation(), None),
        WorthQueryConditionalClockObservationOutcome::Stale
    ));
    assert!(matches!(
        ErasedClockObservationOutcome::Reordered.typed::<TestClock>(granular_installation(), None),
        WorthQueryConditionalClockObservationOutcome::Reordered
    ));
    assert!(matches!(
        ErasedClockObservationOutcome::Closed.typed::<TestClock>(granular_installation(), None),
        WorthQueryConditionalClockObservationOutcome::Closed
    ));
    let failed = ErasedClockObservationOutcome::Failed {
        kind: WorthQueryConditionalClockObservationFailureKind::ObservationFailed,
        detail: "clock read failed".to_string(),
    }
    .typed::<TestClock>(granular_installation(), None);
    let WorthQueryConditionalClockObservationOutcome::Failed(failed) = failed else {
        panic!("provider failure must remain a Query-owned failure posture");
    };
    assert_eq!(
        failed.kind(),
        WorthQueryConditionalClockObservationFailureKind::ObservationFailed
    );
    assert_eq!(failed.detail(), "clock read failed");
}
