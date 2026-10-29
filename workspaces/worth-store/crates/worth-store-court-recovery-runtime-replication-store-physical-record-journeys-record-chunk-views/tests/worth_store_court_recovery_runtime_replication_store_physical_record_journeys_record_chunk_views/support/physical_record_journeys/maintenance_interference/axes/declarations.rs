pub(crate) use {std::num::NonZeroU32};

pub(crate) use {worth_store::physical_runtime::PhysicalReadProtectionDenial, worth_store::physical_runtime::PhysicalReadProtectionPolicy};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::maintenance_interference::append, worth_store_testkit_store::physical_record_journeys::maintenance_interference::initialize_protection, worth_store_testkit_store::physical_record_journeys::maintenance_interference::placement};

pub(crate) fn admit_live_leases(bound: u32) {
    let parent = tempfile::tempdir().unwrap();
    let serving = initialize_protection(
        &parent.path().join("store"),
        worth_store::physical_runtime::PhysicalReadProtectionPolicy::new(nonzero(bound), nonzero(1)),
    );
    let record = append(&serving, placement(), 1, b"lease");
    let mut readers = Vec::new();
    for index in 0..bound {
        let reader = serving.records().unwrap();
        if index == 0 {
            let mut session = reader.open(record, worth_store_testkit_store::physical_record_journeys::maintenance_interference::limits()).unwrap();
            assert_eq!(session.next_chunk().unwrap().unwrap().bytes(), b"lease");
        }
        readers.push(reader);
    }
    let observer = serving.read_protection_observer();
    assert_eq!(observer.snapshot().live_acquisitions(), bound);
    let before = serving.media_counters();
    assert!(matches!(
        serving.records(),
        Err(worth_store::physical_runtime::PhysicalReadProtectionDenial::ProtectionLimit)
    ));
    assert_eq!(serving.media_counters(), before);
    drop(readers);
    assert_eq!(observer.snapshot().live_acquisitions(), 0);
    serving.close();
}

pub(crate) fn admit_retained_roots(bound: u32) {
    let parent = tempfile::tempdir().unwrap();
    let serving = initialize_protection(
        &parent.path().join("store"),
        worth_store::physical_runtime::PhysicalReadProtectionPolicy::new(nonzero(64), nonzero(bound)),
    );
    let policy = placement();
    let mut readers = Vec::new();
    for index in 0..bound {
        append(&serving, policy, u64::from(index) + 1, b"generation");
        readers.push(serving.records().unwrap());
    }
    assert_eq!(readers.len() as u32, bound);
    append(&serving, policy, u64::from(bound) + 1, b"one-over");
    let before = serving.media_counters();
    assert!(matches!(
        serving.records(),
        Err(worth_store::physical_runtime::PhysicalReadProtectionDenial::RetainedRootLimit)
    ));
    assert_eq!(serving.media_counters(), before);
    assert_eq!(
        serving
            .read_protection_observer()
            .snapshot()
            .protected_roots(),
        bound
    );
    drop(readers);
    serving.close();
}

pub(crate) fn nonzero(value: u32) -> NonZeroU32 {
    std::num::NonZeroU32::new(value).unwrap()
}