use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::conditional_operation::clock_observation::erased::ErasedClockObservationOutcome;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::conditional_operation::clock_observation::erased::ErasedClockObservationReceipt;
use worth_query_execution::facade::domain_computation::WorthQueryConditionalClockObservationFailureKind;
use worth_query_execution::facade::domain_computation::WorthQueryConditionalClockObservationOutcome;

pub struct TestClock;

pub fn granular_installation(
) -> worth_query_execution::facade::domain_computation::WorthQueryGranularInvalidationInstallation {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    world.application.granular_invalidation_installation()
}

pub fn receipt() -> ErasedClockObservationReceipt {
    ErasedClockObservationReceipt::new_for_test(7, 41, 2, true, 2, true, 3, 1, 1, 0, 1, 2, 1, 3, 1, Some(17), false, Vec::new(), Vec::new(), worth_query_execution::facade::domain_computation::WorthQueryGranularInvalidationCoverage::Exact)
}

#[test]
fn typed_receipt_preserves_query_owned_clock_and_work_evidence() {
    let installation = granular_installation();
    let mut receipt = receipt().typed_for_test::<TestClock>(installation.clone(), None);

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
            .typed_for_test::<TestClock>(granular_installation(), None),
        WorthQueryConditionalClockObservationOutcome::Accepted(_)
    ));
    assert!(matches!(
        ErasedClockObservationOutcome::Duplicate(receipt())
            .typed_for_test::<TestClock>(granular_installation(), None),
        WorthQueryConditionalClockObservationOutcome::Duplicate(_)
    ));
    assert!(matches!(
        ErasedClockObservationOutcome::Stale
            .typed_for_test::<TestClock>(granular_installation(), None),
        WorthQueryConditionalClockObservationOutcome::Stale
    ));
    assert!(matches!(
        ErasedClockObservationOutcome::Reordered
            .typed_for_test::<TestClock>(granular_installation(), None),
        WorthQueryConditionalClockObservationOutcome::Reordered
    ));
    assert!(matches!(
        ErasedClockObservationOutcome::Closed
            .typed_for_test::<TestClock>(granular_installation(), None),
        WorthQueryConditionalClockObservationOutcome::Closed
    ));
    let failed = ErasedClockObservationOutcome::Failed {
        kind: WorthQueryConditionalClockObservationFailureKind::ObservationFailed,
        detail: "clock read failed".to_string(),
    }
    .typed_for_test::<TestClock>(granular_installation(), None);
    let WorthQueryConditionalClockObservationOutcome::Failed(failed) = failed else {
        panic!("provider failure must remain a Query-owned failure posture");
    };
    assert_eq!(
        failed.kind(),
        WorthQueryConditionalClockObservationFailureKind::ObservationFailed
    );
    assert_eq!(failed.detail(), "clock read failed");
}
