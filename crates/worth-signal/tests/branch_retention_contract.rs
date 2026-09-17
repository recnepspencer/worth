use worth_proof::TransitionOutcome;
use worth_signal::facade::branch::{
    SignalBranchRetentionAcquisitionDenial, SignalBranchRetentionReleaseOutcome,
};
use worth_signal::facade::runtime::SignalBranchRetirementDenial;
use worth_signal::facade::{SignalBranchRetirementReason, SignalGraph, SignalRuntime};

fn runtime() -> SignalRuntime<(), (), (), (), ()> {
    SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build()
}

#[test]
fn retention_capacity_denies_before_unbounded_growth() {
    const INDEPENDENT_SAFETY_CEILING: usize = 4_096;

    let runtime = runtime();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("owner observation should succeed");
    let mut leases = Vec::new();
    let mut reported_capacity = None;
    for _ in 0..=INDEPENDENT_SAFETY_CEILING {
        match runtime.retain_signal_component_basis(&basis) {
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
    assert_eq!(reported_capacity, Some(INDEPENDENT_SAFETY_CEILING));
    assert_eq!(leases.len(), INDEPENDENT_SAFETY_CEILING - 1);

    let descriptor = basis.descriptor().clone();
    let readmitted = runtime
        .readmit_signal_branch_basis(descriptor.clone())
        .expect("canonical readmission reuses the existing admission at capacity");
    assert_eq!(readmitted.descriptor(), &descriptor);
    let observed = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("canonical observation reuses the existing admission at capacity");
    assert_eq!(observed.descriptor(), &descriptor);
    assert!(matches!(
        runtime.retain_signal_component_basis(&basis),
        Err(SignalBranchRetentionAcquisitionDenial::CapacityExhausted {
            maximum_active_leases: INDEPENDENT_SAFETY_CEILING,
        })
    ));
    drop(readmitted);
    drop(observed);

    let released = leases.pop().expect("capacity fixture should retain leases");
    assert!(matches!(
        runtime.release_signal_component_basis(released),
        SignalBranchRetentionReleaseOutcome::Released(_)
    ));
    let reacquired = runtime
        .retain_signal_component_basis(&basis)
        .expect("one released slot is sufficient for one external obligation");
    assert!(matches!(
        runtime.release_signal_component_basis(reacquired),
        SignalBranchRetentionReleaseOutcome::Released(_)
    ));

    for lease in leases {
        assert!(matches!(
            runtime.release_signal_component_basis(lease),
            SignalBranchRetentionReleaseOutcome::Released(_)
        ));
    }
    let reacquired_after_release = runtime
        .retain_signal_component_basis(&basis)
        .expect("released capacity should be reusable");
    assert!(matches!(
        runtime.release_signal_component_basis(reacquired_after_release),
        SignalBranchRetentionReleaseOutcome::Released(_)
    ));
}

#[test]
fn admitted_snapshot_blocks_retirement_until_every_clone_is_released() {
    let mut runtime = runtime();
    let main_basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("owner observation should succeed");
    let (branch, basis) = runtime
        .fork_signal_branch("retained-snapshot", &main_basis)
        .expect("owner fork should succeed")
        .into_parts();
    let (snapshot, captured_basis) = runtime
        .capture_signal_branch_snapshot(&basis)
        .expect("owner capture should succeed")
        .into_parts();
    let descriptor = captured_basis.descriptor().clone();
    let snapshot_clone = snapshot.clone();
    drop(basis);
    drop(captured_basis);
    drop(snapshot);

    let retirement_basis = runtime
        .readmit_signal_branch_basis(descriptor.clone())
        .expect("snapshot retention must leave capacity for retirement admission");
    let denied = runtime.plan_signal_branch_retirement(
        branch.clone(),
        retirement_basis,
        SignalBranchRetirementReason::Superseded,
    );
    assert!(matches!(
        denied,
        TransitionOutcome::Denied(SignalBranchRetirementDenial::RetainedAdmittedBasis {
            active_leases: 2,
            ..
        })
    ));

    drop(snapshot_clone);
    let retirement_basis = runtime
        .readmit_signal_branch_basis(descriptor)
        .expect("dropping the final snapshot holder must release retirement retention");
    let plan = runtime.plan_signal_branch_retirement(
        branch,
        retirement_basis,
        SignalBranchRetirementReason::Superseded,
    );
    let plan = match plan {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("released snapshot should permit retirement: {other:?}"),
    };
    assert!(matches!(
        runtime.retire_signal_branch(plan),
        TransitionOutcome::Success(_)
    ));
}

#[test]
fn reconstructed_snapshot_clone_retains_capacity_until_final_release() {
    const MAXIMUM_ACTIVE_LEASES: usize = 4_096;

    let mut source = runtime();
    let source_basis = source
        .observe_signal_branch_basis(source.current_branch())
        .expect("source owner observation should succeed");
    let portable_snapshot = source
        .capture_signal_branch_snapshot(&source_basis)
        .expect("source capture should succeed")
        .into_parts()
        .0
        .into_snapshot();

    let mut target = runtime();
    let pristine_basis = target
        .observe_signal_branch_basis(target.current_branch())
        .expect("target owner observation should succeed");
    let (admitted_snapshot, reconstructed_basis) = target
        .reconstruct_signal_branch_snapshot(&pristine_basis, &portable_snapshot)
        .expect("portable snapshot should reconstruct into a pristine owner")
        .into_parts();
    drop(pristine_basis);
    let snapshot_clone = admitted_snapshot.clone();

    let mut leases = Vec::new();
    loop {
        match target.retain_signal_component_basis(&reconstructed_basis) {
            Ok(lease) => leases.push(lease),
            Err(SignalBranchRetentionAcquisitionDenial::CapacityExhausted { .. }) => break,
            Err(other) => panic!("unexpected retention denial: {other:?}"),
        }
    }
    assert_eq!(leases.len(), MAXIMUM_ACTIVE_LEASES - 2);

    drop(admitted_snapshot);
    assert!(matches!(
        target.retain_signal_component_basis(&reconstructed_basis),
        Err(SignalBranchRetentionAcquisitionDenial::CapacityExhausted { .. })
    ));
    drop(snapshot_clone);
    let released_slot = target
        .retain_signal_component_basis(&reconstructed_basis)
        .expect("dropping the final reconstructed snapshot clone must release its lease");
    assert!(matches!(
        target.release_signal_component_basis(released_slot),
        SignalBranchRetentionReleaseOutcome::Released(_)
    ));
    for lease in leases {
        assert!(matches!(
            target.release_signal_component_basis(lease),
            SignalBranchRetentionReleaseOutcome::Released(_)
        ));
    }
}
