pub use std::io::Write;
pub use std::path::Path;

pub use worth_store_physical_backend::MediaOperationRole;
pub use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy,
    worth_store::physical_runtime::ExternalPhysicalRecordLocator,
    worth_store::physical_runtime::PhysicalOperationAllocationScope,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRecordResidencyPolicy,
    worth_store::physical_runtime::PhysicalResidencyDimension,
    worth_store::physical_runtime::PhysicalSpeculativeWorkKind,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
};

pub use {
    crate::allocation_probe::physical_record_journeys::peak_live_bytes_during,
    crate::durable_publication, crate::physical_record_journeys::configuration,
    crate::physical_record_journeys::media, crate::physical_record_journeys::success,
    crate::stream_fixture::RepeatedByteSource,
};

pub const RESIDENT_BYTES: u64 = 64 * 1024;
pub const RECORD_BYTES: usize = 3_000;
pub const RECORDS: usize = 192;

pub fn policy(format: AdmittedPhysicalRecordFormat) -> AdmittedPhysicalRecordResidencyPolicy {
    use worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope;
    use worth_store::physical_runtime::PhysicalSpeculativeWorkKind as Speculation;

    let operation_bytes = 16 * 1024 * 1024;
    worth_store::physical_runtime::PhysicalRecordResidencyPolicy::builder()
        .total_bytes(nonzero_bytes(
            operation_bytes + (2 * RESIDENT_BYTES) + (16 * 1024),
        ))
        .resident_bytes(nonzero_bytes(RESIDENT_BYTES))
        .metadata_bytes(nonzero_bytes(16 * 1024))
        .frame_entries(nonzero_count(8))
        .pinned_frames(nonzero_count(8))
        .pin_leases(nonzero_count(8))
        .dirty_frames(nonzero_count(2))
        .dirty_replacement_bytes(nonzero_bytes(RESIDENT_BYTES))
        .operation_bytes(nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::ForegroundRead, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::ForegroundWrite, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Recovery, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Scrub, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Maintenance, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Verification, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Blob, nonzero_bytes(operation_bytes))
        .speculative_frames(Speculation::Prefetch, nonzero_count(8))
        .speculative_frames(Speculation::ReadAhead, nonzero_count(8))
        .speculative_frames(Speculation::WriteBehind, nonzero_count(2))
        .admit(format)
        .into_result()
        .unwrap()
}

pub fn pressure_writer(root: &Path, locator_path: &str) {
    let (format, placement, access) = configuration();
    let serving = success(initialize_record_store!(media(root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )
        .with_residency_policy(policy(format))
    },));
    let batch = (0..RECORDS).fold(
        worth_store::physical_runtime::RecordAppendBatch::builder(),
        |batch, ordinal| {
            batch.push_source(crate::stream_fixture::RepeatedByteSource::new(
                RECORD_BYTES as u64,
                ordinal as u8,
            ))
        },
    );
    let (published, peak_live_bytes) = peak_live_bytes_during(|| {
        crate::durable_publication::root::publish_single(
            &serving,
            placement,
            crate::durable_publication::material::certification_material(
                "residency-pressure-process",
                1,
            ),
            batch.build().unwrap(),
        )
    });
    let published = &published.settled_members()[0];
    let observation = serving.residency_observation();
    let counters = observation.counters();
    assert!(observation.store_generation().get() > 0);
    assert_eq!(
        observation
            .allocations()
            .for_dimension(worth_store::physical_runtime::PhysicalResidencyDimension::TotalBytes)
            .active_units(),
        counters.admitted_bytes(),
    );
    assert!(counters.evictions() > 0);
    assert!(counters.peak_resident_bytes() <= RESIDENT_BYTES);
    assert!(counters.metadata_bytes() <= policy(format).metadata_bytes());
    assert!(counters.peak_operation_bytes() <= policy(format).operation_bytes());
    assert!(
        peak_live_bytes as u64 <= counters.peak_admitted_bytes(),
        "process-observed peak allocation {peak_live_bytes} escaped admitted resident, metadata, and operation memory {}",
        counters.peak_admitted_bytes(),
    );

    let mut locator_file = std::io::BufWriter::new(std::fs::File::create(locator_path).unwrap());
    for index in 0..RECORDS {
        let locator = worth_store::physical_runtime::ExternalPhysicalRecordLocator::new(
            serving.store_identity(),
            published.record_id(index).unwrap(),
        );
        writeln!(
            locator_file,
            "{}",
            crate::child_process::locator_codec::hex(&locator.encode())
        )
        .unwrap();
    }
    locator_file.flush().unwrap();
    assert!(!serving.close().residency().requires_inspection());
    println!("RESIDENCY_PRESSURE writer-ok");
}

pub fn pressure_reader(root: &Path, locator_path: &str) {
    let (format, _, access) = configuration();
    let serving = success(open_record_store!(media(root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
            .with_residency_policy(policy(format))
    },));
    let locators = std::fs::read_to_string(locator_path)
        .unwrap()
        .lines()
        .map(crate::child_process::locator_codec::decode_locator)
        .collect::<Vec<_>>();
    assert_eq!(locators.len(), RECORDS);

    let reads_before = positioned_reads(&serving);
    assert_record(&serving, locators[0], 0);
    let cold_reads = positioned_reads(&serving);
    assert!(cold_reads > reads_before);
    assert_record(&serving, locators[0], 0);
    assert_eq!(positioned_reads(&serving), cold_reads);
    for (ordinal, locator) in locators.iter().copied().enumerate().skip(1) {
        assert_record(&serving, locator, ordinal as u8);
    }
    let reads_after_pressure = positioned_reads(&serving);
    assert_record(&serving, locators[0], 0);
    assert!(positioned_reads(&serving) > reads_after_pressure);
    let counters = serving.residency_observation().counters();
    assert!(counters.evictions() > 0);
    assert!(counters.peak_resident_bytes() <= RESIDENT_BYTES);
    assert!(counters.eviction_candidate_inspections() >= counters.evictions());
    assert!(!serving.close().residency().requires_inspection());
    println!("RESIDENCY_PRESSURE reader-ok");
}

pub fn nonzero_bytes(value: u64) -> std::num::NonZeroU64 {
    std::num::NonZeroU64::new(value).unwrap()
}

pub fn nonzero_count(value: u32) -> std::num::NonZeroU32 {
    std::num::NonZeroU32::new(value).unwrap()
}

pub fn open(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    locator: ExternalPhysicalRecordLocator,
) -> Result<
    worth_store::physical_runtime::RecordReadSession,
    worth_store::physical_runtime::RecordReadError,
> {
    serving
        .records()
        .expect("read protection admission")
        .open_external(
            locator,
            worth_store::physical_runtime::RecordReadLimits::new(
                worth_store::physical_runtime::RecordByteLimit::new(RECORD_BYTES as u32).unwrap(),
            ),
        )
}

pub fn assert_record(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    locator: ExternalPhysicalRecordLocator,
    expected: u8,
) {
    let mut record = open(serving, locator).unwrap();
    let mut bytes = [0_u8; RECORD_BYTES];
    assert_eq!(record.read_next(&mut bytes).unwrap(), RECORD_BYTES);
    assert!(bytes.iter().all(|byte| *byte == expected));
}

pub fn positioned_reads(serving: &worth_store::physical_runtime::ServingPhysicalRuntime) -> u64 {
    serving
        .media_counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead)
}

pub fn directory_bytes(root: &Path) -> u64 {
    std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .map(|path| {
            if path.is_dir() {
                directory_bytes(&path)
            } else {
                path.metadata().unwrap().len()
            }
        })
        .sum()
}
