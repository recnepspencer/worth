use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::branch::admit_runtime_signal_branch_observation;
use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;
use crate::branch::owner_services::SignalOwnerCancellationSource;
use crate::branch::SignalBranchBasisObservationDenial;

use super::super::cancellation::restore::{restore_fixture, PopulatedRestoreFixture};

#[test]
fn restore_post_movement_faults_preserve_performed_truth_and_release_output_custody() {
    for boundary in [
        SignalOwnerOperationBoundary::AfterCanonicalMovement,
        SignalOwnerOperationBoundary::OutcomeConstruction,
    ] {
        exercise_restore_post_movement_fault(boundary);
    }
}

fn exercise_restore_post_movement_fault(boundary: SignalOwnerOperationBoundary) {
    let PopulatedRestoreFixture {
        _runtime,
        owner,
        cell,
        branch,
        snapshot,
        current_basis,
        snapshot_source,
        live_source: _,
        dispatch,
    } = restore_fixture();
    let admission = owner.admit().expect("restore fault admits");
    let ledger_before = owner.retention_ledger_observation();
    let cell_before = cell.cost_snapshot();
    let metadata_before = owner.metadata.pending_snapshot_reservation_count();
    let expected_generation = current_basis.observation().generation().get() + 1;
    let snapshot_id = snapshot.snapshot().meta.snapshot_id;
    owner.operation_control().inject_panic_once(boundary);

    let fault = catch_unwind(AssertUnwindSafe(|| {
        let ready = owner
            .reserve_restore_output(&admission, &cell)
            .expect("restore output custody reserves")
            .restore(
                &current_basis,
                &snapshot,
                &SignalOwnerCancellationSource::new().token(),
            )
            .expect("restore reaches its post-movement outcome seam");
        let _ = ready.into_basis();
    }));
    assert!(fault.is_err(), "{boundary:?} must inject a restore fault");

    let truth = cell.restore_state_truth_after_fault(dispatch);
    assert_eq!(truth.handle.id, branch.id);
    assert_eq!(truth.handle.head_snapshot_id, Some(snapshot_id));
    assert_eq!(truth.generation, expected_generation);
    assert_eq!(truth.restore_snapshot_id, Some(snapshot_id));
    assert_eq!(truth.observation.generation().get(), expected_generation);
    assert_eq!(truth.dependency_sources, vec![snapshot_source]);
    assert_eq!(
        truth
            .observation
            .target()
            .as_basis()
            .and_then(|target| target.snapshot_id()),
        Some(snapshot_id.0)
    );
    assert_eq!(
        truth
            .observation
            .target()
            .as_basis()
            .and_then(|target| target.restore_snapshot_id()),
        Some(snapshot_id.0)
    );
    assert!(owner
        .metadata
        .has_snapshot_state(&admission, branch.id, snapshot_id)
        .expect("restore snapshot metadata remains owner-observable"));
    assert_eq!(
        owner.metadata.pending_snapshot_reservation_count(),
        metadata_before
    );
    assert_eq!(
        cell.cost_snapshot().movements(),
        cell_before.movements() + 1
    );
    let mut released = ledger_before.clone();
    released.next_lease_id += 1;
    assert_eq!(owner.retention_ledger_observation(), released);

    match boundary {
        SignalOwnerOperationBoundary::AfterCanonicalMovement => {
            assert!(matches!(
                cell.observe_exact(&admission),
                Err(SignalBranchBasisObservationDenial::QuarantinedBranch { branch_id })
                    if branch_id == branch.id
            ));
            assert!(cell.poison_recovery().is_some());
        }
        SignalOwnerOperationBoundary::OutcomeConstruction => {
            assert_eq!(cell.poison_recovery(), None);
            let observation = cell
                .observe_exact(&admission)
                .expect("outcome panic leaves the performed restore cell healthy");
            let basis = admit_runtime_signal_branch_observation(
                observation,
                branch.id,
                owner
                    .acquire_admitted_retention(&admission, branch.id)
                    .expect("performed restore can be readmitted"),
            );
            let healthy = owner
                .reserve_advance_output(&admission, &cell)
                .expect("performed restore releases output capacity")
                .advance::<(), (), _>(
                    &basis,
                    &mut (),
                    &SignalOwnerCancellationSource::new().token(),
                    |_| Ok(()),
                )
                .into_result()
                .expect("a healthy target operation follows outcome unwind");
            let _ = healthy.into_parts();
        }
        _ => unreachable!("the restore unwind matrix names only post-movement seams"),
    }

    let sibling_id = branch
        .parent_branch_id
        .expect("the populated restore branch has a real parent sibling");
    let sibling = owner
        .lookup_cell(&admission, sibling_id)
        .expect("the unrelated sibling remains registered");
    sibling
        .observe_exact(&admission)
        .expect("unrelated branch progresses after restore unwind");
    assert_eq!(sibling.branch_id(), sibling_id);

    let healthy_fixture = restore_fixture();
    let healthy_admission = healthy_fixture.owner.admit().expect("healthy twin admits");
    let healthy = healthy_fixture
        .owner
        .reserve_restore_output(&healthy_admission, &healthy_fixture.cell)
        .expect("healthy twin reserves")
        .restore(
            &healthy_fixture.current_basis,
            &healthy_fixture.snapshot,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("healthy populated twin restores")
        .into_basis();
    assert_eq!(
        healthy.observation().generation().get(),
        healthy_fixture
            .current_basis
            .observation()
            .generation()
            .get()
            + 1
    );
    let healthy_truth = healthy_fixture
        .cell
        .restore_state_truth_after_fault(healthy_fixture.dispatch);
    assert_eq!(
        healthy_truth.dependency_sources,
        vec![healthy_fixture.snapshot_source]
    );
    assert_eq!(
        truth.graph, healthy_truth.graph,
        "the faulted restore publishes the same full serialized graph authority as an independent healthy twin"
    );
    assert_eq!(
        truth.mutation_ledger, healthy_truth.mutation_ledger,
        "the faulted restore publishes the exact healthy mutation-ledger posture"
    );
}
