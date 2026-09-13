use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use worth_proof::TransitionOutcome;

use crate::branch::{
    admit_runtime_signal_branch_observation, SignalBranchRetentionAcquisitionDenial,
    SignalBranchRetentionTerminalOutcome, SignalBranchRetirementDenial,
    SignalBranchRetirementReason,
};

use super::super::super::operation_control::SignalOwnerOperationBoundary;
use super::super::super::tests::runtime_root::runtime_with_two_branches;
use super::super::super::SignalOwnerCancellationSource;

const PROGRESS_BOUND: Duration = Duration::from_secs(2);

#[test]
fn retention_inserted_after_planning_denies_execution_then_release_reopens_it() {
    let (mut runtime, _, target, basis) = runtime_with_two_branches();
    let (_, _, port) = runtime.owner_port_slots().expect("the fixture seals");
    let owner = port.upgrade_owner().expect("the owner remains live");
    let plan = match port.plan_retirement_exact(basis, SignalBranchRetirementReason::Rejected) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the exact plan issues before the new retention: {other:?}"),
    };
    let admission = owner.admit().expect("retention acquisition admits");
    let external = owner
        .acquire_external_retention(&admission, plan.admitted_basis())
        .expect("the saved exact target retains after planning");
    assert!(matches!(
        port.retire_exact(
            plan,
            &SignalOwnerCancellationSource::new().token(),
        ),
        TransitionOutcome::Denied(SignalBranchRetirementDenial::RetainedComponentBasis {
            branch_id,
            active_leases: 1,
        }) if branch_id == target.id
    ));
    assert_eq!(owner.live_count(), 2);
    let contract = owner
        .metadata
        .retirement_contract_observation(&admission, target.id)
        .expect("the denied reservation leaves metadata observable");
    assert_eq!(contract.active_reservations, 0);
    assert_eq!(contract.reserved_receipt_count, 0);
    assert_eq!(
        external.release().outcome(),
        SignalBranchRetentionTerminalOutcome::Released
    );

    let observation = owner
        .observe_branch_exact(&admission, target.id)
        .expect("the unchanged target remains observable");
    let refreshed = admit_runtime_signal_branch_observation(
        observation,
        target.id,
        owner
            .acquire_admitted_retention(&admission, target.id)
            .expect("the unchanged target reissues admitted custody"),
    );
    drop(admission);
    let replacement_plan =
        match port.plan_retirement_exact(refreshed, SignalBranchRetirementReason::Rejected) {
            TransitionOutcome::Success(plan) => plan,
            other => panic!("release reopens exact planning: {other:?}"),
        };
    assert!(matches!(
        port.retire_exact(
            replacement_plan,
            &SignalOwnerCancellationSource::new().token(),
        ),
        TransitionOutcome::Success(receipt) if receipt.retired_branch() == &target
    ));
}

#[test]
fn lifecycle_retirement_reservation_fences_target_while_sibling_retains() {
    let (mut runtime, sibling, target, target_basis) = runtime_with_two_branches();
    let sibling_basis = runtime
        .observe_signal_branch_basis(sibling.clone())
        .expect("the sibling issues a real basis");
    let plan = match runtime.plan_signal_branch_retirement(
        target.clone(),
        target_basis,
        SignalBranchRetirementReason::Superseded,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the exact pre-seal plan issues: {other:?}"),
    };
    let (_, _, port) = runtime.owner_port_slots().expect("the fixture seals");
    let owner = port.upgrade_owner().expect("the owner remains live");
    let admission = owner.admit().expect("the independent caller admits");
    let pause = owner
        .operation_control()
        .arm_pause_once(SignalOwnerOperationBoundary::BeforeCanonicalMovement);
    let (done_tx, done_rx) = mpsc::sync_channel(1);
    let worker_port = port.clone();
    let worker = thread::spawn(move || {
        let outcome = worker_port.retire_exact(plan, &SignalOwnerCancellationSource::new().token());
        let _ = done_tx.send(outcome);
    });
    assert!(pause.wait_until_reached(PROGRESS_BOUND));

    let ledger_before = owner.retention_ledger_observation();
    assert!(matches!(
        owner.acquire_admitted_retention(&admission, target.id),
        Err(SignalBranchRetentionAcquisitionDenial::RetiredBranch { branch_id })
            if branch_id == target.id
    ));
    assert_eq!(owner.retention_ledger_observation(), ledger_before);
    let sibling_lease = owner
        .acquire_external_retention(&admission, &sibling_basis)
        .expect("the unrelated sibling retains while retirement is reserved");
    assert_eq!(
        sibling_lease.release().outcome(),
        SignalBranchRetentionTerminalOutcome::Released
    );
    pause.release();
    assert!(matches!(
        done_rx.recv_timeout(PROGRESS_BOUND),
        Ok(TransitionOutcome::Success(receipt)) if receipt.retired_branch() == &target
    ));
    worker
        .join()
        .expect("the lifecycle retirement worker remains healthy");
}
