use std::sync::mpsc;
use std::thread;

use worth_proof::TransitionOutcome;
use worth_signal::facade::branch::{
    AdmittedSignalBranchBasis, ManagedSignalBranchReference,
    ManagedSignalBranchReferenceAdmissionDenial, SignalBranchBasisObservationDenial,
    SignalBranchObservation, SignalBranchRetirementDenial, SignalBranchRetirementReceipt,
    SignalOwnerCancellationSource, SignalOwnerOperationBoundary, SignalOwnerOperationControl,
};
use worth_signal::facade::history::RuntimeBranchId;

use super::super::world::{AdversarialWorld, BasisPort, LifecyclePort, PROGRESS_BOUND};
use super::races::{prove_retirement_is_effectful, retire, retirement_plan};

fn run_order(observe_first: bool) {
    let world = AdversarialWorld::new();
    let branch_id = world.child_basis.branch_id();
    let expected = world.child_basis.observation().clone();
    let control: SignalOwnerOperationControl = world
        .runtime
        .as_ref()
        .expect("the observer and retirement share a live owner root")
        .owner_operation_control()
        .expect("operation control is issued after sealing");
    let reference = world
        .basis
        .issue_managed_branch_reference(&world.child_basis)
        .expect("the observer uses an owner-issued reference");
    let plan = retirement_plan(&world.lifecycle, world.child_basis)
        .expect("the retirement contender is independently admissible");
    let observation_port = world.basis.clone();
    let lifecycle = world.lifecycle.clone();
    let observed_reference = reference.clone();
    let (observe_tx, observe_rx) = mpsc::sync_channel(1);
    let (retire_tx, retire_rx) = mpsc::sync_channel(1);

    let (observation, retirement) = thread::scope(|scope| {
        if observe_first {
            // Retirement checks admitted retention before its first controllable boundary, so
            // complete observation custody is acquired first to force this legal ordering.
            scope.spawn(move || {
                let _ = observe_tx.send(observation_port.observe_current(&observed_reference));
            });
            let observation = observe_rx
                .recv_timeout(PROGRESS_BOUND)
                .expect("the observer completes before retirement is admitted");
            scope.spawn(move || {
                let _ = retire_tx.send(
                    lifecycle.retire_exact(plan, &SignalOwnerCancellationSource::new().token()),
                );
            });
            let retirement = retire_rx
                .recv_timeout(PROGRESS_BOUND)
                .expect("the retirement resolves after observed custody is acquired");
            (observation, retirement)
        } else {
            // The observer reaches this boundary before acquiring its managed basis; pausing it
            // lets the retirement complete first and makes the terminal denial deterministic.
            let pause = control.arm_pause_once(SignalOwnerOperationBoundary::BranchRegistryLookup);
            scope.spawn(move || {
                let _ = observe_tx.send(observation_port.observe_current(&observed_reference));
            });
            assert!(pause.wait_until_reached(PROGRESS_BOUND));
            scope.spawn(move || {
                let _ = retire_tx.send(
                    lifecycle.retire_exact(plan, &SignalOwnerCancellationSource::new().token()),
                );
            });
            let retirement = retire_rx
                .recv_timeout(PROGRESS_BOUND)
                .expect("the unparked retirement resolves first");
            pause.release();
            let observation = observe_rx
                .recv_timeout(PROGRESS_BOUND)
                .expect("the parked observer resolves after release");
            (observation, retirement)
        }
    });

    if observe_first {
        assert_observe_first(
            &world.basis,
            &world.lifecycle,
            branch_id,
            &expected,
            &reference,
            observation,
            retirement,
        );
    } else {
        assert_retire_first(observation, retirement);
    }
}

fn assert_observe_first(
    basis: &BasisPort,
    lifecycle: &LifecyclePort,
    branch_id: RuntimeBranchId,
    expected: &SignalBranchObservation,
    reference: &ManagedSignalBranchReference,
    observation: Result<AdmittedSignalBranchBasis, SignalBranchBasisObservationDenial>,
    retirement: TransitionOutcome<SignalBranchRetirementReceipt, SignalBranchRetirementDenial>,
) {
    let observed = match observation {
        Ok(observed) => observed,
        Err(denial) => panic!("observe-first observer must win: {denial:?}"),
    };
    assert_eq!(observed.branch_id(), branch_id);
    assert_eq!(
        observed.observation().canonical_encoding(),
        expected.canonical_encoding(),
        "an observer winner must expose one complete canonical posture"
    );
    assert!(matches!(
        retirement,
        TransitionOutcome::Denied(SignalBranchRetirementDenial::RetainedAdmittedBasis {
            branch_id: denied_branch,
            active_leases: 2,
        }) if denied_branch == branch_id
    ));

    drop(observed);
    let retry_basis = basis
        .observe_current(reference)
        .expect("the child reference remains owner-issued after custody release");
    let retry_plan = retirement_plan(lifecycle, retry_basis)
        .expect("retirement is admissible after observed custody release");
    retire(lifecycle, retry_plan).expect("the uncontended retirement retry succeeds");
    match basis.observe_current(reference) {
        Err(SignalBranchBasisObservationDenial::ManagedReferenceDenied {
            denial: ManagedSignalBranchReferenceAdmissionDenial::BranchLifecycleEnded,
        }) => {}
        other => panic!("the retry must leave a terminal retired observation: {other:?}"),
    }
}

fn assert_retire_first(
    observation: Result<AdmittedSignalBranchBasis, SignalBranchBasisObservationDenial>,
    retirement: TransitionOutcome<SignalBranchRetirementReceipt, SignalBranchRetirementDenial>,
) {
    assert!(matches!(retirement, TransitionOutcome::Success(_)));
    match observation {
        Err(SignalBranchBasisObservationDenial::ManagedReferenceDenied {
            denial: ManagedSignalBranchReferenceAdmissionDenial::BranchLifecycleEnded,
        }) => {}
        other => panic!("retire-first observer must see terminal retirement: {other:?}"),
    }
}

#[test]
fn same_branch_observe_retire_returns_a_complete_pre_or_post_state() {
    prove_retirement_is_effectful();
    run_order(true);
    run_order(false);
}
