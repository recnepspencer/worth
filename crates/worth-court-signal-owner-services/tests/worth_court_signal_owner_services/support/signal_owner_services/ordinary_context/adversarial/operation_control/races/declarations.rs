pub(crate) use {crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld, crate::signal_owner_services::ordinary_context::adversarial::world::PROGRESS_BOUND};
pub(crate) use {crate::signal_owner_services::ordinary_context::execution_custody};
pub(crate) use {std::sync::mpsc, std::sync::Arc, std::sync::Barrier};
pub(crate) use {std::thread};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_signal::facade::branch::AdmittedSignalBranchBasis, worth_signal::facade::branch::AdmittedSignalBranchSnapshot, worth_signal::facade::branch::SignalBranchAdvanceDenial, worth_signal::facade::branch::SignalBranchRestoreDenial, worth_signal::facade::branch::SignalBranchRetirementDenial, worth_signal::facade::branch::SignalBranchRetirementReason, worth_signal::facade::branch::SignalBranchSnapshotCaptureDenial, worth_signal::facade::branch::SignalOwnerCancellationSource};


pub(crate) fn assert_one_success(
    left: Result<(), &'static str>,
    right: Result<(), &'static str>,
    label: &str,
) {
    assert_eq!(
        [left.is_ok(), right.is_ok()]
            .into_iter()
            .filter(|succeeded| *succeeded)
            .count(),
        1,
        "{label} must have exactly one canonical winner: left={left:?} right={right:?}"
    );
}
pub(crate) fn advance_denial(denial: SignalBranchAdvanceDenial) -> &'static str {
    match denial {
        worth_signal::facade::branch::SignalBranchAdvanceDenial::BasisMismatch { .. } => "BasisMismatch",
        worth_signal::facade::branch::SignalBranchAdvanceDenial::RetirementInProgress { .. } => "RetirementInProgress",
        worth_signal::facade::branch::SignalBranchAdvanceDenial::RetiredBranch { .. } => "RetiredBranch",
        other => panic!("unexpected advance denial: {other:?}"),
    }
}
pub(crate) fn restore_denial(denial: SignalBranchRestoreDenial) -> &'static str {
    match denial {
        worth_signal::facade::branch::SignalBranchRestoreDenial::BasisMismatch { .. } => "BasisMismatch",
        worth_signal::facade::branch::SignalBranchRestoreDenial::RetirementInProgress { .. } => "RetirementInProgress",
        worth_signal::facade::branch::SignalBranchRestoreDenial::RetiredBranch { .. } => "RetiredBranch",
        other => panic!("unexpected restore denial: {other:?}"),
    }
}
pub(crate) fn capture_denial(denial: SignalBranchSnapshotCaptureDenial) -> &'static str {
    match denial {
        worth_signal::facade::branch::SignalBranchSnapshotCaptureDenial::BasisMismatch { .. } => "BasisMismatch",
        worth_signal::facade::branch::SignalBranchSnapshotCaptureDenial::RetirementInProgress { .. } => "RetirementInProgress",
        worth_signal::facade::branch::SignalBranchSnapshotCaptureDenial::RetiredBranch { .. } => "RetiredBranch",
        other => panic!("unexpected snapshot denial: {other:?}"),
    }
}
pub(crate) fn retirement_denial(denial: SignalBranchRetirementDenial) -> &'static str {
    match denial {
        worth_signal::facade::branch::SignalBranchRetirementDenial::StaleBranchHead { .. } => "StaleBranchHead",
        worth_signal::facade::branch::SignalBranchRetirementDenial::RetirementInProgress { .. } => "RetirementInProgress",
        worth_signal::facade::branch::SignalBranchRetirementDenial::RetiredBranch { .. } => "RetiredBranch",
        worth_signal::facade::branch::SignalBranchRetirementDenial::SharedAdmittedBasis { .. } => "SharedAdmittedBasis",
        other => panic!("unexpected retirement denial: {other:?}"),
    }
}
pub(crate) fn advance(
    mutation: &crate::signal_owner_services::ordinary_context::adversarial::world::MutationPort,
    basis: &AdmittedSignalBranchBasis,
) -> Result<(), &'static str> {
    mutation
        .advance_exact(
            worth_execution::ExecutionRequest::serial(
                &crate::signal_owner_services::ordinary_context::execution_custody::operational_serial_request(),
            ),
            basis,
            &mut (),
            &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .map(|_| ())
        .map_err(advance_denial)
}
pub(crate) fn capture(
    mutation: &crate::signal_owner_services::ordinary_context::adversarial::world::MutationPort,
    basis: &AdmittedSignalBranchBasis,
) -> Result<(AdmittedSignalBranchSnapshot, AdmittedSignalBranchBasis), &'static str> {
    mutation
        .capture_exact(basis, &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token())
        .map(|outcome| outcome.into_parts())
        .map_err(capture_denial)
}
pub(crate) fn restore(
    mutation: &crate::signal_owner_services::ordinary_context::adversarial::world::MutationPort,
    basis: &AdmittedSignalBranchBasis,
    snapshot: &AdmittedSignalBranchSnapshot,
) -> Result<(), &'static str> {
    mutation
        .restore_exact(
            basis,
            snapshot,
            &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
        )
        .map(|_| ())
        .map_err(restore_denial)
}
pub(crate) fn retirement_plan(
    lifecycle: &crate::signal_owner_services::ordinary_context::adversarial::world::LifecyclePort,
    basis: AdmittedSignalBranchBasis,
) -> Result<worth_signal::facade::branch::PlannedSignalBranchRetirement, &'static str> {
    match lifecycle.plan_retirement_exact(basis, worth_signal::facade::branch::SignalBranchRetirementReason::Superseded) {
        worth_proof::TransitionOutcome::Success(plan) => Ok(plan),
        worth_proof::TransitionOutcome::Denied(denial) => Err(retirement_denial(denial)),
    }
}

