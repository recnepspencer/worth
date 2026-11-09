//! A durable original transfers its one admitted response slot through continuation.
use super::*;

fn one_response_slot() -> RuntimeWorldBudgets {
    RuntimeWorldBudgets::install(RuntimeWorldBudgetInstallation {
        branches: RuntimeWorldBranchBudgetInstallation {
            live_product_branches: 8,
        },
        history: RuntimeWorldHistoryBudgetInstallation {
            retained_composite_commits: 8,
            history_metadata_bytes: 1 << 20,
        },
        observations: RuntimeWorldObservationBudgetInstallation {
            active_observations: 2,
        },
        publication: RuntimeWorldPublicationBudgetInstallation {
            active_publication_attempts: 8,
        },
        recovery: RuntimeWorldRecoveryBudgetInstallation {
            retained_product_unpublished_records: 8,
            retained_partial_metadata_bytes: 1 << 20,
        },
        retention: RuntimeWorldRetentionBudgetInstallation {
            unique_exact_component_pins: 64,
            in_flight_pin_acquisition_reservations: 8,
        },
        custody: RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: 8,
        },
    })
    .unwrap()
}

fn complete_with_one_response_slot(pending: bool) {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal_with_budgets(&request, one_response_slot());
    let parent = root(&mut f, &request);
    f.recorder.hold_publication(pending);
    let outcome = create(&f, &request, &parent, "reuse", false, false);
    let outcome = if pending {
        let RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) = outcome else {
            panic!("actual original intent is pending: {outcome:?}")
        };
        assert_eq!(effects.owner_effect_count(), 0);
        let handle = effects.recovery_handle();
        drop(effects);
        f.recorder.hold_publication(false);
        f.world
            .branch_port()
            .resume_creation(
                &request,
                &handle,
                &RuntimeWorldCancellationSource::new().token(),
            )
            .unwrap()
    } else {
        outcome
    };
    let RuntimeWorldBranchCreationOutcome::Performed(child) = outcome else {
        panic!("the prepaid response completes without another admission: {outcome:?}")
    };
    assert_ne!(child.branch_identity(), parent.branch_identity());
    assert_eq!(child.selected_commit(), parent.selected_commit());
    let actual_parent = f
        .world
        .observation_port()
        .observe_product_branch(parent.branch_identity());
    // A fresh third observation must still be refused: neither response nor
    // source was released and reacquired to get past the installed policy.
    assert!(matches!(
        actual_parent,
        Err(RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchAdmissionDenial::CapacityExhausted
        ))
    ));
    assert_eq!(
        f.world
            .inspection_port()
            .recovery_snapshot()
            .unwrap()
            .installed(),
        0
    );
    let expected = (
        parent.branch_identity().name().as_str().to_owned(),
        parent.reference_generation().get(),
        parent.selected_commit().ordinal(),
    );
    drop((parent, child));
    let before = descriptions(&f, &request);
    assert_eq!(before.len(), 2);
    assert!(before
        .iter()
        .any(|b| (&b.0, b.1, b.2) == (&expected.0, expected.1, expected.2)));
    let f = f.reopen(&request);
    assert_eq!(descriptions(&f, &request), before);
    f.finish(&request);
}

#[test]
fn durable_exact_reuse_consumes_its_prepaid_response_with_one_observation_slot_free() {
    complete_with_one_response_slot(false);
}

#[test]
fn pending_exact_reuse_continues_with_its_original_response_slot_and_reopens_shared_history() {
    complete_with_one_response_slot(true);
}
