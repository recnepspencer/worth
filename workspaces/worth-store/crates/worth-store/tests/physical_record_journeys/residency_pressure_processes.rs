use std::io::Write;
use std::path::Path;

use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    ExternalPhysicalRecordLocator, PhysicalOperationAllocationScope, PhysicalRecordInitialization,
    PhysicalRecordOpen, PhysicalRecordResidencyPolicy, PhysicalResidencyDimension,
    PhysicalSpeculativeWorkKind, RecordAppendBatch, RecordByteLimit, RecordReadLimits,
};
use worth_store_physical_backend::MediaOperationRole;

use super::{
    allocation_probe::peak_live_bytes_during, configuration, durable_publication, media,
    stream_fixture::RepeatedByteSource, success,
};

const RESIDENT_BYTES: u64 = 64 * 1024;
const RECORD_BYTES: usize = 3_000;
const RECORDS: usize = 192;

fn policy(format: AdmittedPhysicalRecordFormat) -> AdmittedPhysicalRecordResidencyPolicy {
    use PhysicalOperationAllocationScope as Scope;
    use PhysicalSpeculativeWorkKind as Speculation;

    let operation_bytes = 16 * 1024 * 1024;
    PhysicalRecordResidencyPolicy::builder()
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

#[test]
fn bounded_residency_survives_pressure_across_three_processes() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("bounded-residency");
    let locators = parent.path().join("locators.txt");
    let writer = super::child_process::run_child(
        "residency_pressure_writer",
        &root,
        Some(locators.to_str().unwrap()),
    );
    assert!(writer
        .lines()
        .any(|line| line == "RESIDENCY_PRESSURE writer-ok"));
    assert!(directory_bytes(&root) >= RESIDENT_BYTES * 8);

    for _ in 0..2 {
        let reader = super::child_process::run_child(
            "residency_pressure_reader",
            &root,
            Some(locators.to_str().unwrap()),
        );
        assert!(reader
            .lines()
            .any(|line| line == "RESIDENCY_PRESSURE reader-ok"));
    }
}

pub(super) fn pressure_writer(root: &Path, locator_path: &str) {
    let (format, placement, access) = configuration();
    let serving = success(initialize_record_store!(media(root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
            .with_residency_policy(policy(format))
    },));
    let batch = (0..RECORDS).fold(RecordAppendBatch::builder(), |batch, ordinal| {
        batch.push_source(RepeatedByteSource::new(RECORD_BYTES as u64, ordinal as u8))
    });
    let (published, peak_live_bytes) = peak_live_bytes_during(|| {
        durable_publication::publish_single(
            &serving,
            placement,
            durable_publication::certification_material("residency-pressure-process", 1),
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
            .for_dimension(PhysicalResidencyDimension::TotalBytes)
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
        let locator = ExternalPhysicalRecordLocator::new(
            serving.store_identity(),
            published.record_id(index).unwrap(),
        );
        writeln!(
            locator_file,
            "{}",
            super::child_process::hex(&locator.encode())
        )
        .unwrap();
    }
    locator_file.flush().unwrap();
    assert!(!serving.close().residency().requires_inspection());
    println!("RESIDENCY_PRESSURE writer-ok");
}

pub(super) fn pressure_reader(root: &Path, locator_path: &str) {
    let (format, _, access) = configuration();
    let serving = success(open_record_store!(media(root), |durability| {
        PhysicalRecordOpen::new(format, access, durability).with_residency_policy(policy(format))
    },));
    let locators = std::fs::read_to_string(locator_path)
        .unwrap()
        .lines()
        .map(super::child_process::decode_locator)
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

fn nonzero_bytes(value: u64) -> std::num::NonZeroU64 {
    std::num::NonZeroU64::new(value).unwrap()
}

fn nonzero_count(value: u32) -> std::num::NonZeroU32 {
    std::num::NonZeroU32::new(value).unwrap()
}

fn open(
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
            RecordReadLimits::new(RecordByteLimit::new(RECORD_BYTES as u32).unwrap()),
        )
}

fn assert_record(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    locator: ExternalPhysicalRecordLocator,
    expected: u8,
) {
    let mut record = open(serving, locator).unwrap();
    let mut bytes = [0_u8; RECORD_BYTES];
    assert_eq!(record.read_next(&mut bytes).unwrap(), RECORD_BYTES);
    assert!(bytes.iter().all(|byte| *byte == expected));
}

fn positioned_reads(serving: &worth_store::physical_runtime::ServingPhysicalRuntime) -> u64 {
    serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedRead)
}

fn directory_bytes(root: &Path) -> u64 {
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
