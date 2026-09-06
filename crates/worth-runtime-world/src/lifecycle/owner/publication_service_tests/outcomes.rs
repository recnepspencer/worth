use super::*;

#[test]
fn ready_drop_retains_effects_and_allows_close_before_explicit_cleanup() {
    let (fixture, owner, expected) = setup();
    let ready = ready_relational(&fixture, &owner, &expected, "publication-service");
    assert_eq!(owner.state.operation.active(), 1);
    assert_eq!(owner.state.publication_capacity.active(), 1);
    assert_eq!(owner.state.history.reserved_len(), 1);
    assert_eq!(owner.state.recovery.reserved_slots(), 1);
    assert_eq!(owner.recovery_record_count(), 0);
    assert_eq!(
        owner
            .close()
            .expect_err("a live reserved attempt denies close"),
        RuntimeWorldCloseDenial::AlreadyClosing
    );
    assert_eq!(
        owner.lifecycle_observation(),
        crate::lifecycle::RuntimeWorldOwnerLifecycleObservation::Open
    );

    drop(ready);
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(owner.state.publication_capacity.active(), 0);
    assert_eq!(owner.state.history.reserved_len(), 1);
    assert_eq!(owner.state.recovery.reserved_slots(), 0);
    assert_eq!(owner.recovery_record_count(), 1);
    let report = owner
        .close()
        .expect("close succeeds after attempt teardown");
    assert_eq!(report.retained_records().len(), 1);
    let row = &report.retained_records()[0];
    assert_eq!(row.cause(), ProductUnpublishedCause::CallerAbandoned);
    // Retained publication pair plus the pre-acquired history pair.
    assert_eq!(row.live_component_obligations(), 4);
    assert_eq!(row.live_composite_obligations(), 2);
    let handle = owner.recovery_handles().pop().unwrap();
    let record = owner.inspect_recovery(&handle).unwrap();
    assert_eq!(record.owner_effect_count(), 1);
    assert_eq!(
        record.retention_posture(),
        crate::recovery::ProductUnpublishedRetentionPosture::PublicationPinsRetained
    );
    assert!(record.successor_commit().is_none());
    drop(record);
    assert!(owner.cleanup_recovery_handle(&handle).is_some());
    assert_eq!(owner.state.history.reserved_len(), 0);
    assert_eq!(owner.recovery_record_count(), 0);
    assert_eq!(owner.state.recovery.metadata_bytes(), 0);
    assert_eq!(
        owner.lifecycle_observation(),
        crate::lifecycle::RuntimeWorldOwnerLifecycleObservation::Closed
    );
}

#[test]
fn service_dispatch_records_one_exact_final_publication() {
    let (fixture, owner, expected) = setup();
    let cell = owner.state.branches.root_cell().expect("bootstrapped cell");
    let ready = ready_relational(&fixture, &owner, &expected, "publication-service");
    let outcome = crate::lifecycle::ports::RuntimeWorldProductPublicationService::publish(
        owner.as_ref(),
        ready,
        &cell,
        CompositeLateCancellationPosture::NotRequested,
    );
    let performed = match outcome {
        RuntimeWorldPublicationOutcome::Performed(performed) => performed,
        other => panic!("service publication must perform: {other:?}"),
    };
    let counters = performed.cost_counters();
    assert_eq!(counters.expected_head_rechecks(), 1);
    assert_eq!(counters.history_slots_installed(), 1);
    // SPEC-P4-016 reads the cell twice on the winning path: once for the
    // expected-observation comparison that precedes materialization, and once
    // inside the CAS itself.
    assert_eq!(counters.product_cell_touches(), 2);
    assert_eq!(counters.cas_attempts(), 1);
    assert_eq!(counters.cas_wins(), 1);
    assert_eq!(counters.cas_losses(), 0);
    assert_eq!(counters.cancellation_observations(), 0);
    assert_eq!(performed.old_product_head(), expected.snapshot());
    assert_eq!(performed.new_product_head(), &cell.atomic_snapshot());
    assert_eq!(owner.state.operation.active(), 0);
}

