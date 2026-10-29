#[path = "../fixtures/retention_support/mod.rs"]
// Courts for Signal exact-target retention and lease terminality.
//
// Every court here fixes one obligation of the Phase 4 contract: an external
// component obligation names an exact immutable target, is independent of how
// current that target is, and reaches exactly one terminal state whether it is
// released explicitly, dropped, or outlives its owner.
pub(crate) mod retention_support;

use retention_support::{fork_with_historical_target, runtime};
use worth_proof::TransitionOutcome;
use worth_signal::facade::branch::{
    SignalBranchBasisReadmissionDenial, SignalBranchRetainedReadmissionDenial,
    SignalBranchRetentionAcquisitionDenial, SignalBranchRetentionOwnerPosture,
    SignalBranchRetentionReleaseDenial, SignalBranchRetentionReleaseOutcome,
    SignalBranchRetentionTerminalOutcome,
};
use worth_signal::facade::runtime::SignalBranchRetirementDenial;
use worth_signal::facade::SignalBranchRetirementReason;

#[test]
fn an_exact_obligation_pins_a_real_historical_admitted_target() {
    let mut runtime = runtime();
    let (_branch, historical, current) = fork_with_historical_target(&mut runtime);

    // Ordinary readmission speaks about the branch's current state, so the
    // historical descriptor is legitimately refused there.
    assert!(matches!(
        runtime.readmit_signal_branch_basis(historical.descriptor().clone()),
        Err(SignalBranchBasisReadmissionDenial::ReferenceMismatch { .. })
    ));

    // The exact obligation is a different question, and it is answered `yes`.
    let lease = runtime
        .retain_signal_component_basis(&historical)
        .expect("a stored historical target must remain retainable");
    assert_eq!(
        lease.retained_target(),
        historical
            .observation()
            .target()
            .as_basis()
            .expect("an owner observation carries a basis target"),
    );
    assert_ne!(
        lease.retained_target(),
        current
            .observation()
            .target()
            .as_basis()
            .expect("an owner observation carries a basis target"),
        "the obligation must pin the historical target, not the current one"
    );
    assert_eq!(
        lease.owner_posture(),
        SignalBranchRetentionOwnerPosture::Live
    );
}

#[test]
fn a_live_obligation_readmits_its_exact_retained_target() {
    let foreign = runtime();
    let mut runtime = runtime();
    let (_branch, historical, _current) = fork_with_historical_target(&mut runtime);
    let descriptor = historical.descriptor().clone();
    let lease = runtime
        .retain_signal_component_basis(&historical)
        .expect("a stored historical target must remain retainable");
    drop(historical);

    let readmitted = runtime
        .readmit_retained_signal_branch_basis(descriptor.clone(), &lease)
        .expect("a live obligation must readmit the exact target it retains");
    assert_eq!(readmitted.descriptor(), &descriptor);
    assert_eq!(readmitted.observation(), descriptor.observation());

    // The obligation authorizes exactly one target, not any target.
    let other = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("owner observation should succeed");
    assert!(matches!(
        runtime.readmit_retained_signal_branch_basis(other.descriptor().clone(), &lease),
        Err(SignalBranchRetainedReadmissionDenial::DescriptorMismatch)
    ));

    // A foreign owner cannot spend another owner's obligation.
    assert!(matches!(
        foreign.readmit_retained_signal_branch_basis(descriptor, &lease),
        Err(SignalBranchRetainedReadmissionDenial::ForeignRetention)
    ));
}

#[test]
fn explicit_release_returns_governed_exact_target_evidence() {
    let mut runtime = runtime();
    let (_branch, historical, _current) = fork_with_historical_target(&mut runtime);
    let retained_target = historical
        .observation()
        .target()
        .as_basis()
        .expect("an owner observation carries a basis target")
        .clone();
    let first = runtime
        .retain_signal_component_basis(&historical)
        .expect("a stored historical target must remain retainable");
    let second = runtime
        .retain_signal_component_basis(&historical)
        .expect("the same exact target may carry more than one obligation");

    let receipt = match runtime.release_signal_component_basis(first) {
        SignalBranchRetentionReleaseOutcome::Released(receipt) => receipt,
        other => panic!("an owner release must succeed: {other:?}"),
    };
    assert_eq!(receipt.released_target(), &retained_target);
    assert_eq!(receipt.branch_id(), historical.branch_id());
    assert_eq!(
        receipt.outcome(),
        SignalBranchRetentionTerminalOutcome::Released
    );
    assert_eq!(receipt.remaining_target_leases(), 1);
    assert_eq!(receipt.remaining_branch_leases(), 1);

    let receipt = match runtime.release_signal_component_basis(second) {
        SignalBranchRetentionReleaseOutcome::Released(receipt) => receipt,
        other => panic!("an owner release must succeed: {other:?}"),
    };
    assert_eq!(receipt.remaining_target_leases(), 0);
    assert_eq!(receipt.remaining_branch_leases(), 0);

    let counts = runtime.signal_component_retention_terminal_counts();
    assert_eq!(counts.explicit_releases(), 2);
    assert_eq!(counts.dropped_releases(), 0);
    assert_eq!(counts.owner_loss_releases(), 0);
    assert_eq!(counts.unknown_lease_defenses(), 0);
}

