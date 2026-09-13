use std::sync::mpsc;
use std::thread;

use worth_signal::facade::branch::{
    AdmittedSignalBranchBasis, SignalBranchRetirementReason, SignalOwnerCancellationSource,
    SignalOwnerOperationBoundary,
};

use super::super::super::world::PROGRESS_BOUND;

pub(super) fn exercise(
    runtime: Option<super::super::super::world::Runtime>,
    lifecycle: super::super::super::world::LifecyclePort,
    mutation: super::super::super::world::MutationPort,
    root_basis: AdmittedSignalBranchBasis,
    retirement_basis: AdmittedSignalBranchBasis,
) {
    let control = runtime
        .as_ref()
        .expect("the strong root remains live")
        .owner_operation_control()
        .expect("the sealed owner issues operation control");
    let pause = control.arm_pause_once(SignalOwnerOperationBoundary::ExactBasisPreflight);
    let worker_lifecycle = lifecycle.clone();
    let worker_mutation = mutation.clone();
    let worker_root_basis = root_basis.clone();
    let (plan_tx, plan_rx) = mpsc::sync_channel(1);
    let (advance_tx, advance_rx) = mpsc::sync_channel(1);

    thread::scope(|scope| {
        scope.spawn(move || {
            let result = worker_lifecycle
                .plan_retirement_exact(retirement_basis, SignalBranchRetirementReason::Superseded);
            let _ = plan_tx.send(result);
        });
        assert!(pause.wait_until_reached(PROGRESS_BOUND));
        scope.spawn(move || {
            let _ = advance_tx.send(super::advance_result(&worker_mutation, &worker_root_basis));
        });
        super::assert_advanced(
            advance_rx
                .recv_timeout(PROGRESS_BOUND)
                .expect("the unrelated branch advances while planning is parked"),
            &root_basis,
        );
        pause.release();
        let plan = match plan_rx
            .recv_timeout(PROGRESS_BOUND)
            .expect("retirement planning reports after release")
        {
            worth_proof::TransitionOutcome::Success(plan) => plan,
            other => panic!("the parked retirement plan remains lawful: {other:?}"),
        };
        assert!(matches!(
            lifecycle.retire_exact(plan, &SignalOwnerCancellationSource::new().token()),
            worth_proof::TransitionOutcome::Success(_)
        ));
    });
    runtime
        .as_ref()
        .expect("the owner root remains live through retirement planning")
        .owner_operation_control()
        .expect("the owner control remains available after contenders finish");
}
