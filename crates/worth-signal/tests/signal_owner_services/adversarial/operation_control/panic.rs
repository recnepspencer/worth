use std::panic::{catch_unwind, AssertUnwindSafe};

use worth_signal::facade::branch::{
    SignalBranchBasisObservationDenial, SignalOwnerCancellationSource, SignalOwnerOperationBoundary,
};

use super::super::world::AdversarialWorld;

#[test]
fn transaction_callback_panic_quarantines_source_and_keeps_sibling_healthy() {
    let world = AdversarialWorld::new();
    let reference = world
        .basis
        .issue_managed_branch_reference(&world.root_basis)
        .expect("the rollback observation uses owner-issued reference custody");
    let fault = catch_unwind(AssertUnwindSafe(|| {
        let _ = world.mutation.advance_exact(
            &world.root_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| panic!("transaction callback failure"),
        );
    }));
    assert!(
        fault.is_err(),
        "the transaction callback panic must reach the caller"
    );
    assert!(matches!(
        world.basis.observe_current(&reference),
        Err(SignalBranchBasisObservationDenial::QuarantinedBranch { branch_id })
            if branch_id == world.root_basis.branch_id()
    ));
    world
        .mutation
        .advance_exact(
            &world.child_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("a transaction panic must not poison an unrelated branch");
}

#[test]
fn pre_effect_advance_panic_unwinds_without_poisoning_the_owner() {
    let world = AdversarialWorld::new();
    let control = world
        .runtime
        .as_ref()
        .expect("the strong root remains live")
        .owner_operation_control()
        .expect("the control handle comes from the sealed owner");
    control.inject_panic_once(SignalOwnerOperationBoundary::BeforeCanonicalMovement);
    let mutation = world.mutation.clone();
    let basis = world.root_basis.clone();
    let fault = catch_unwind(AssertUnwindSafe(|| {
        let _ = mutation.advance_exact(
            &basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        );
    }));
    assert!(
        fault.is_err(),
        "the injected owner fault must reach the caller"
    );
    assert!(matches!(
        world.lifecycle.owner_lifecycle_observation(),
        worth_signal::facade::branch::SignalOwnerLifecycleObservation::Open
    ));
    world
        .mutation
        .advance_exact(
            &world.child_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("an unrelated branch remains usable after unwind");
}

#[test]
fn post_effect_advance_panic_preserves_performed_truth_and_sibling_progress() {
    let world = AdversarialWorld::new();
    let reference = world
        .basis
        .issue_managed_branch_reference(&world.root_basis)
        .expect("the observation starts from an owner-issued reference");
    let control = world
        .runtime
        .as_ref()
        .expect("the strong root remains live")
        .owner_operation_control()
        .expect("the control handle comes from the sealed owner");
    control.inject_panic_once(SignalOwnerOperationBoundary::OutcomeConstruction);
    let fault = catch_unwind(AssertUnwindSafe(|| {
        let _ = world.mutation.advance_exact(
            &world.root_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        );
    }));
    assert!(
        fault.is_err(),
        "the post-effect fault must reach the caller"
    );
    let observed = world
        .basis
        .observe_current(&reference)
        .expect("a post-effect outcome fault leaves canonical truth healthy");
    assert_eq!(
        observed.observation().generation().get(),
        world.root_basis.observation().generation().get() + 1,
        "the performed movement remains observable after outcome construction unwinds"
    );
    world
        .mutation
        .advance_exact(
            &world.child_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("the faulted cell does not poison an unrelated sibling");
}

#[test]
fn fork_post_capture_fault_preserves_performed_child_and_source_health() {
    for boundary in [
        SignalOwnerOperationBoundary::ForkDestinationInstallation,
        SignalOwnerOperationBoundary::OutcomeConstruction,
    ] {
        let world = AdversarialWorld::new();
        let runtime = world
            .runtime
            .as_ref()
            .expect("the strong root remains live");
        let control = runtime
            .owner_operation_control()
            .expect("the control handle comes from the sealed owner");
        let name = "performed-fork";
        control.inject_panic_once(boundary);
        let fault = catch_unwind(AssertUnwindSafe(|| {
            let _ = world.mutation.fork_exact(
                worth_signal::facade::branch::validate_signal_branch_name(name)
                    .expect("the fork name is valid"),
                &world.root_basis,
                &SignalOwnerCancellationSource::new().token(),
            );
        }));
        assert!(fault.is_err(), "the post-capture fault reaches the caller");

        let performed = runtime
            .known_branches()
            .into_iter()
            .find(|branch| branch.name == name)
            .expect("post-linearization panic preserves the canonical child");
        assert_eq!(
            performed.parent_branch_id,
            Some(world.root_basis.branch_id())
        );
        let performed_basis = runtime
            .observe_signal_branch_basis(performed.clone())
            .expect("the performed child is recoverable as exact owner authority");
        assert_eq!(performed_basis.branch_id(), performed.id);
        world
            .mutation
            .advance_exact(
                &world.root_basis,
                &mut (),
                &SignalOwnerCancellationSource::new().token(),
                |_| Ok(()),
            )
            .expect("post-capture destination work releases the healthy source");
    }
}

#[test]
fn fork_source_capture_fault_quarantines_only_source_and_releases_destination() {
    let world = AdversarialWorld::new();
    let source_reference = world
        .basis
        .issue_managed_branch_reference(&world.root_basis)
        .expect("the source observation uses owner-issued reference custody");
    let control = world
        .runtime
        .as_ref()
        .expect("the strong root remains live")
        .owner_operation_control()
        .expect("the sealed owner issues operation control");
    control.inject_panic_once(SignalOwnerOperationBoundary::ForkSourceCapture);
    let fault = catch_unwind(AssertUnwindSafe(|| {
        let _ = world.mutation.fork_exact(
            worth_signal::facade::branch::validate_signal_branch_name("faulted-source")
                .expect("the fault fixture uses a valid identity"),
            &world.root_basis,
            &SignalOwnerCancellationSource::new().token(),
        );
    }));
    assert!(
        fault.is_err(),
        "the source-capture fault must reach the caller"
    );
    assert!(matches!(
        world.basis.observe_current(&source_reference),
        Err(SignalBranchBasisObservationDenial::QuarantinedBranch { branch_id })
            if branch_id == world.root_basis.branch_id()
    ));
    let child_reference = world
        .basis
        .issue_managed_branch_reference(&world.child_basis)
        .expect("the healthy sibling can issue a refresh reference");
    world
        .mutation
        .advance_exact(
            &world.child_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("source quarantine must not poison the sibling");
    let current_child = world
        .basis
        .observe_current(&child_reference)
        .expect("the retry reacquires the sibling's current basis");
    let retry = world
        .mutation
        .fork_exact(
            worth_signal::facade::branch::validate_signal_branch_name("healthy-after-source")
                .expect("the retry uses a valid identity"),
            &current_child,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("the failed destination reservation must be reusable");
    assert_eq!(retry.created_branch().name, "healthy-after-source");
}

#[test]
fn snapshot_and_restore_faults_do_not_poison_an_unrelated_branch() {
    let snapshot_world = AdversarialWorld::new();
    let control = snapshot_world
        .runtime
        .as_ref()
        .expect("the strong root remains live")
        .owner_operation_control()
        .expect("the control handle comes from the sealed owner");
    control.inject_panic_once(SignalOwnerOperationBoundary::OutcomeConstruction);
    let snapshot_fault = catch_unwind(AssertUnwindSafe(|| {
        let _ = snapshot_world.mutation.capture_exact(
            &snapshot_world.child_basis,
            &SignalOwnerCancellationSource::new().token(),
        );
    }));
    assert!(
        snapshot_fault.is_err(),
        "snapshot output construction faults"
    );
    snapshot_world
        .mutation
        .advance_exact(
            &snapshot_world.root_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("snapshot outcome unwind does not poison an unrelated branch");

    let restore_world = AdversarialWorld::new();
    let (snapshot, current) = restore_world
        .mutation
        .capture_exact(
            &restore_world.child_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("restore starts with a real snapshot")
        .into_parts();
    let control = restore_world
        .runtime
        .as_ref()
        .expect("the strong root remains live")
        .owner_operation_control()
        .expect("the control handle comes from the sealed owner");
    control.inject_panic_once(SignalOwnerOperationBoundary::OutcomeConstruction);
    let restore_fault = catch_unwind(AssertUnwindSafe(|| {
        let _ = restore_world.mutation.restore_exact(
            &current,
            &snapshot,
            &SignalOwnerCancellationSource::new().token(),
        );
    }));
    assert!(restore_fault.is_err(), "restore output construction faults");
    restore_world
        .mutation
        .advance_exact(
            &restore_world.root_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("restore outcome unwind does not poison an unrelated branch");
}
