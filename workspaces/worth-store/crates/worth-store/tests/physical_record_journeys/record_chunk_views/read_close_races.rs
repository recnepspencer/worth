use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use worth_store::physical_runtime::{
    PhysicalReadProtectionDisposition, RecordByteLimit, RecordReadLimits, RecordStreamFailureKind,
};

use super::{fixture, managed_append::append};

#[test]
fn close_and_abort_drain_admitted_warm_chunk_and_copy_calls() {
    for abort in [false, true] {
        for copy in [false, true] {
            race_read_with_shutdown(abort, copy);
        }
    }
}

fn race_read_with_shutdown(abort: bool, copy: bool) {
    let parent = tempfile::tempdir().unwrap();
    let (serving, placement) = fixture::initialize(&parent.path().join("warm-read-close"));
    let expected = b"warm bytes must survive shutdown";
    let record = append(&serving, placement, 211, expected);
    let observer = serving.read_protection_observer();
    let mut session = serving
        .records()
        .unwrap()
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(100).unwrap()),
        )
        .unwrap();
    let work_before = session.observation().physical_work_count();
    let pause = session.certification_pause_next_read_call();
    let (closed, await_close) = mpsc::sync_channel(1);
    let reader = thread::spawn(move || {
        if copy {
            let mut bytes = [0; 100];
            let count = session.read_next(&mut bytes).unwrap();
            await_close.recv_timeout(Duration::from_secs(10)).unwrap();
            assert_eq!(&bytes[..count], expected);
        } else {
            let chunk = session.next_chunk().unwrap().unwrap();
            await_close.recv_timeout(Duration::from_secs(10)).unwrap();
            assert_eq!(chunk.bytes(), expected);
        }
        assert_eq!(session.observation().physical_work_count(), work_before);
        assert!(
            matches!(session.next_chunk(), Err(error) if error.kind() == RecordStreamFailureKind::RuntimeReleased)
        );
    });
    assert!(pause.await_arrival());
    let shutdown = thread::spawn(move || {
        if abort {
            serving.abort().read_protection()
        } else {
            serving.close().read_protection()
        }
    });
    // This observes the actual condition-variable drain, not a timing guess
    // that a spawned close thread probably ran. Removing read-call admission
    // makes this assertion fail (or shutdown panic) while the warm Arc is held.
    assert!(pause.await_shutdown_drain());
    assert!(observer.snapshot().revoked());
    pause.release();
    let outcome = shutdown
        .join()
        .expect("an admitted warm read cannot panic shutdown");
    assert_eq!(
        outcome.disposition(),
        PhysicalReadProtectionDisposition::RetainedUntilReadersRelease
    );
    closed.send(()).unwrap();
    reader.join().unwrap();
    assert_eq!(observer.snapshot().live_acquisitions(), 0);
    assert_eq!(observer.snapshot().releases(), 1);
}