pub(crate) fn retire(
    lifecycle: &crate::signal_owner_services::ordinary_context::adversarial::world::LifecyclePort,
    plan: worth_signal::facade::branch::PlannedSignalBranchRetirement,
) -> Result<(), &'static str> {
    match lifecycle.retire_exact(plan, &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token()) {
        worth_proof::TransitionOutcome::Success(_) => Ok(()),
        worth_proof::TransitionOutcome::Denied(denial) => Err(retirement_denial(denial)),
    }
}
pub(crate) fn run_pair<L, R>(
    runtime: &Option<crate::signal_owner_services::ordinary_context::adversarial::world::Runtime>,
    left: L,
    right: R,
    label: &str,
) where
    L: FnOnce() -> Result<(), &'static str> + Send,
    R: FnOnce() -> Result<(), &'static str> + Send,
{
    let owner_root = runtime
        .as_ref()
        .expect("the race contenders share a live owner root");
    let gate = std::sync::Arc::new(std::sync::Barrier::new(3));
    let (left_tx, left_rx) = std::sync::mpsc::sync_channel(1);
    let (right_tx, right_rx) = std::sync::mpsc::sync_channel(1);
    std::thread::scope(|scope| {
        let left_gate = std::sync::Arc::clone(&gate);
        scope.spawn(move || {
            left_gate.wait();
            let _ = left_tx.send(left());
        });
        let right_gate = std::sync::Arc::clone(&gate);
        scope.spawn(move || {
            right_gate.wait();
            let _ = right_tx.send(right());
        });
        gate.wait();
        let left_result = left_rx
            .recv_timeout(PROGRESS_BOUND)
            .expect("left race participant completes");
        let right_result = right_rx
            .recv_timeout(PROGRESS_BOUND)
            .expect("right race participant completes");
        assert_one_success(left_result, right_result, label);
    });
    owner_root
        .owner_operation_control_for_test()
        .expect("the owner root remains live through both race contenders");
}

pub(crate) fn prove_advance_is_effectful() {
    let world = crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
    advance(&world.mutation, &world.child_basis).expect("an uncontended advance succeeds");
}

pub(crate) fn prove_capture_and_restore_are_effectful() {
    let world = crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
    let (snapshot, basis) =
        capture(&world.mutation, &world.child_basis).expect("an uncontended capture succeeds");
    restore(&world.mutation, &basis, &snapshot).expect("an uncontended restore succeeds");
}

pub(crate) fn prove_retirement_is_effectful() {
    let AdversarialWorld {
        runtime,
        lifecycle,
        child_basis,
        ..
    } = crate::signal_owner_services::ordinary_context::adversarial::world::AdversarialWorld::new();
    let plan =
        retirement_plan(&lifecycle, child_basis).expect("an uncontended retirement plan succeeds");
    retire(&lifecycle, plan).expect("an uncontended retirement succeeds");
    drop(runtime);
}