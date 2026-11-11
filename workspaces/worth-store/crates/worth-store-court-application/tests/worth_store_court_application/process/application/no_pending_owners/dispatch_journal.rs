//! The Home lends one dispatch-journal operation. A recorded prefix refuses
//! service until its Query owner admits it, and survives graceful reopen.
//! Only Query admits a prefix, so this court never does.
use super::*;
use std::time::{Duration, Instant};
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam;
use worth_store_application::{
    ApplicationHomeDispatchJournalDenial as JournalDenial, ApplicationHomeDispatchJournalOperation,
    ApplicationHomeDispatchPublication as Publication,
    ApplicationHomeOwnerAdmissionDenial as Denial,
};
use worth_store_testkit_application_oracle::world_home::WorldHomeFixture;

const MAXIMUM_FRAME_BYTES: u64 = 64;

/// Drive the one original publication and return its committed payload digest.
fn publish(
    operation: &ApplicationHomeDispatchJournalOperation,
    execution: &ExecutionRequest<'_, '_>,
    position: u64,
    frame: &[u8],
) -> [u8; 32] {
    loop {
        match operation.advance_publication(execution, position, frame) {
            Publication::Pending => std::thread::yield_now(),
            Publication::Committed(proof) => {
                return proof.consume(|committed| {
                    let subject = committed.subject();
                    assert_eq!(subject.admit_to(operation.authority()), Ok(()));
                    assert_eq!(subject.position(), position);
                    subject.payload_digest()
                })
            }
            other => panic!("the quiescent Home commits its original frame: {other:?}"),
        }
    }
}

fn read(
    operation: &ApplicationHomeDispatchJournalOperation,
    execution: &ExecutionRequest<'_, '_>,
    position: u64,
) -> Option<Vec<u8>> {
    operation
        .read_frame(execution, position, MAXIMUM_FRAME_BYTES)
        .unwrap()
}

#[test]
fn a_recorded_dispatch_prefix_refuses_service_until_its_query_owner_admits_it() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open(directory.path(), &execution);
    drop(fixture.bootstrap_root(&execution));
    let home = fixture.home;
    let operation = home.query_dispatch_journal_operation();
    assert_eq!(operation.home(), home.home_identity());
    assert_eq!(
        read(&operation, &execution, 1),
        None,
        "an empty journal proves position one unwritten"
    );
    assert!(matches!(
        operation.read_frame(&execution, 2, MAXIMUM_FRAME_BYTES),
        Err(JournalDenial::PrefixMismatch)
    ));
    publish(&operation, &execution, 1, b"opaque-dispatch-frame");
    assert_eq!(
        read(&operation, &execution, 1).as_deref(),
        Some(b"opaque-dispatch-frame".as_slice())
    );
    let refused = home
        .readmit_no_pending_owners(&execution)
        .err()
        .expect("a recorded prefix without its Query owner cannot reach service");
    assert!(matches!(
        refused.reason(),
        Denial::QueryDispatch(JournalDenial::AdmissionUnavailable)
    ));
    let home = refused.into_home();
    assert_eq!(
        read(&operation, &execution, 1).as_deref(),
        Some(b"opaque-dispatch-frame".as_slice()),
        "refused owner admission cannot alter the original prefix"
    );
    close(home, &execution);
    assert!(
        matches!(
            operation.advance_publication(&execution, 2, b"after-close"),
            Publication::Refused(JournalDenial::OwnerUnavailable)
        ),
        "a released Home's operation cannot publish"
    );
}

#[test]
fn recorded_dispatch_frames_survive_graceful_reopen_under_a_fresh_incarnation() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open(directory.path(), &execution);
    drop(fixture.bootstrap_root(&execution));
    let home = fixture.home;
    let identity = home.home_identity();
    let original = home.query_dispatch_journal_operation();
    let first = publish(&original, &execution, 1, b"attempt");
    publish(&original, &execution, 2, b"observation");
    close(home, &execution);

    let reopened = open(directory.path(), &execution).home;
    assert_eq!(reopened.home_identity(), identity);
    let operation = reopened.query_dispatch_journal_operation();
    assert_ne!(operation.incarnation(), original.incarnation());
    for (position, frame) in [(1, b"attempt".as_slice()), (2, b"observation".as_slice())] {
        assert_eq!(
            read(&operation, &execution, position).as_deref(),
            Some(frame),
            "graceful reopen preserves the original recorded frame"
        );
    }
    assert_eq!(read(&operation, &execution, 3), None);
    let third = publish(&operation, &execution, 3, b"next-attempt");
    assert_ne!(third, first);
    let refused = reopened
        .readmit_no_pending_owners(&execution)
        .err()
        .expect("the reopened prefix still awaits this incarnation's Query admission");
    assert!(matches!(
        refused.reason(),
        Denial::QueryDispatch(JournalDenial::AdmissionUnavailable)
    ));
    close(refused.into_home(), &execution);
}

