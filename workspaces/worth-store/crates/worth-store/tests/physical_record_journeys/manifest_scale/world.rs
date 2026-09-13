use std::path::PathBuf;

use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy, AdmittedRecordPlacementPolicy,
    ExternalPhysicalRecordLocator, PhysicalMutationIdempotencyMaterial, PhysicalRecordId,
    PhysicalRecordInitialization, PhysicalRecordOpen, PhysicalResidencyCounterSnapshot,
    RecordAppendBatch, RecordByteLimit, RecordReadLimits, RecordReadObservation,
    RecordScanCounterSnapshot,
};
use worth_store_offline_verifier::OfflineDurableManifestWalk;
use worth_store_physical_backend::{MediaCounterSnapshot, MediaOperationRole};

use super::super::scenario_evidence::ScenarioProcessEvidence;
use super::evidence::ScaleCourtroomEvidence;
use super::ScaleObservation;

struct SeededScaleWorld {
    _parent: tempfile::TempDir,
    root: PathBuf,
    format: AdmittedPhysicalRecordFormat,
    placement: AdmittedRecordPlacementPolicy,
    last: PhysicalRecordId,
    locator: ExternalPhysicalRecordLocator,
}

struct RuntimeScaleWorld {
    observation: ScaleObservation,
    walk: OfflineDurableManifestWalk,
    process: ScenarioProcessEvidence,
}

struct LiveScaleMeasurements {
    media_before_point: MediaCounterSnapshot,
    media_after_point: MediaCounterSnapshot,
    media_after_scan: MediaCounterSnapshot,
    residency_before_point: PhysicalResidencyCounterSnapshot,
    residency_after_point: PhysicalResidencyCounterSnapshot,
    residency_after_scan: PhysicalResidencyCounterSnapshot,
    point: RecordReadObservation,
    scan: RecordScanCounterSnapshot,
    signal_clock_advance: u64,
    signal_invalidation_delta: u64,
}

impl LiveScaleMeasurements {
    fn into_observation(self, walk: &OfflineDurableManifestWalk) -> ScaleObservation {
        ScaleObservation {
            record_count: walk.placements().len() as u16,
            routing_level: walk.routing_level().unwrap(),
            whole_blocks: walk.manifest_blocks(),
            point_blocks: self.point.manifest_blocks(),
            point_pages: self.point.touched_pages(),
            point_comparisons: self.point.manifest_comparisons(),
            point_work: self.point.physical_work_count(),
            point_faults: self
                .residency_after_point
                .faults()
                .saturating_sub(self.residency_before_point.faults()),
            point_media_reads: self
                .media_after_point
                .completed_operations_for(MediaOperationRole::PositionedRead)
                .saturating_sub(
                    self.media_before_point
                        .completed_operations_for(MediaOperationRole::PositionedRead),
                ),
            point_media_bytes: self
                .media_after_point
                .completed_bytes_for(MediaOperationRole::PositionedRead)
                .saturating_sub(
                    self.media_before_point
                        .completed_bytes_for(MediaOperationRole::PositionedRead),
                ),
            point_manifest_bytes: self.point.manifest_bytes(),
            scan_records: self.scan.records(),
            scan_payload_bytes: self.scan.payload_bytes(),
            scan_blocks: self.scan.manifest_blocks(),
            scan_frames: self.scan.frames_traversed(),
            scan_work: self.scan.physical_work_count(),
            scan_faults: self
                .residency_after_scan
                .faults()
                .saturating_sub(self.residency_after_point.faults()),
            scan_media_reads: self
                .media_after_scan
                .completed_operations_for(MediaOperationRole::PositionedRead)
                .saturating_sub(
                    self.media_after_point
                        .completed_operations_for(MediaOperationRole::PositionedRead),
                ),
            scan_media_bytes: self
                .media_after_scan
                .completed_bytes_for(MediaOperationRole::PositionedRead)
                .saturating_sub(
                    self.media_after_point
                        .completed_bytes_for(MediaOperationRole::PositionedRead),
                ),
            scan_manifest_bytes: self.scan.manifest_bytes(),
            signal_clock_advance: self.signal_clock_advance,
            signal_invalidation_delta: self.signal_invalidation_delta,
            point_allocations: 0,
            scan_allocations: 0,
            invalid_worlds: 0,
        }
    }
}

pub(super) fn observe_scale_world(record_count: u16) -> ScaleObservation {
    let seeded = seed_scale_world(record_count);
    let changed_access = super::super::scale_support::access(seeded.format, 7);
    let mut runtime = observe_runtime_world(&seeded, changed_access);
    let (allocation_process, point_allocations, scan_allocations) =
        observe_allocation_probe(&seeded);
    let invalid = super::super::scale_invalid_worlds::exercise(
        &seeded.root,
        seeded.format,
        seeded.placement,
        changed_access,
    );
    assert_invalid_worlds(&invalid);
    runtime.observation.point_allocations = point_allocations;
    runtime.observation.scan_allocations = scan_allocations;
    runtime.observation.invalid_worlds = invalid.count();
    assert!(
        runtime.observation.point_blocks <= runtime.observation.whole_blocks,
        "C5_PREDICATE:locate-open-scale: point lookup cannot read more routing blocks than the whole manifest"
    );
    let processes = [runtime.process, allocation_process];
    super::evidence::emit(ScaleCourtroomEvidence {
        record_count,
        last: seeded.last,
        locator: seeded.locator,
        walk: &runtime.walk,
        processes: &processes,
        observation: runtime.observation,
        invalid: &invalid,
    });
    runtime.observation
}

