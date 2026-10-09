use std::sync::{Arc, Barrier};

use super::cancellation_residue_assertion::CancellationResidueSnapshot;
use crate::facade::history::BranchId;
use crate::facade::inspection::RelationalMvccCostScope;
use crate::facade::mvcc::{
    RelationalBranchTransactionAdmissionDenial, RelationalCancellationSource,
    RelationalInterruptionBoundary, RelationalOperationControl, RelationalOperationInterruption,
    RelationalTransactionIntent,
};
use crate::tests::support::*;

#[test]
fn admission_interruptions_after_retention_acquisition_release_exactly_once() {
    for interruption in interruption_twins() {
        let runtime = runtime_with_test_schema();
        create_entity(&runtime, "admission-interruption-anchor");
        let identity = runtime.main_branch_identity();
        let reference_before = runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap();
        let observation_scope = RelationalMvccCostScope::capture(&runtime, vec![identity.clone()]);
        let control = injected_control(
            RelationalInterruptionBoundary::ObservationAdmission,
            interruption,
            2,
        );
        let denial = runtime
            .observe_branch_with_control(&identity, &control)
            .unwrap_err();
        assert_eq!(
            denial,
            match interruption {
                RelationalOperationInterruption::Cancelled => {
                    crate::facade::branch::RelationalBranchBasisDenial::Cancelled
                }
                RelationalOperationInterruption::TimedOut => {
                    crate::facade::branch::RelationalBranchBasisDenial::TimedOut
                }
            }
        );
        let observation_cost = runtime.observe_mvcc_counters(&observation_scope).unwrap();
        assert_eq!(
            observation_cost.retention_cost_delta().observation_acquires,
            1
        );
        assert_eq!(
            observation_cost.retention_cost_delta().observation_releases,
            1
        );
        assert_interruption_count(
            &observation_cost,
            RelationalInterruptionBoundary::ObservationAdmission,
            interruption,
        );
        assert_eq!(
            runtime
                .branch_reference_state(&BranchId("main".to_owned()))
                .unwrap(),
            reference_before,
        );

        let (_, basis) = runtime.observe_branch(&identity).unwrap();
        let transaction_scope = RelationalMvccCostScope::capture(&runtime, vec![identity.clone()]);
        let control = injected_control(
            RelationalInterruptionBoundary::TransactionAdmission,
            interruption,
            2,
        );
        let denial = runtime
            .begin_branch_transaction_with_control(
                &basis,
                RelationalTransactionIntent::ordinary(),
                control,
            )
            .unwrap_err();
        assert_eq!(
            denial,
            match interruption {
                RelationalOperationInterruption::Cancelled => {
                    RelationalBranchTransactionAdmissionDenial::Cancelled
                }
                RelationalOperationInterruption::TimedOut => {
                    RelationalBranchTransactionAdmissionDenial::TimedOut
                }
            }
        );
        let transaction_cost = runtime.observe_mvcc_counters(&transaction_scope).unwrap();
        assert_eq!(
            transaction_cost.retention_cost_delta().transaction_acquires,
            1
        );
        assert_eq!(
            transaction_cost.retention_cost_delta().transaction_releases,
            1
        );
        assert_interruption_count(
            &transaction_cost,
            RelationalInterruptionBoundary::TransactionAdmission,
            interruption,
        );
    }
}