/// Drive one step sequence to its first terminal answer.
fn drive(
    operation: &ApplicationHomeDispatchJournalOperation,
    execution: &ExecutionRequest<'_, '_>,
    frame: &[u8],
) -> Publication {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match operation.advance_publication(execution, 1, frame) {
            Publication::Pending => {
                assert!(
                    Instant::now() < deadline,
                    "the original dispatch publication must settle"
                );
                std::thread::yield_now();
            }
            terminal @ (Publication::Committed(_)
            | Publication::ProvenNoEffect(_)
            | Publication::Refused(_)
            | Publication::Unresolved) => return terminal,
        }
    }
}

/// A first frame whose invoking request stopped while its real physical
/// publication was paused at `seam`. The Home keeps that original publication.
fn stopped_at(
    seam: Seam,
    frame: &[u8],
) -> (
    Box<WorldHomeFixture>,
    ApplicationHomeDispatchJournalOperation,
    tempfile::TempDir,
) {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = Box::new(open(directory.path(), &execution));
    drop(fixture.bootstrap_root(&execution));
    let operation = fixture.home.query_dispatch_journal_operation();
    let gate = fixture
        .home
        .physical_serving_for_test()
        .certification_pause_physical_mutation_at(seam);
    let cancellation = CancellationSource::new();
    let stopped =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| drive(&operation, &ExecutionRequest::serial(&stopped), frame));
        let deadline = Instant::now() + Duration::from_secs(30);
        let arrived = loop {
            if gate.await_arrival() {
                break true;
            }
            if worker.is_finished() || Instant::now() >= deadline {
                break false;
            }
        };
        cancellation.cancel();
        while !worker.is_finished() && Instant::now() < deadline {
            std::thread::yield_now();
        }
        let returned_while_paused = worker.is_finished();
        // Release even on a failing assertion; a scoped join must not park
        // behind a gate retained by the joining parent.
        gate.release();
        let answer = worker.join().unwrap();
        assert!(
            arrived,
            "the original publication must reach its real physical seam"
        );
        assert!(
            returned_while_paused,
            "the stopped caller returns before physical settlement"
        );
        assert!(
            matches!(answer, Publication::Refused(JournalDenial::Execution(_))),
            "{answer:?}"
        );
    });
    (fixture, operation, directory)
}

#[test]
fn a_stopped_dispatch_publication_readmits_its_actual_commit_without_a_second_frame() {
    let (fixture, operation, _directory) = stopped_at(Seam::AfterRootReplacement, b"attempt");
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    for _same_original in 0..2 {
        let Publication::Committed(proof) = drive(&operation, &execution, b"attempt") else {
            panic!("the performed original hands back committed custody");
        };
        proof.consume(|committed| {
            assert_eq!(committed.subject().admit_to(operation.authority()), Ok(()));
            assert_eq!(committed.subject().position(), 1);
        });
        assert_eq!(
            read(&operation, &execution, 1).as_deref(),
            Some(b"attempt".as_slice())
        );
        assert_eq!(
            read(&operation, &execution, 2),
            None,
            "the stopped caller's frame was recorded exactly once"
        );
    }
    close(fixture.home, &execution);
}

#[test]
fn a_stopped_dispatch_publication_proves_no_effect_before_a_fresh_frame_commits() {
    let (fixture, operation, _directory) = stopped_at(Seam::BeforeEffectCutover, b"attempt");
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let Publication::ProvenNoEffect(proof) = drive(&operation, &execution, b"attempt") else {
        panic!("the stopped original has a sealed no-effect fate");
    };
    assert_eq!(proof.subject().admit_to(operation.authority()), Ok(()));
    assert_eq!(proof.subject().position(), 1);
    assert_eq!(
        read(&operation, &execution, 1),
        None,
        "a proven no-effect frame was never written"
    );
    assert!(
        matches!(
            operation.advance_publication(&execution, 1, b"another-frame"),
            Publication::Refused(JournalDenial::PublicationPending)
        ),
        "the original stays retained until its no-effect proof is acknowledged"
    );
    operation.acknowledge_no_effect(proof).unwrap();
    let Publication::Committed(proof) = drive(&operation, &execution, b"fresh-attempt") else {
        panic!("a fresh frame commits at the same position after proven no effect");
    };
    proof.consume(|committed| assert_eq!(committed.subject().position(), 1));
    assert_eq!(
        read(&operation, &execution, 1).as_deref(),
        Some(b"fresh-attempt".as_slice())
    );
    close(fixture.home, &execution);
}
