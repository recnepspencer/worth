use std::sync::{mpsc, Arc};
use std::thread;

use worth_foundational::FoundationalBranchTargetBasis;

use crate::branch::{
    SignalBranchRetentionAcquisitionDenial, SignalBranchRetentionTerminalOutcome,
    SignalOwnerLifecycleObservation,
};

use super::super::{
    SignalOwnerAdmissionDenial, SignalOwnerLifecycleState, SignalOwnerServiceCounters,
};
use super::progress_bound::{wait_until_progress, PROGRESS_BOUND};
use super::runtime_root::runtime_with_two_branches;

#[test]
fn direct_lease_terminality_linearizes_before_or_during_owner_close() {
    let (mut runtime, _, branch, basis) = runtime_with_two_branches();
    let (basis_port, _, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = basis_port.upgrade_owner().expect("owner remains live");
    let acquisition = owner.admit().expect("direct retention acquisition admits");
    let released_before_close = owner
        .acquire_external_retention(&acquisition, &basis)
        .expect("the first direct obligation opens");
    let released_after_close = owner
        .acquire_external_retention(&acquisition, &basis)
        .expect("the second direct obligation opens before the close fence");
    drop(acquisition);

    let before_receipt = released_before_close.release();
    assert_eq!(
        before_receipt.outcome(),
        SignalBranchRetentionTerminalOutcome::Released
    );
    assert_eq!(before_receipt.branch_id(), branch.id);
    assert_eq!(before_receipt.remaining_target_leases(), 1);
    assert_eq!(before_receipt.remaining_branch_leases(), 1);

    let (fenced_tx, fenced_rx) = mpsc::sync_channel(1);
    let (continue_tx, continue_rx) = mpsc::sync_channel(1);
    let (closed_tx, closed_rx) = mpsc::sync_channel(1);
    let closing_owner = Arc::clone(&owner);
    thread::spawn(move || {
        let mut first_batch = true;
        let result = closing_owner.close_with_cleanup_observer(|_, _| {
            if !first_batch {
                return;
            }
            first_batch = false;
            fenced_tx
                .send(())
                .expect("the close observer reports the retention fence");
            continue_rx
                .recv()
                .expect("the bounded close observer is released");
        });
        let _ = closed_tx.send(result);
    });
    assert_eq!(fenced_rx.recv_timeout(PROGRESS_BOUND), Ok(()));
    assert_eq!(
        owner.lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closing,
        "the real close is fenced but cleanup remains incomplete"
    );

    let after_receipt = released_after_close.release();
    assert_eq!(
        after_receipt.outcome(),
        SignalBranchRetentionTerminalOutcome::OwnerUnavailable
    );
    assert_eq!(after_receipt.branch_id(), branch.id);
    assert_eq!(after_receipt.remaining_target_leases(), 0);
    assert_eq!(after_receipt.remaining_branch_leases(), 0);
    continue_tx
        .send(())
        .expect("the bounded close observer resumes cleanup");
    assert_eq!(closed_rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(())));
    assert_eq!(
        owner.lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );

    let counts = owner.retention_terminal_counts();
    assert_eq!(counts.explicit_releases(), 2);
    assert_eq!(counts.dropped_releases(), 0);
    assert_eq!(counts.owner_loss_releases(), 1);
    assert_eq!(counts.unknown_lease_defenses(), 0);
}

