use std::sync::atomic::{AtomicBool, Ordering};

use crate::branch::RelationalOwnerLifecycleObservation;
use crate::runtime::{
    RelationalRuntime, RelationalRuntimeAdmissionHoldDenial, RelationalRuntimeAdmissionPosture,
};
use crate::tests::support::{create_entity, runtime_with_test_schema};

#[test]
fn try_hold_with_active_admission_refuses_and_admission_still_works() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "seal-held-admission-anchor");
    let services = runtime.owner_component_services();
    let held = runtime
        .owner_binding()
        .admit()
        .expect("an open owner admits");

    assert_eq!(
        runtime.try_hold_admission().into_result().unwrap_err(),
        RelationalRuntimeAdmissionHoldDenial::AdmissionsActive
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

    runtime.try_hold_admission().unwrap().seal();
    assert_eq!(
        runtime.try_hold_admission().into_result().unwrap_err(),
        RelationalRuntimeAdmissionHoldDenial::AlreadySealed
    );
}

#[test]
fn refused_hold_never_denies_a_concurrent_admission() {
    const ADMISSIONS: usize = 50_000;
    let mut runtime = runtime_with_test_schema();
    let admitting = runtime.owner_binding();
    let held = admitting.admit().expect("an open owner admits");
    assert_eq!(
        runtime.try_hold_admission().into_result().unwrap_err(),
        RelationalRuntimeAdmissionHoldDenial::AdmissionsActive
    );
    assert_eq!(
        admitting.admission_posture(),
        RelationalRuntimeAdmissionPosture::Open
    );
    let admissions_finished = std::sync::Arc::new(AtomicBool::new(false));
    let finished = admissions_finished.clone();
    let concurrent_binding = admitting.clone();
    let admitter = std::thread::spawn(move || {
        let denied = (0..ADMISSIONS)
            .filter(|_| concurrent_binding.admit().is_none())
            .count();
        finished.store(true, Ordering::SeqCst);
        denied
    });
    while !admissions_finished.load(Ordering::SeqCst) {
        assert_eq!(
            runtime.try_hold_admission().into_result().unwrap_err(),
            RelationalRuntimeAdmissionHoldDenial::AdmissionsActive
        );
        assert_eq!(
            admitting.admission_posture(),
            RelationalRuntimeAdmissionPosture::Open
        );
    }
    assert_eq!(
        admitter.join().unwrap(),
        0,
        "a refused hold never denies admission"
    );

    drop(held);
    runtime.try_hold_admission().unwrap().seal();
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

    runtime.try_hold_admission().unwrap().seal();

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
fn try_hold_on_an_admitted_handle_is_refused_as_not_owner() {
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
        admitted.try_hold_admission().into_result().unwrap_err(),
        RelationalRuntimeAdmissionHoldDenial::NotOwner
    );
    assert_eq!(
        runtime.try_hold_admission().into_result().unwrap_err(),
        RelationalRuntimeAdmissionHoldDenial::AdmissionsActive,
        "the admitted handle is itself an operation in flight"
    );
    drop(admitted);

    runtime.try_hold_admission().unwrap().seal();
}

#[test]
fn repeated_successful_holds_and_releases_never_deny_concurrent_admitters() {
    const ROUNDS: usize = 64;
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "repeated-held-admission");
    let identity = runtime.main_branch_identity();
    let services = runtime.owner_component_services();
    let binding = runtime.owner_binding();
    let (events, observed) = std::sync::mpsc::channel();
    let mut starts = Vec::new();
    let mut admitters = Vec::new();
    for _ in 0..2 {
        let (start, rounds) = std::sync::mpsc::channel();
        starts.push(start);
        let port = services.basis_port();
        let identity = identity.clone();
        let completed = events.clone();
        admitters.push(std::thread::spawn(move || {
            let mut denied = 0;
            while rounds.recv().is_ok() {
                denied += usize::from(port.observe_branch(&identity).is_err());
                completed.send(false).unwrap();
            }
            denied
        }));
    }
    for _ in 0..ROUNDS {
        let hold = runtime.try_hold_admission().unwrap();
        for start in &starts {
            binding.install_test_hold_wait_ack(events.clone());
            start.send(()).unwrap();
            assert!(
                observed.recv().unwrap(),
                "each admission waits before completing"
            );
        }
        drop(hold);
        assert_eq!(
            binding.admission_posture(),
            RelationalRuntimeAdmissionPosture::Open
        );
        for _ in &starts {
            assert!(
                !observed.recv().unwrap(),
                "both admissions complete after release"
            );
        }
    }
    drop(starts);
    for admitter in admitters {
        assert_eq!(
            admitter.join().unwrap(),
            0,
            "temporary holds never deny admission"
        );
    }
    runtime.try_hold_admission().unwrap().seal();
}
