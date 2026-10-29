pub(crate) use {std::sync::mpsc};
pub(crate) use {std::thread};
pub(crate) use {std::time::Duration};

pub(crate) use {worth_store::physical_runtime::PhysicalReadProtectionDisposition, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::RecordStreamFailureKind};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture, crate::record_chunk_views::managed_append::append};

pub(crate) fn race_read_with_shutdown(abort: bool, copy: bool) {
    let parent = tempfile::tempdir().unwrap();
    let (serving, placement) = worth_store_testkit_store::physical_record_journeys::record_chunk_views::fixture::initialize(&parent.path().join("warm-read-close"));
    let expected = b"warm bytes must survive shutdown";
    let record = append(&serving, placement, 211, expected);
    let observer = serving.read_protection_observer();
    let mut session = serving
        .records()
        .unwrap()
        .open(
            record,
            worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(100).unwrap()),
        )
        .unwrap();
    let work_before = session.observation().physical_work_count();
    let pause = session.certification_pause_next_read_call();
    let (closed, await_close) = std::sync::mpsc::sync_channel(1);
    let reader = std::thread::spawn(move || {
        if copy {
            let mut bytes = [0; 100];
            let count = session.read_next(&mut bytes).unwrap();
            await_close.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
            assert_eq!(&bytes[..count], expected);
        } else {
            let chunk = session.next_chunk().unwrap().unwrap();
            await_close.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
            assert_eq!(chunk.bytes(), expected);
        }
        assert_eq!(session.observation().physical_work_count(), work_before);
        assert!(
            matches!(session.next_chunk(), Err(error) if error.kind() == worth_store::physical_runtime::RecordStreamFailureKind::RuntimeReleased)
        );
    });
    assert!(pause.await_arrival());
    let shutdown = std::thread::spawn(move || {
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
        worth_store::physical_runtime::PhysicalReadProtectionDisposition::RetainedUntilReadersRelease
    );
    closed.send(()).unwrap();
    reader.join().unwrap();
    assert_eq!(observer.snapshot().live_acquisitions(), 0);
    assert_eq!(observer.snapshot().releases(), 1);
}