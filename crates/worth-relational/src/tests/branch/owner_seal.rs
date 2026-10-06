use std::sync::atomic::{AtomicBool, Ordering};

use crate::branch::RelationalOwnerLifecycleObservation;
use crate::runtime::{
    RelationalRuntime, RelationalRuntimeAdmissionPosture, RelationalRuntimeSealDenial,
    RelationalRuntimeSealOutcome,
};
use crate::tests::support::{create_entity, runtime_with_test_schema};

#[test]
fn try_seal_with_held_admission_refuses_and_admission_still_works() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "seal-held-admission-anchor");
    let services = runtime.owner_component_services();
    let held = runtime
        .owner_binding()
        .admit()
        .expect("an open owner admits");

    assert_eq!(
        runtime.try_seal(),
        Err(RelationalRuntimeSealDenial::AdmissionsActive)
    );
    assert_eq!(
        runtime.owner_binding().admission_posture(),
        RelationalRuntimeAdmissionPosture::Open
    );
    services
        .basis_port()
        .observe_branch(&runtime.main_branch_identity())
        .expect("a refused seal never stopped admission");
    drop(held);
    services
        .basis_port()
        .observe_branch(&runtime.main_branch_identity())
        .expect("admission still works once the held admission returns");

    assert_eq!(runtime.try_seal(), Ok(RelationalRuntimeSealOutcome::Sealed));
    assert_eq!(
        runtime.try_seal(),
        Ok(RelationalRuntimeSealOutcome::AlreadySealed)
    );
}

#[test]
fn refused_seal_never_denies_a_concurrent_admission() {
    const ADMISSIONS: usize = 50_000;
    let mut runtime = runtime_with_test_schema();
    let admitting = runtime.owner_binding();
    let held = admitting.admit().expect("an open owner admits");
    let admissions_finished = AtomicBool::new(false);

    std::thread::scope(|scope| {
        let admitter = scope.spawn(|| {
            let denied = (0..ADMISSIONS)
                .filter(|_| admitting.admit().is_none())
                .count();
            admissions_finished.store(true, Ordering::SeqCst);
            denied
        });
        let mut refusals = 0_u64;
        while !admissions_finished.load(Ordering::SeqCst) {
            assert_eq!(
                runtime.try_seal(),
                Err(RelationalRuntimeSealDenial::AdmissionsActive)
            );
            refusals += 1;
        }
        assert!(refusals > 0, "the seal was attempted while admitting");
        assert_eq!(
            admitter.join().expect("the admitting thread finishes"),
            0,
            "a refused seal changes nothing, so no concurrent admission is denied"
        );
    });

    drop(held);
    assert_eq!(runtime.try_seal(), Ok(RelationalRuntimeSealOutcome::Sealed));
    assert!(admitting.admit().is_none());
}

#[test]
fn sealed_owner_is_observed_closed_while_its_state_is_alive() {
    let mut runtime = runtime_with_test_schema();
    let lifecycle = runtime.owner_component_services().lifecycle_port();
    assert_eq!(
        lifecycle.owner_lifecycle_observation(),
        RelationalOwnerLifecycleObservation::Open
    );

    assert_eq!(runtime.try_seal(), Ok(RelationalRuntimeSealOutcome::Sealed));

    assert_eq!(
        lifecycle.owner_lifecycle_observation(),
        RelationalOwnerLifecycleObservation::Closed,
        "a sealed owner is closed, not closing, though its state is still alive"
    );
    assert_eq!(
        runtime.owner_binding().admission_posture(),
        RelationalRuntimeAdmissionPosture::Closed
    );
    drop(runtime);
    assert_eq!(
        lifecycle.owner_lifecycle_observation(),
        RelationalOwnerLifecycleObservation::Closed
    );
}

#[test]
fn try_seal_on_an_admitted_handle_is_refused_as_not_owner() {
    let mut runtime = runtime_with_test_schema();
    let state = runtime
        .state_binding()
        .upgrade()
        .expect("the owner holds its state");
    let operation = runtime
        .owner_binding()
        .admit()
        .expect("an open owner admits");
    let mut admitted = RelationalRuntime::admitted(state, operation);

    assert_eq!(
        admitted.try_seal(),
        Err(RelationalRuntimeSealDenial::NotOwner)
    );
    assert_eq!(
        runtime.try_seal(),
        Err(RelationalRuntimeSealDenial::AdmissionsActive),
        "the admitted handle is itself an operation in flight"
    );
    drop(admitted);

    assert_eq!(runtime.try_seal(), Ok(RelationalRuntimeSealOutcome::Sealed));
}
