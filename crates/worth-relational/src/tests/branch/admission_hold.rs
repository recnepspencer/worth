use std::sync::mpsc;

use crate::branch::{RelationalBranchBasisDenial, RelationalOwnerLifecycleObservation};
use crate::tests::support::{create_entity, runtime_with_test_schema};

#[test]
fn admission_during_a_hold_waits_and_proceeds_after_release() {
    assert_waiting_admission_resolves(false);
}

#[test]
fn admission_during_a_hold_waits_and_is_owner_unavailable_after_seal() {
    assert_waiting_admission_resolves(true);
}

#[test]
fn all_waiters_proceed_after_a_hold_is_released() {
    assert_waiting_admissions_resolve(false, 4);
}

#[test]
fn all_waiters_are_owner_unavailable_after_a_hold_is_sealed() {
    assert_waiting_admissions_resolve(true, 4);
}

fn assert_waiting_admission_resolves(seal: bool) {
    assert_waiting_admissions_resolve(seal, 1);
}

fn assert_waiting_admissions_resolve(seal: bool, count: usize) {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "held-admission");
    let identity = runtime.main_branch_identity();
    let services = runtime.owner_component_services();
    let binding = runtime.owner_binding();
    let lifecycle = services.lifecycle_port();
    let (events, observed) = mpsc::channel();
    let hold = runtime.try_hold_admission().unwrap();
    assert_eq!(
        lifecycle.owner_lifecycle_observation(),
        RelationalOwnerLifecycleObservation::Held
    );
    let mut admissions = Vec::new();
    for _ in 0..count {
        binding.install_test_hold_wait_ack(events.clone());
        let completed = events.clone();
        let port = services.basis_port();
        let identity = identity.clone();
        admissions.push(std::thread::spawn(move || {
            let result = port.observe_branch(&identity);
            completed.send(false).unwrap();
            result
        }));
        assert!(
            observed.recv().unwrap(),
            "the first event must be waiting, not completion"
        );
    }
    if seal {
        hold.seal();
    } else {
        drop(hold);
    }
    assert_eq!(
        lifecycle.owner_lifecycle_observation(),
        if seal {
            RelationalOwnerLifecycleObservation::Closed
        } else {
            RelationalOwnerLifecycleObservation::Open
        }
    );
    for admission in admissions {
        let result = admission.join().unwrap();
        if seal {
            assert!(matches!(
                result,
                Err(RelationalBranchBasisDenial::OwnerUnavailable)
            ));
        } else {
            result.expect("release admits every waiting observation");
        }
    }
}

#[test]
fn checkpoint_through_a_hold_equals_the_sealed_checkpoint() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "held-checkpoint-first");
    create_entity(&runtime, "held-checkpoint-second");
    let hold = runtime.try_hold_admission().unwrap();
    let captured = hold.native_checkpoint().unwrap();
    hold.seal();
    assert_eq!(
        captured,
        runtime.durability_authority().native_checkpoint().unwrap()
    );
    let mut restored = runtime_with_test_schema();
    restored
        .durability_recovery()
        .restore_native_checkpoint(&captured)
        .unwrap();
    assert_eq!(
        restored.history().latest_commit(),
        runtime.history().latest_commit()
    );
}

#[test]
fn admission_hold_dropped_on_unwind_releases() {
    let mut runtime = runtime_with_test_schema();
    let binding = runtime.owner_binding();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _hold = runtime.try_hold_admission().unwrap();
        panic!("unwind through an unresolved hold");
    }));
    assert!(result.is_err());
    assert_eq!(
        binding.admission_posture(),
        crate::runtime::RelationalRuntimeAdmissionPosture::Open
    );
    assert!(binding.admit().is_some());
    runtime.try_hold_admission().unwrap().seal();
}