#[test]
fn prior_admission_retains_acquisition_rights_during_closing_and_closed_denies_fresh_work() {
    let (mut runtime, _, branch, basis) = runtime_with_two_branches();
    let (basis_port, _, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = basis_port.upgrade_owner().expect("owner remains live");
    let admission = owner.admit().expect("operation admits before close");
    let target_key = basis
        .descriptor()
        .observation()
        .target()
        .as_basis()
        .expect("the exact fixture descriptor carries a basis target")
        .canonical_encoding();
    let before = owner.retention_ledger_observation();
    let before_contacts = owner.cost_snapshot().retention_registry_contacts();

    let closing_owner = Arc::clone(&owner);
    let (closed_tx, closed_rx) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let _ = closed_tx.send(closing_owner.close());
    });
    assert!(wait_until_progress(
        "owner enters Closing with an older admission",
        || { owner.lifecycle_observation() == SignalOwnerLifecycleObservation::Closing }
    ));
    assert!(matches!(
        owner.admit(),
        Err(SignalOwnerAdmissionDenial::OwnerUnavailable)
    ));
    assert_eq!(owner.retention_ledger_observation(), before);
    assert_eq!(
        owner.cost_snapshot().retention_registry_contacts(),
        before_contacts
    );

    let admitted = owner
        .acquire_admitted_retention(&admission, branch.id)
        .expect("the pre-close admission reserves its output during Closing");
    let external = owner
        .acquire_external_retention(&admission, &basis)
        .expect("the same pre-close admission opens its exact obligation during Closing");
    let acquired = owner.retention_ledger_observation();
    assert_eq!(acquired.next_lease_id, before.next_lease_id + 2);
    assert_eq!(acquired.used_capacity, before.used_capacity + 2);
    assert_eq!(
        acquired.admitted_lease_count,
        before.admitted_lease_count + 1
    );
    assert_eq!(
        acquired.external_lease_count,
        before.external_lease_count + 1
    );
    assert_eq!(
        acquired.admitted_lease_identities,
        {
            let mut expected = before.admitted_lease_identities.clone();
            expected.push((before.next_lease_id + 1, branch.id));
            expected.sort_unstable();
            expected
        },
        "the admitted output owns its exact reserved identity"
    );
    assert_eq!(
        acquired.external_lease_identities,
        vec![(before.next_lease_id + 2, branch.id, target_key.clone())],
        "the external obligation owns its exact branch and target key"
    );
    assert_eq!(
        acquired.admitted_branch_total_count,
        before.admitted_branch_total_count + 1
    );
    assert_eq!(
        acquired.external_branch_total_count,
        before.external_branch_total_count + 1
    );
    assert_eq!(
        acquired.external_target_total_count,
        before.external_target_total_count + 1
    );
    assert_eq!(acquired.reserved_admitted_lease_count, 0);
    assert_eq!(acquired.reserved_branch_total_count, 0);
    assert_eq!(acquired.admitted_count_by_branch, vec![(branch.id, 2)]);
    assert!(acquired.reserved_count_by_branch.is_empty());
    assert_eq!(acquired.external_count_by_branch, vec![(branch.id, 1)]);
    assert_eq!(acquired.external_count_by_target, vec![(target_key, 1)]);
    assert_eq!(acquired.maximum_active_leases, before.maximum_active_leases);

    assert_eq!(
        external.release().outcome(),
        SignalBranchRetentionTerminalOutcome::OwnerUnavailable
    );
    drop(admitted);
    let mut expected_released = before.clone();
    expected_released.next_lease_id += 2;
    assert_eq!(
        owner.retention_ledger_observation(),
        expected_released,
        "release returns every keyed map and lease identity while preserving the monotonic allocator"
    );

    drop(admission);
    assert_eq!(closed_rx.recv_timeout(PROGRESS_BOUND), Ok(Ok(())));
    assert_eq!(
        owner.lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
    let closed = owner.retention_ledger_observation();
    let expired_lifecycle = SignalOwnerLifecycleState::new(
        owner.runtime_instance_id(),
        Arc::new(SignalOwnerServiceCounters::default()),
    );
    let expired = expired_lifecycle
        .admit(owner.runtime_instance_id())
        .expect("independent lifecycle issues a real but expired admission");
    assert!(matches!(
        owner.acquire_external_retention(&expired, &basis),
        Err(SignalBranchRetentionAcquisitionDenial::OwnerUnavailable(_))
    ));
    assert_eq!(owner.retention_ledger_observation(), closed);
}

#[test]
fn foreign_expired_and_reentrant_acquisition_deny_before_retention_contact() {
    let (mut runtime_a, _, branch_a, basis_a) = runtime_with_two_branches();
    let (mut runtime_b, _, _, _) = runtime_with_two_branches();
    let (basis_a_port, _, _) = runtime_a.owner_port_slots().expect("owner A seals");
    let (basis_b_port, _, _) = runtime_b.owner_port_slots().expect("owner B seals");
    let owner_a = basis_a_port.upgrade_owner().expect("owner A remains live");
    let owner_b = basis_b_port.upgrade_owner().expect("owner B remains live");
    let admission_a = owner_a.admit().expect("owner A admits");
    let second_admission_a = owner_a.admit().expect("owner A admits an idle peer");
    let admission_b = owner_b.admit().expect("owner B admits");
    let cell_a = owner_a
        .lookup_cell(&admission_a, branch_a.id)
        .expect("owner A cell is live");
    let before = owner_a.retention_ledger_observation();
    let before_contacts = owner_a.cost_snapshot().retention_registry_contacts();

    assert!(matches!(
        owner_a.acquire_admitted_retention(&admission_b, branch_a.id),
        Err(SignalBranchRetentionAcquisitionDenial::OwnerUnavailable(_))
    ));
    assert!(matches!(
        owner_a.acquire_external_retention(&admission_b, &basis_a),
        Err(SignalBranchRetentionAcquisitionDenial::OwnerUnavailable(_))
    ));
    let expired_lifecycle = SignalOwnerLifecycleState::new(
        owner_a.runtime_instance_id(),
        Arc::new(SignalOwnerServiceCounters::default()),
    );
    let expired = expired_lifecycle
        .admit(owner_a.runtime_instance_id())
        .expect("same runtime number admits a different lifecycle incarnation");
    assert!(matches!(
        owner_a.acquire_admitted_retention(&expired, branch_a.id),
        Err(SignalBranchRetentionAcquisitionDenial::OwnerUnavailable(_))
    ));

    cell_a
        .with_state(&admission_a, |_, _| {
            assert!(matches!(
                owner_a.acquire_admitted_retention(&admission_a, branch_a.id),
                Err(SignalBranchRetentionAcquisitionDenial::OwnerReentry)
            ));
            assert!(matches!(
                owner_a.acquire_external_retention(&second_admission_a, &basis_a),
                Err(SignalBranchRetentionAcquisitionDenial::OwnerReentry)
            ));
        })
        .expect("the outer real cell operation remains healthy");
    assert_eq!(owner_a.retention_ledger_observation(), before);
    assert_eq!(
        owner_a.cost_snapshot().retention_registry_contacts(),
        before_contacts
    );

    let healthy = owner_a
        .acquire_external_retention(&admission_a, &basis_a)
        .expect("released cell posture permits the healthy acquisition");
    assert_eq!(healthy.release().remaining_branch_leases(), 0);
}