#[test]
fn preparation_interruptions_leave_no_candidate_or_publication_residue() {
    for (boundary, trigger_on_visit) in [
        (RelationalInterruptionBoundary::ProposalValidation, 1),
        (RelationalInterruptionBoundary::CandidatePreparation, 3),
    ] {
        for interruption in interruption_twins() {
            let runtime = runtime_with_test_schema();
            create_entity(&runtime, "preparation-interruption-anchor");
            let identity = runtime.main_branch_identity();
            let (_, basis) = runtime.observe_branch(&identity).unwrap();
            let control = injected_control(boundary, interruption, trigger_on_visit);
            let mut transaction = runtime
                .begin_branch_transaction_with_control(
                    &basis,
                    RelationalTransactionIntent::ordinary(),
                    control,
                )
                .unwrap();
            transaction
                .push_batch(
                    batch_create("interrupted-preparation"),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let residue_before =
                CancellationResidueSnapshot::capture(&runtime, &BranchId("main".to_owned()));
            let position_before = runtime.patch_position_reservation_counters();
            let cost_scope = RelationalMvccCostScope::capture(&runtime, vec![identity]);

            let error = runtime
                .prepare_branch_transaction(
                    transaction,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap_err();
            let crate::facade::transactions::TransactionCommitError::Interrupted {
                interruption: event,
                ..
            } = error
            else {
                panic!("preparation interruption must remain typed: {error:?}");
            };
            assert_eq!(event.interruption(), interruption);
            assert_eq!(event.boundary(), boundary);
            assert_eq!(
                CancellationResidueSnapshot::capture(&runtime, &BranchId("main".to_owned())),
                residue_before,
            );
            assert_eq!(
                runtime.patch_position_reservation_counters(),
                position_before
            );
            let cost = runtime.observe_mvcc_counters(&cost_scope).unwrap();
            assert_eq!(cost.retention_cost_delta().transaction_releases, 1);
            if boundary == RelationalInterruptionBoundary::CandidatePreparation {
                assert_eq!(cost.retention_cost_delta().candidate_acquires, 1);
                assert_eq!(cost.retention_cost_delta().candidate_releases, 1);
            } else {
                assert_eq!(cost.retention_cost_delta().candidate_acquires, 0);
            }
            assert_interruption_count(&cost, boundary, interruption);
        }
    }
}

#[test]
fn pre_effect_publication_interruptions_preserve_every_owner_surface() {
    for boundary in [
        RelationalInterruptionBoundary::PublicationPreflight,
        RelationalInterruptionBoundary::BeforeCriticalSection,
    ] {
        for interruption in interruption_twins() {
            let runtime = runtime_with_test_schema();
            create_entity(&runtime, "publication-interruption-anchor");
            let identity = runtime.main_branch_identity();
            let (_, basis) = runtime.observe_branch(&identity).unwrap();
            let control = injected_control(boundary, interruption, 1);
            let residue_before =
                CancellationResidueSnapshot::capture(&runtime, &BranchId("main".to_owned()));
            let mut transaction = runtime
                .begin_branch_transaction_with_control(
                    &basis,
                    RelationalTransactionIntent::ordinary(),
                    control,
                )
                .unwrap();
            transaction
                .push_batch(
                    batch_create("interrupted-publication"),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let candidate = runtime
                .prepare_branch_transaction(
                    transaction,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let position_before = runtime.patch_position_reservation_counters();
            let cost_scope = RelationalMvccCostScope::capture(&runtime, vec![identity]);

            let outcome = runtime.publication_port().compare_and_publish(candidate);
            let event = match outcome {
                crate::mvcc::RelationalPublicationOutcome::Interrupted(event) => event,
                outcome => panic!("pre-effect interruption must defer: {outcome:?}"),
            };
            assert_eq!(event.interruption(), interruption);
            assert_eq!(event.boundary(), boundary);
            assert_eq!(
                CancellationResidueSnapshot::capture(&runtime, &BranchId("main".to_owned())),
                residue_before,
            );
            assert_eq!(
                runtime.patch_position_reservation_counters(),
                position_before
            );
            let cost = runtime.observe_mvcc_counters(&cost_scope).unwrap();
            assert_eq!(cost.retention_cost_delta().candidate_releases, 1);
            assert_interruption_count(&cost, boundary, interruption);
        }
    }
}

#[test]
fn performed_and_settlement_boundaries_report_both_late_interruption_reasons() {
    for boundary in [
        RelationalInterruptionBoundary::AfterLinearization,
        RelationalInterruptionBoundary::Settlement,
    ] {
        for interruption in interruption_twins() {
            let runtime = runtime_with_test_schema();
            create_entity(&runtime, "late-interruption-anchor");
            let identity = runtime.main_branch_identity();
            let (_, basis) = runtime.observe_branch(&identity).unwrap();
            let control = injected_control(boundary, interruption, 1);
            let mut transaction = runtime
                .begin_branch_transaction_with_control(
                    &basis,
                    RelationalTransactionIntent::ordinary(),
                    control,
                )
                .unwrap();
            transaction
                .push_batch(
                    batch_create("late-interruption"),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let candidate = runtime
                .prepare_branch_transaction(
                    transaction,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let reference_before = runtime
                .branch_reference_state(&BranchId("main".to_owned()))
                .unwrap();
            let cost_scope = RelationalMvccCostScope::capture(&runtime, vec![identity]);
            let performed = match runtime.publication_port().compare_and_publish(candidate) {
                crate::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
                outcome => panic!("late interruption must preserve performed work: {outcome:?}"),
            };
            assert_ne!(
                runtime
                    .branch_reference_state(&BranchId("main".to_owned()))
                    .unwrap(),
                reference_before,
            );
            if boundary == RelationalInterruptionBoundary::AfterLinearization {
                let event = performed
                    .late_interruption()
                    .expect("after-linearization interruption is attached immediately");
                assert_eq!(event.interruption(), interruption);
                assert_eq!(event.boundary(), boundary);
            } else {
                assert!(performed.late_interruption().is_none());
            }
            let committed = runtime.settle_performed_publication(performed).unwrap();
            let event = committed
                .late_interruption()
                .expect("performed commit retains its late interruption");
            assert_eq!(event.interruption(), interruption);
            assert_eq!(event.boundary(), boundary);
            let cost = runtime.observe_mvcc_counters(&cost_scope).unwrap();
            assert_interruption_count(&cost, boundary, interruption);
            release_test_commit_snapshot(&runtime, &committed);
        }
    }
}

#[test]
fn cancellation_after_linearization_returns_the_performed_commit() {
    let runtime = runtime_with_test_schema();
    create_entity(&runtime, "late-cancellation-anchor");
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let source = RelationalCancellationSource::new();
    let reached = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let control = RelationalOperationControl::from(source.token())
        .with_post_linearization_pause(Arc::clone(&reached), Arc::clone(&release));
    let mut transaction = runtime
        .begin_branch_transaction_with_control(
            &basis,
            RelationalTransactionIntent::ordinary(),
            control,
        )
        .unwrap();
    transaction
        .push_batch(
            batch_create("late-cancellation-write"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let before = runtime
        .branch_reference_state(&BranchId("main".to_owned()))
        .unwrap();
    let port = runtime.publication_port();

    let publisher = std::thread::spawn(move || port.compare_and_publish(candidate));
    reached.wait();
    assert_ne!(
        runtime
            .branch_reference_state(&BranchId("main".to_owned()))
            .unwrap(),
        before,
        "the owner reference has already crossed its linearization point"
    );
    source.cancel();
    release.wait();

    let performed = match publisher.join().unwrap() {
        crate::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("late cancellation cannot erase performed movement: {outcome:?}"),
    };
    let interruption = performed
        .late_interruption()
        .expect("late cancellation remains attached to the performed result");
    assert_eq!(
        interruption.interruption(),
        RelationalOperationInterruption::Cancelled
    );
    assert_eq!(
        interruption.boundary(),
        RelationalInterruptionBoundary::AfterLinearization
    );
    let committed = runtime
        .settle_performed_publication(performed)
        .expect("performed publication remains settleable after late cancellation");
    release_test_commit_snapshot(&runtime, &committed);
}

fn interruption_twins() -> [RelationalOperationInterruption; 2] {
    [
        RelationalOperationInterruption::Cancelled,
        RelationalOperationInterruption::TimedOut,
    ]
}

fn injected_control(
    boundary: RelationalInterruptionBoundary,
    interruption: RelationalOperationInterruption,
    trigger_on_visit: usize,
) -> RelationalOperationControl {
    RelationalOperationControl::uninterrupted().with_injected_interruption(
        boundary,
        interruption,
        trigger_on_visit,
    )
}

fn assert_interruption_count(
    cost: &crate::facade::inspection::RelationalMvccCounterObservation,
    boundary: RelationalInterruptionBoundary,
    interruption: RelationalOperationInterruption,
) {
    assert_eq!(
        cost.interruption_cost_delta().count(boundary, interruption),
        1,
    );
}