#[test]
fn dropping_an_obligation_is_the_same_terminal_release() {
    let mut runtime = runtime();
    let (branch, historical, current) = fork_with_historical_target(&mut runtime);
    let lease = runtime
        .retain_signal_component_basis(&historical)
        .expect("a stored historical target must remain retainable");
    drop(historical);

    let denied = runtime.plan_signal_branch_retirement(
        branch.clone(),
        current,
        SignalBranchRetirementReason::Superseded,
    );
    assert!(matches!(
        denied,
        TransitionOutcome::Denied(SignalBranchRetirementDenial::RetainedComponentBasis {
            active_leases: 1,
            ..
        })
    ));

    drop(lease);

    let counts = runtime.signal_component_retention_terminal_counts();
    assert_eq!(counts.explicit_releases(), 0);
    assert_eq!(counts.dropped_releases(), 1);
    assert_eq!(counts.terminal_releases(), 1);
    assert_eq!(counts.unknown_lease_defenses(), 0);

    // The dropped obligation discharged exactly the accounting an explicit
    // release would have, so retirement is no longer blocked.
    let basis = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("owner observation should succeed");
    assert!(matches!(
        runtime.plan_signal_branch_retirement(
            branch,
            basis,
            SignalBranchRetirementReason::Superseded,
        ),
        TransitionOutcome::Success(_)
    ));
}

#[test]
fn a_foreign_release_hands_the_live_obligation_back() {
    let mut owner = runtime();
    let foreign = runtime();
    let (_branch, historical, _current) = fork_with_historical_target(&mut owner);
    let descriptor = historical.descriptor().clone();
    let lease = owner
        .retain_signal_component_basis(&historical)
        .expect("a stored historical target must remain retainable");

    let lease = match foreign.release_signal_component_basis(lease) {
        SignalBranchRetentionReleaseOutcome::Denied {
            lease,
            denial: SignalBranchRetentionReleaseDenial::ForeignRuntime,
        } => lease,
        other => panic!("a foreign release must preserve the obligation: {other:?}"),
    };
    assert_eq!(
        foreign
            .signal_component_retention_terminal_counts()
            .terminal_releases(),
        0,
        "a refused release must not be accounted anywhere"
    );

    // The returned obligation is still live and still spendable at its owner.
    owner
        .readmit_retained_signal_branch_basis(descriptor, &lease)
        .expect("a refused release must leave the obligation usable");
    assert!(matches!(
        owner.release_signal_component_basis(lease),
        SignalBranchRetentionReleaseOutcome::Released(_)
    ));
    assert_eq!(
        owner
            .signal_component_retention_terminal_counts()
            .explicit_releases(),
        1
    );
}

#[test]
fn an_obligation_outlives_its_owner_and_records_owner_loss() {
    let mut owner = runtime();
    let (_branch, historical, _current) = fork_with_historical_target(&mut owner);
    let descriptor = historical.descriptor().clone();
    let released = owner
        .retain_signal_component_basis(&historical)
        .expect("a stored historical target must remain retainable");
    let witness = owner
        .retain_signal_component_basis(&historical)
        .expect("a second obligation over the same exact target is legitimate");
    assert_eq!(
        released.owner_posture(),
        SignalBranchRetentionOwnerPosture::Live
    );

    drop(historical);
    drop(owner);

    assert_eq!(
        released.owner_posture(),
        SignalBranchRetentionOwnerPosture::Lost
    );
    assert_eq!(released.descriptor(), &descriptor);
    let receipt = released.release();
    assert_eq!(
        receipt.released_target(),
        descriptor.observation().target().as_basis().unwrap()
    );
    assert_eq!(
        receipt.outcome(),
        SignalBranchRetentionTerminalOutcome::OwnerUnavailable,
        "owner loss must be recorded distinctly from an ordinary release"
    );
    assert_eq!(receipt.remaining_target_leases(), 1);

    // The ledger survives the owner, so the loss stays observable.
    let counts = witness.owner_terminal_counts();
    assert_eq!(counts.explicit_releases(), 1);
    assert_eq!(counts.owner_loss_releases(), 1);
    assert_eq!(counts.unknown_lease_defenses(), 0);

    drop(witness);
}

