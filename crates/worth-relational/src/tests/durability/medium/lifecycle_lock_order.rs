//! Force exclusive lifecycle admission before a competing transition contacts it.
use super::slow_publication::{PublicationGate, SlowFirstPublication};
use crate::facade::history::BranchId;
use crate::tests::support::*;
use std::sync::{mpsc, Arc};
use std::time::Duration;
use worth_execution::ExecutionRequest;

#[derive(Clone, Copy, Debug)]
enum Snapshot {
    Checkpoint,
    RuntimeFork,
    Reclamation,
}
#[derive(Clone, Copy, Debug)]
enum Transition {
    Fork,
    Archive,
}

fn exclusive_before_transition(snapshot: Snapshot, transition: Transition) {
    let runtime = Arc::new(durable_runtime_with_test_schema());
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    create_entity(execution, &runtime, "source");
    let identity = runtime.branch_identity(&BranchId("main".into())).unwrap();
    let (_, source) = runtime.observe_fork_source(identity.branch_id()).unwrap();
    let routes = runtime.history.canonical_checkpoint_gate();
    let pause = routes.pause_exclusive_snapshot_for_test();
    let contacts = routes.transition_lifecycle_contacts_for_test();
    let (snapshot_done, snapshot_result) = mpsc::sync_channel(1);
    let observing = Arc::clone(&runtime);
    std::thread::spawn(move || {
        let request = crate::test_execution_policy::serial_request();
        let execution = &ExecutionRequest::serial(&request);
        match snapshot {
            Snapshot::Checkpoint => {
                if let Err(stop) = observing.durability_authority().checkpoint(execution) {
                    assert_eq!(expect_durability_failure(stop).class, crate::durability::data::RecoveryFailureClass::CheckpointPublicationInFlight);
                }
            }
            Snapshot::RuntimeFork => {
                observing.fork().unwrap();
            }
            Snapshot::Reclamation => {
                assert!(observing.run_index_generation_reclamation_pass().is_some());
            }
        }
        snapshot_done.send(()).unwrap();
    });
    pause.await_admission();
    let (transition_done, transition_result) = mpsc::sync_channel(1);
    let publishing = Arc::clone(&runtime);
    std::thread::spawn(move || {
        let request = crate::test_execution_policy::serial_request();
        let execution = &ExecutionRequest::serial(&request);
        match transition {
            Transition::Fork => {
                publishing
                    .fork_branch(execution, BranchId("target".into()), source)
                    .unwrap();
            }
            Transition::Archive => {
                publishing
                    .owner_component_services()
                    .lifecycle_port()
                    .archive_branch(execution, &identity)
                    .unwrap();
            }
        }
        transition_done.send(()).unwrap();
    });
    routes.await_transition_lifecycle_contact_for_test(contacts);
    // At this rendezvous the rejected order already holds source state; the
    // corrected order is still waiting before coordination/state acquisition.
    pause.release();
    snapshot_result
        .recv_timeout(Duration::from_secs(10))
        .unwrap_or_else(|stop| panic!("{snapshot:?} deadlocked against {transition:?}: {stop}"));
    transition_result
        .recv_timeout(Duration::from_secs(10))
        .unwrap_or_else(|stop| panic!("{transition:?} did not finish after {snapshot:?}: {stop}"));
    assert_eq!(runtime.patch_position_reservation_counters().deferrals, 0);
}

#[test]
fn fork_against_checkpoint_capture() {
    exclusive_before_transition(Snapshot::Checkpoint, Transition::Fork);
}
#[test]
fn fork_against_runtime_fork() {
    exclusive_before_transition(Snapshot::RuntimeFork, Transition::Fork);
}
#[test]
fn fork_against_reclamation() {
    exclusive_before_transition(Snapshot::Reclamation, Transition::Fork);
}
#[test]
fn archive_against_checkpoint_capture() {
    exclusive_before_transition(Snapshot::Checkpoint, Transition::Archive);
}
#[test]
fn archive_against_runtime_fork() {
    exclusive_before_transition(Snapshot::RuntimeFork, Transition::Archive);
}
#[test]
fn archive_against_reclamation() {
    exclusive_before_transition(Snapshot::Reclamation, Transition::Archive);
}

#[test]
fn checkpoint_registry_is_readable_while_a_fork_is_durable_not_yet_live() {
    let contents = TestDurableMedium::new();
    let gate = Arc::new(PublicationGate::default());
    let runtime = Arc::new(
        crate::facade::runtime::RelationalRuntimeApi::builder()
            .profile(crate::config::data::RelationalRuntimeProfile::CertificationCore)
            .schema_registry(test_schema_registry())
            .build_durable(Box::new(SlowFirstPublication {
                contents: contents.attachment(),
                gate: Arc::clone(&gate),
            })),
    );
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    create_entity(execution, &runtime, "source");
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    gate.arm();
    let (published, publication) = mpsc::sync_channel(1);
    let publishing = Arc::clone(&runtime);
    std::thread::spawn(move || {
        let request = crate::test_execution_policy::serial_request();
        publishing
            .fork_branch(
                &ExecutionRequest::serial(&request),
                BranchId("target".into()),
                source,
            )
            .unwrap();
        published.send(()).unwrap();
    });
    gate.await_publication();
    let (captured, capture) = mpsc::sync_channel(1);
    let capturing = Arc::clone(&runtime);
    std::thread::spawn(move || {
        let cells = capturing.history().branch_cells_snapshot();
        let lifecycle = capturing.history.branch_lifecycle_snapshot();
        assert_eq!(cells.len(), 1);
        assert_eq!(lifecycle.len(), 1);
        captured.send(()).unwrap();
    });
    // Fork coordination still excludes source writers, but a checkpoint reader
    // must not wait on source state held across the medium's durable I/O.
    capture
        .recv_timeout(Duration::from_secs(10))
        .expect("checkpoint registry must remain readable");
    gate.open();
    publication.recv_timeout(Duration::from_secs(10)).unwrap();
    assert_eq!(contents.transition_sequences().len(), 1);
}
