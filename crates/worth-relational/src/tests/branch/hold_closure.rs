use std::sync::mpsc;

use crate::branch::{RelationalBranchBasisDenial, RelationalOwnerLifecycleObservation};
use crate::runtime::RelationalRuntimeAdmissionPosture;
use crate::tests::support::{create_entity, runtime_with_test_schema};

#[test]
fn forgotten_hold_then_owner_drop_denies_parked_and_new_admissions() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "forgotten-held-admission");
    let identity = runtime.main_branch_identity();
    let services = runtime.owner_component_services();
    let binding = runtime.owner_binding();
    let (events, observed) = mpsc::channel();
    binding.install_test_hold_wait_ack(events.clone());
    let hold = runtime.try_hold_admission().unwrap();
    let port = services.basis_port();
    let parked_identity = identity.clone();
    let parked = std::thread::spawn(move || {
        let result = port.observe_branch(&parked_identity);
        events.send(false).unwrap();
        result
    });
    assert!(
        observed.recv().unwrap(),
        "the admission parks before the owner closes"
    );
    std::mem::forget(hold);
    drop(runtime);
    assert_eq!(
        binding.admission_posture(),
        RelationalRuntimeAdmissionPosture::Closed
    );
    assert!(
        !binding.test_has_hold(),
        "owner close resolves even a forgotten hold"
    );
    assert_eq!(
        services.lifecycle_port().owner_lifecycle_observation(),
        RelationalOwnerLifecycleObservation::Closed
    );
    assert!(matches!(
        parked.join().unwrap(),
        Err(RelationalBranchBasisDenial::OwnerUnavailable)
    ));
    assert!(matches!(
        services.basis_port().observe_branch(&identity),
        Err(RelationalBranchBasisDenial::OwnerUnavailable)
    ));
    assert!(
        binding.admit().is_none(),
        "a new admission sees the closed word"
    );
}

#[test]
fn seal_preserves_a_waiting_admissions_provisional_increment() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "provisional-held-admission");
    let binding = runtime.owner_binding();
    let (paused, observed_pause) = mpsc::channel();
    let (resume, continued) = mpsc::channel();
    binding.install_test_stopped_admission_pause(paused.clone(), continued);
    let hold = runtime.try_hold_admission().unwrap();
    let port = binding.clone();
    let admission = std::thread::spawn(move || {
        let result = port.admit();
        paused.send(false).unwrap();
        result
    });
    assert!(
        observed_pause.recv().unwrap(),
        "admission pauses before returning its increment"
    );
    assert_eq!(binding.test_in_flight_count(), 1);
    hold.seal();
    let preserved = binding.test_in_flight_count();
    resume.send(()).unwrap();
    assert!(admission.join().unwrap().is_none());
    assert_eq!(preserved, 1, "sealing cannot erase a provisional increment");
    assert_eq!(binding.test_in_flight_count(), 0);
    assert_eq!(
        binding.admission_posture(),
        RelationalRuntimeAdmissionPosture::Closed
    );
}
