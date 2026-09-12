use std::thread;

use worth_store::physical_runtime::{
    certification::CertificationReadRootCaptureStage, RecordByteLimit, RecordReadDenial,
    RecordReadLimits,
};

use super::{fixture, managed_append::append};

#[test]
fn publication_before_capture_registers_only_the_new_root() {
    let parent = tempfile::tempdir().unwrap();
    let (serving, placement) = fixture::initialize(&parent.path().join("publish-first"));
    append(&serving, placement, 221, b"baseline");
    let old_root = serving.records().unwrap().protected_root();
    let observer = serving.read_protection_observer();
    thread::scope(|threads| {
        // Keep the gate inside the scope: a failing assertion drops it before
        // scoped threads are joined, so the reader cannot strand teardown.
        let pause = serving.certification_pause_next_read_root_capture(
            CertificationReadRootCaptureStage::BeforeRootLock,
        );
        let capture = threads.spawn(|| serving.records().unwrap());
        assert!(pause.await_arrival());
        let added = append(&serving, placement, 222, b"published-before-capture");
        assert_eq!(observer.snapshot().live_acquisitions(), 0);
        pause.release();
        let reader = capture.join().unwrap();
        assert_ne!(reader.protected_root().root(), old_root.root());
        assert_eq!(observer.acquisitions_for_root(old_root), 0);
        assert_eq!(observer.acquisitions_for_root(reader.protected_root()), 1);
        let mut session = reader.open(added, limits()).unwrap();
        assert_eq!(
            session.next_chunk().unwrap().unwrap().bytes(),
            b"published-before-capture"
        );
    });
    assert_eq!(observer.snapshot().live_acquisitions(), 0);
    fixture::assert_clean_close(serving);
}

#[test]
fn observed_root_is_registered_before_publication_can_cross_capture() {
    let parent = tempfile::tempdir().unwrap();
    let (serving, placement) = fixture::initialize(&parent.path().join("capture-first"));
    append(&serving, placement, 223, b"baseline");
    let old_root = serving.records().unwrap().protected_root();
    let observer = serving.read_protection_observer();
    thread::scope(|threads| {
        let pause = serving.certification_pause_next_read_root_capture(
            CertificationReadRootCaptureStage::AfterObservationBeforeRegistration,
        );
        let capture = threads.spawn(|| serving.records().unwrap());
        assert!(pause.await_arrival());
        assert_eq!(observer.acquisitions_for_root(old_root), 0);
        let publication = threads.spawn(|| append(&serving, placement, 224, b"after-capture"));
        // Positive observation of WouldBlock on the real publication mutex,
        // not a timeout used to infer that the publisher probably waited.
        assert!(pause.await_publication_lock_wait());
        pause.release();
        let reader = capture.join().unwrap();
        let added = publication.join().unwrap();
        assert_eq!(reader.protected_root(), old_root);
        assert_eq!(observer.acquisitions_for_root(old_root), 1);
        assert!(matches!(reader.open(added, limits()), Err(error)
            if error.denial() == RecordReadDenial::RecordNotFound));
        let current = serving.records().unwrap();
        assert_ne!(current.protected_root().root(), old_root.root());
        let mut session = current.open(added, limits()).unwrap();
        assert_eq!(
            session.next_chunk().unwrap().unwrap().bytes(),
            b"after-capture"
        );
        drop(reader);
        assert_eq!(observer.acquisitions_for_root(old_root), 0);
    });
    assert_eq!(observer.snapshot().live_acquisitions(), 0);
    fixture::assert_clean_close(serving);
}

fn limits() -> RecordReadLimits {
    RecordReadLimits::new(RecordByteLimit::new(100).unwrap())
}