#[test]
fn released_and_dropped_obligations_both_recover_capacity() {
    const MAXIMUM_ACTIVE_LEASES: usize = 4_096;

    let mut runtime = runtime();
    let (_branch, historical, _current) = fork_with_historical_target(&mut runtime);
    let mut leases = Vec::new();
    let mut reported_capacity = None;
    for _ in 0..=MAXIMUM_ACTIVE_LEASES {
        match runtime.retain_signal_component_basis(&historical) {
            Ok(lease) => leases.push(lease),
            Err(SignalBranchRetentionAcquisitionDenial::CapacityExhausted {
                maximum_active_leases,
            }) => {
                reported_capacity = Some(maximum_active_leases);
                break;
            }
            Err(other) => panic!("unexpected retention denial: {other:?}"),
        }
    }
    assert_eq!(reported_capacity, Some(MAXIMUM_ACTIVE_LEASES));

    // Dropping recovers exactly one slot, and so does releasing.
    drop(leases.pop().expect("capacity fixture retains obligations"));
    let reacquired = runtime
        .retain_signal_component_basis(&historical)
        .expect("a dropped obligation must return its capacity slot");
    assert!(matches!(
        runtime.release_signal_component_basis(reacquired),
        SignalBranchRetentionReleaseOutcome::Released(_)
    ));
    let reacquired = runtime
        .retain_signal_component_basis(&historical)
        .expect("a released obligation must return its capacity slot");
    drop(reacquired);

    let held = leases.len() as u64;
    drop(leases);
    let counts = runtime.signal_component_retention_terminal_counts();
    assert_eq!(counts.explicit_releases(), 1);
    assert_eq!(counts.dropped_releases(), held + 2);
    assert_eq!(counts.unknown_lease_defenses(), 0);
    runtime
        .retain_signal_component_basis(&historical)
        .expect("bulk release must restore the whole obligation budget");
}

#[test]
fn deletion_waits_for_the_exact_obligation_and_then_reclaims_it() {
    let mut runtime = runtime();
    let (branch, historical, current) = fork_with_historical_target(&mut runtime);
    let historical_descriptor = historical.descriptor().clone();
    let lease = runtime
        .retain_signal_component_basis(&historical)
        .expect("a stored historical target must remain retainable");
    drop(historical);

    let denied = runtime.plan_signal_branch_retirement(
        branch.clone(),
        current,
        SignalBranchRetirementReason::Superseded,
    );
    assert!(matches!(
        denied,
        TransitionOutcome::Denied(SignalBranchRetirementDenial::RetainedComponentBasis {
            active_leases: 1,
            ..
        })
    ));

    // While the obligation lives, the exact historical target is still there.
    runtime
        .readmit_retained_signal_branch_basis(historical_descriptor.clone(), &lease)
        .expect("a blocked deletion must leave the retained target available")
        .descriptor();

    let receipt = match runtime.release_signal_component_basis(lease) {
        SignalBranchRetentionReleaseOutcome::Released(receipt) => receipt,
        other => panic!("an owner release must succeed: {other:?}"),
    };
    assert_eq!(receipt.remaining_branch_leases(), 0);

    let retirement_basis = runtime
        .observe_signal_branch_basis(branch.clone())
        .expect("release should permit a fresh owner admission");
    let plan = match runtime.plan_signal_branch_retirement(
        branch,
        retirement_basis,
        SignalBranchRetirementReason::Superseded,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("a released obligation must permit retirement: {other:?}"),
    };
    let receipt = match runtime.retire_signal_branch(plan) {
        TransitionOutcome::Success(receipt) => receipt,
        other => panic!("planned retirement must execute: {other:?}"),
    };
    assert_eq!(
        receipt.reclaimed_snapshot_state_count(),
        2,
        "retirement must reclaim every exact target no obligation retained"
    );

    // The reclaimed exact target is no longer admissible by any route.
    assert!(matches!(
        runtime.readmit_signal_branch_basis(historical_descriptor),
        Err(SignalBranchBasisReadmissionDenial::RetiredBranch { .. })
    ));
}
