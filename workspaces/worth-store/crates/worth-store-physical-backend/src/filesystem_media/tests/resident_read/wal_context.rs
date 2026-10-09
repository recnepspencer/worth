use super::*;

#[test]
fn wal_context_copy_denial_precedes_payload_read_and_same_owner_retries() {
    // Arbitrary payload bytes exercise C.4 allocation/read ordering, not WAL
    // grammar or recovery authority. The listed directory needs no context copy.
    let name = std::ffi::OsStr::new("observed.wal");
    let (parent, observer, mut discovery) = discovery(
        |root| {
            let wal = root.join("families/wal");
            std::fs::create_dir_all(wal.join("nested")).expect("real WAL directory");
            std::fs::write(wal.join(name), b"payload").expect("real observed file");
        },
        4,
        32,
    );
    let reads_before = observer.snapshot().positioned_read_attempts();
    let context_calls = Cell::new(0);
    let denied = discovery
        .read_wal_artifacts_with_allocators(
            segments(2),
            uncharged(),
            |count| Ok(Vec::with_capacity(count)),
            |_| panic!("context refusal must precede payload allocation"),
            |count| {
                assert_eq!(count, name.as_encoded_bytes().len());
                context_calls.set(context_calls.get() + 1);
                Err::<std::ffi::OsString, _>(DeniedAllocation)
            },
        )
        .observed();
    assert!(matches!(
        denied,
        Err(RecoveryDiscoveryAllocationFailure::Allocation {
            artifact: RecoveryDiscoveryArtifact::WalDirectory,
            offset: 0,
            requested,
            cause: DeniedAllocation,
        }) if requested == name.as_encoded_bytes().len()
    ));
    assert_eq!(context_calls.get(), 1);
    assert_eq!(discovery.counters().directory_entries_observed, 2);
    assert_eq!(discovery.counters().bytes_read, 0);
    assert_eq!(discovery.counters().wal_bytes_read, 0);
    assert_eq!(observer.snapshot().positioned_read_attempts(), reads_before);

    for mut storage in [
        std::ffi::OsString::from("wrong.wal"),
        std::ffi::OsString::new(),
    ] {
        let expected = if storage.is_empty() {
            (name.as_encoded_bytes().len(), storage.capacity())
        } else {
            (0, storage.len())
        };
        let malformed = discovery
            .read_wal_artifacts_with_allocators(
                segments(2),
                uncharged(),
                |count| Ok(Vec::with_capacity(count)),
                |_| panic!("malformed context storage must precede payload allocation"),
                |_| Ok::<_, DeniedAllocation>(std::mem::take(&mut storage)),
            )
            .observed();
        assert!(matches!(malformed,
            Err(RecoveryDiscoveryAllocationFailure::BufferLengthMismatch {
                artifact: RecoveryDiscoveryArtifact::WalDirectory, offset: 0,
                requested, observed,
            }) if (requested, observed) == expected
        ));
        assert_eq!(discovery.counters().bytes_read, 0);
        assert_eq!(observer.snapshot().positioned_read_attempts(), reads_before);
    }

    let observed = discovery
        .read_wal_artifacts_with_allocators(
            segments(2),
            uncharged(),
            |count| Ok(Vec::with_capacity(count)),
            |length| Ok(vec![0; length]),
            |count| {
                assert_eq!(count, name.as_encoded_bytes().len());
                context_calls.set(context_calls.get() + 1);
                Ok::<_, DeniedAllocation>(std::ffi::OsString::with_capacity(count))
            },
        )
        .observed()
        .expect("same discovery retries after context refusal");
    assert_eq!(context_calls.get(), 2);
    assert_eq!(observed.len(), 2);
    let file = observed.iter().find(|entry| entry.name() == name).unwrap();
    assert_eq!(file.bytes(), Some(b"payload".as_slice()));
    assert_eq!(discovery.counters().bytes_read, 7);
    assert_eq!(discovery.counters().wal_bytes_read, 7);
    assert_eq!(
        observer.snapshot().positioned_read_attempts(),
        reads_before + 1
    );
    assert_eq!(discovery.finish().recovery_effect_count(), 0);
    assert_eq!(
        std::fs::read(parent.path().join("store/families/wal").join(name)).unwrap(),
        b"payload"
    );
}