#[test]
fn cancellation_after_owner_movement_retains_partial_until_explicit_cleanup() {
    let (fixture, owner, expected) = setup();
    let cell = owner.state.branches.root_cell().expect("bootstrapped cell");
    let ready = ready_relational(&fixture, &owner, &expected, "publication-service");
    let before = cell.atomic_snapshot();
    let outcome = crate::lifecycle::ports::RuntimeWorldProductPublicationService::publish(
        owner.as_ref(),
        ready,
        &cell,
        CompositeLateCancellationPosture::RequestedBeforeProductMovement,
    );
    let retained = match outcome {
        RuntimeWorldPublicationOutcome::ProductUnpublished(retained) => retained,
        other => panic!("cancelled owner effects must be retained: {other:?}"),
    };
    assert_eq!(
        retained.cause(),
        ProductUnpublishedCause::CancellationAfterEffect
    );
    assert_eq!(cell.atomic_snapshot(), before);
    assert_eq!(owner.state.recovery.reserved_slots(), 0);
    assert_eq!(owner.recovery_record_count(), 1);
    assert_eq!(owner.state.operation.active(), 0);
    assert!(
        owner.cleanup_recovery(retained).is_some(),
        "a publication's retained record is released by its own capability"
    );
    assert_eq!(owner.recovery_record_count(), 0);
    let _report = owner
        .close()
        .expect("close succeeds after recovery custody drops");
}

#[test]
fn cancellation_before_product_movement_does_not_cas_current_head() {
    let (fixture, owner, expected) = setup();
    let cell = owner.state.branches.root_cell().expect("bootstrapped cell");
    let ready = ready_relational(&fixture, &owner, &expected, "publication-service");
    let before = cell.atomic_snapshot();
    let outcome = crate::lifecycle::ports::RuntimeWorldProductPublicationService::publish(
        owner.as_ref(),
        ready,
        &cell,
        CompositeLateCancellationPosture::RequestedBeforeProductMovement,
    );
    let retained = match outcome {
        RuntimeWorldPublicationOutcome::ProductUnpublished(retained) => retained,
        other => panic!("current-head cancellation must retain owner effects: {other:?}"),
    };
    assert_eq!(
        retained.cause(),
        ProductUnpublishedCause::CancellationAfterEffect
    );
    assert_eq!(cell.atomic_snapshot(), before);
    assert_eq!(owner.state.recovery.reserved_slots(), 0);
    assert_eq!(owner.recovery_record_count(), 1);
    let handle = retained.recovery_handle();
    drop(retained);
    // SPEC-P4-008: an installed retained record is exposed in the terminal
    // report, never refused as an undrainable critical section.
    let report = owner
        .close()
        .expect("close exposes installed recovery custody instead of refusing it");
    let row = report
        .retained_records()
        .iter()
        .find(|row| row.identity() == handle.identity())
        .expect("close names the retained cancellation record");
    assert_eq!(
        row.cause(),
        ProductUnpublishedCause::CancellationAfterEffect
    );
    assert!(owner.cleanup_recovery_handle(&handle).is_some());
}

#[test]
fn cancellation_observed_after_movement_is_performed_with_evidence() {
    let (fixture, owner, expected) = setup();
    let cell = owner.state.branches.root_cell().expect("bootstrapped cell");
    let ready = ready_relational(&fixture, &owner, &expected, "publication-service");
    let outcome = crate::lifecycle::ports::RuntimeWorldProductPublicationService::publish(
        owner.as_ref(),
        ready,
        &cell,
        CompositeLateCancellationPosture::RequestedAfterProductMovement,
    );
    let performed = match outcome {
        RuntimeWorldPublicationOutcome::Performed(performed) => performed,
        other => panic!("post-movement cancellation must preserve publication: {other:?}"),
    };
    assert_eq!(
        performed.late_cancellation(),
        CompositeLateCancellationPosture::RequestedAfterProductMovement
    );
    assert_eq!(performed.cost_counters().cancellation_observations(), 1);
}