fn seed_scale_world(record_count: u16) -> SeededScaleWorld {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let format = super::super::scale_support::format();
    let placement = super::super::scale_support::placement(format, 2, 2, 50);
    let initial_access = super::super::scale_support::access(format, 17);
    let serving = super::super::success(initialize_record_store!(
        super::super::media(&root),
        |durability| PhysicalRecordInitialization::new(
            format,
            placement,
            initial_access,
            durability
        ),
    ));
    let payloads = (0..record_count)
        .map(|ordinal| vec![(ordinal % 251) as u8; 100])
        .collect::<Vec<_>>();
    let published = super::super::durable_publication::publish_single(
        &serving,
        placement,
        PhysicalMutationIdempotencyMaterial::new([record_count as u8; 32]),
        RecordAppendBatch::try_from_iter(payloads.iter()).unwrap(),
    );
    let last = published.settled_members()[0]
        .record_id(usize::from(record_count - 1))
        .unwrap();
    let locator = ExternalPhysicalRecordLocator::new(serving.store_identity(), last);
    serving.close();
    SeededScaleWorld {
        _parent: parent,
        root,
        format,
        placement,
        last,
        locator,
    }
}

fn observe_runtime_world(
    seeded: &SeededScaleWorld,
    changed_access: AdmittedRecordAccessPolicy,
) -> RuntimeScaleWorld {
    let serving = super::super::success(open_record_store!(
        super::super::media(&seeded.root),
        |durability| PhysicalRecordOpen::new(seeded.format, changed_access, durability)
    ));
    assert_eq!(
        serving
            .records()
            .readmit_locator(seeded.locator)
            .into_result()
            .unwrap(),
        seeded.last
    );
    let measurements = observe_live_reads(&serving, seeded.last);
    let walk = worth_store_offline_verifier::walk_current_durable_record_manifest(
        &seeded.root,
        seeded.format.declaration(),
    )
    .unwrap();
    super::super::scale_support::assert_canonical_parity(&serving, &walk);
    let process = ScenarioProcessEvidence::current_runtime("scale-reopener", &serving);
    serving.close();
    RuntimeScaleWorld {
        observation: measurements.into_observation(&walk),
        walk,
        process,
    }
}

fn observe_live_reads(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    last: PhysicalRecordId,
) -> LiveScaleMeasurements {
    let media_before_point = serving.media_counters();
    let residency_before_point = serving.residency_observation().counters();
    let signal_before = serving.physical_signal_observation().unwrap();
    let session = serving
        .records()
        .open(
            last,
            RecordReadLimits::new(RecordByteLimit::new(100).unwrap()),
        )
        .unwrap();
    let (bytes, point) = super::super::read_record(session, 100);
    assert_eq!(
        bytes,
        vec![(last.ordinal().saturating_sub(1) % 251) as u8; 100]
    );
    let media_after_point = serving.media_counters();
    let residency_after_point = serving.residency_observation().counters();
    let scan = super::super::scale_support::complete_scan(serving, 7, 16_384);
    let media_after_scan = serving.media_counters();
    let residency_after_scan = serving.residency_observation().counters();
    await_signal_cleanup(serving);
    let signal_after = serving.physical_signal_observation().unwrap();
    assert_eq!(signal_after.active_locality_count(), 0);
    assert_eq!(signal_after.active_in_flight_count(), 0);
    LiveScaleMeasurements {
        media_before_point,
        media_after_point,
        media_after_scan,
        residency_before_point,
        residency_after_point,
        residency_after_scan,
        point,
        scan,
        signal_clock_advance: signal_after
            .clock()
            .last_advance_ordinal()
            .saturating_sub(signal_before.clock().last_advance_ordinal()),
        signal_invalidation_delta: signal_after
            .aspect_invalidation_count()
            .saturating_sub(signal_before.aspect_invalidation_count()),
    }
}

fn observe_allocation_probe(seeded: &SeededScaleWorld) -> (ScenarioProcessEvidence, usize, usize) {
    let stdout = super::super::child_process::run_child(
        "scale_allocation_reader",
        &seeded.root,
        Some(&super::super::child_process::hex(&seeded.locator.encode())),
    );
    let process = ScenarioProcessEvidence::parse_child(&stdout, "scale-allocation-probe");
    let (point, scan) = scale_allocations(&stdout);
    (process, point, scan)
}

fn assert_invalid_worlds(invalid: &super::super::scale_invalid_worlds::InvalidScaleWorlds) {
    assert!(
        invalid.missing_catalog_refused,
        "C5_PREDICATE:current-truth missing catalog was guessed"
    );
    assert!(
        invalid.checksum_damage_refused,
        "checksum damage was admitted"
    );
    assert!(
        invalid.stale_manifest_refused,
        "stale manifest was admitted"
    );
    assert!(invalid.format_drift_refused, "format drift was admitted");
    assert!(
        invalid.residue_excluded,
        "unpublished residue was treated as current"
    );
    assert_eq!(invalid.count(), 5);
}

fn await_signal_cleanup(serving: &worth_store::physical_runtime::ServingPhysicalRuntime) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        let observation = serving.physical_signal_observation().unwrap();
        if observation.active_locality_count() == 0 && observation.active_in_flight_count() == 0 {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "scale read retained Signal state: locality={}, in_flight={}",
            observation.active_locality_count(),
            observation.active_in_flight_count(),
        );
        std::thread::yield_now();
    }
}

fn scale_allocations(stdout: &str) -> (usize, usize) {
    let fields = stdout
        .lines()
        .find_map(|line| line.strip_prefix("C5_SCALE_ALLOC "))
        .expect("the isolated allocation probe must emit its exact observation")
        .split_whitespace()
        .collect::<Vec<_>>();
    assert_eq!(fields.len(), 2);
    (fields[0].parse().unwrap(), fields[1].parse().unwrap())
}
