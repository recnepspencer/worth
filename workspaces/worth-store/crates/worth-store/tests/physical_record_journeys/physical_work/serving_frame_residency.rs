use worth_store::physical_runtime::{
    certification::CertificationPhysicalExecutionCheckpoint, CertificationFrameReadFailure,
    PhysicalOperationAllocationScope, PhysicalRecordInitialization, PhysicalRecordOpen,
    PhysicalRecordResidencyFailureKind, PhysicalRecordResidencyPolicy, PhysicalSpeculativeWorkKind,
    RecordAppendBatch, RecordByteLimit, RecordReadDenial, RecordReadLimits,
};
use worth_store_buffer_pool::PhysicalResidencyDenial;
use worth_store_physical_backend::{ArtifactTreeFailureKind, MediaOperationRole};
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use super::{configuration, media, success};
use crate::durable_publication;

const FRAME_BYTES: u32 = 8;

#[test]
fn pins_distinguish_faults_hits_overpin_and_refault_without_another_runtime() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("pin-inheritance");
    let (format, placement, access) = configuration();
    let seeded = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    assert!(!seeded.close().residency().requires_inspection());
    let policy = admitted_policy(format);
    let serving = success(open_record_store!(media(&root), |durability| {
        PhysicalRecordOpen::new(format, access, durability).with_residency_policy(policy)
    },));
    serving
        .certification_physical_residency()
        .drain_unpinned_clean_frames();
    let residency = serving.certification_physical_residency();

    let first_coordinate = coordinate(0);
    let reads_before = positioned_reads(&serving);
    let first = residency.pin_exact(first_coordinate).unwrap();
    assert_eq!(first.coordinate(), first_coordinate);
    assert_eq!(first.physical_work_count(), 1);
    assert!(first.first_physical_work().is_some());
    assert_eq!(positioned_reads(&serving), reads_before + 1);

    let hot = residency.pin_exact(first_coordinate).unwrap();
    assert_eq!(hot.physical_work_count(), 0);
    assert_eq!(positioned_reads(&serving), reads_before + 1);
    assert_eq!(residency.counters().pinned_frames(), 1);
    assert_eq!(residency.counters().pin_leases(), 2);
    assert!(matches!(
        residency.pin_exact(first_coordinate),
        Err(CertificationFrameReadFailure::Residency(
            PhysicalResidencyDenial::Pressure(pressure),
        ))
            if pressure.dimension()
                == worth_store::physical_runtime::PhysicalResidencyDimension::PinLeases
                && pressure.scope()
                    == worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundRead
                && pressure.requested() == 1
                && pressure.current() == 2
                && pressure.limit() == 2
    ));

    drop(hot);
    drop(first);
    assert_eq!(residency.counters().pinned_frames(), 0);
    assert_eq!(residency.counters().pin_leases(), 0);

    for ordinal in 1..=8 {
        pin_and_release(&residency, coordinate(ordinal * u64::from(FRAME_BYTES)));
    }
    let after_pressure = residency.counters();
    assert!(after_pressure.evictions() > 0);
    let reads_before_refault = positioned_reads(&serving);
    let refault = residency.pin_exact(first_coordinate).unwrap();
    assert_eq!(refault.physical_work_count(), 1);
    assert_eq!(positioned_reads(&serving), reads_before_refault + 1);
    assert!(residency.counters().peak_resident_bytes() <= policy.resident_bytes());
    drop(refault);

    let shutdown = serving.close();
    assert!(!shutdown.residency().requires_inspection());
}

#[test]
fn overlapping_pin_coalesces_without_second_media_work_or_signal_authority() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("coalesced-pin");
    let (format, placement, access) = configuration();
    let seeded = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    assert!(!seeded.close().residency().requires_inspection());
    let serving = success(open_record_store!(media(&root), |durability| {
        PhysicalRecordOpen::new(format, access, durability)
            .with_residency_policy(admitted_policy(format))
    },));
    serving
        .certification_physical_residency()
        .drain_unpinned_clean_frames();
    let residency = serving.certification_physical_residency();
    let coordinate = coordinate(0);
    let media_before = serving.media_counters();
    let residency_before = residency.counters();
    let gate = serving.certification_pause_physical_execution_at(
        CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let owner_residency = residency.clone();
    let owner = std::thread::spawn(move || owner_residency.pin_exact(coordinate).unwrap());
    assert!(
        gate.await_arrival(),
        "the sole source read never reached dispatch"
    );

    let work_before_waiter = serving.physical_work_counters();
    let signal_before_waiter = serving
        .physical_signal_observation()
        .unwrap()
        .aspect_invalidation_count();
    let waiter_residency = residency.clone();
    let waiter = std::thread::spawn(move || waiter_residency.pin_exact(coordinate).unwrap());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while residency.counters().coalesced_waiters() == 0 {
        assert!(
            std::time::Instant::now() < deadline,
            "the overlapping pin never attached to the loading identity"
        );
        std::thread::yield_now();
    }

    assert_eq!(serving.media_counters(), media_before);
    assert_eq!(serving.physical_work_counters(), work_before_waiter);
    assert_eq!(
        serving
            .physical_signal_observation()
            .unwrap()
            .aspect_invalidation_count(),
        signal_before_waiter
    );
    gate.release();
    let owner = owner.join().unwrap();
    let waiter = waiter.join().unwrap();

    assert_eq!(owner.bytes(), waiter.bytes());
    assert_eq!(owner.physical_work_count(), 1);
    assert_eq!(waiter.physical_work_count(), 0);
    assert_eq!(
        positioned_reads(&serving),
        media_before.attempts_for(MediaOperationRole::PositionedRead) + 1
    );
    let counters = residency.counters();
    assert_eq!(counters.faults(), residency_before.faults() + 1);
    assert_eq!(
        counters.coalesced_waiters(),
        residency_before.coalesced_waiters() + 1
    );
    assert_eq!(counters.source_loads(), residency_before.source_loads() + 1);
    drop((owner, waiter));
    assert!(!serving.close().residency().requires_inspection());
}

#[test]
fn coalesced_transient_read_preserves_terminal_truth_and_store_health() {
    const PAYLOAD: &[u8] = b"coalesced transient frame fault";

    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("coalesced-transient-read");
    let (_, placement, _) = configuration();
    let initial = super::serving_from_initialization(&root);
    let publication = durable_publication::publish_single(
        &initial,
        placement,
        durable_publication::certification_material("serving-frame-coalesced-transient", 1),
        RecordAppendBatch::try_from_iter([PAYLOAD]).unwrap(),
    );
    let record = publication.settled_members()[0].record_id(0).unwrap();
    assert!(!initial.close().residency().requires_inspection());

    let calibration = super::super::serving_from_open(&root);
    let bootstrap_reads = positioned_reads(&calibration);
    assert!(!calibration.close().residency().requires_inspection());
    let (serving, gate) =
        super::fault_fixture::serving_from_open_with_paused_positioned_read_failure(
            &root,
            bootstrap_reads + 1,
        );
    serving
        .certification_physical_residency()
        .drain_unpinned_clean_frames();

    let limits = RecordReadLimits::new(RecordByteLimit::new(PAYLOAD.len() as u32).unwrap());
    let media_before = serving.media_counters();
    let residency = serving.certification_physical_residency();
    let residency_before = residency.counters();
    let invalidations_before = serving
        .physical_signal_observation()
        .unwrap()
        .aspect_invalidation_count();
    let owner_reader = serving.records().expect("read protection admission");
    let owner = std::thread::spawn(move || match owner_reader.open(record, limits) {
        Err(error) => error,
        Ok(_) => panic!("the scheduled owner fault unexpectedly succeeded"),
    });
    gate.wait_until_reached();

    let waiter_reader = serving.records().expect("read protection admission");
    let waiter = std::thread::spawn(move || match waiter_reader.open(record, limits) {
        Err(error) => error,
        Ok(_) => panic!("the coalesced waiter unexpectedly succeeded"),
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while residency.counters().coalesced_waiters() == residency_before.coalesced_waiters() {
        assert!(
            std::time::Instant::now() < deadline,
            "the public read waiter never joined the loading identity"
        );
        std::thread::yield_now();
    }
    gate.release();

    let owner = owner.join().unwrap();
    let waiter = waiter.join().unwrap();
    assert!(matches!(
        owner.denial(),
        RecordReadDenial::BackendUnavailable(failure)
            if failure.kind() == ArtifactTreeFailureKind::DeniedBeforeEffect
    ));
    assert_eq!(owner.observation().physical_work_count(), 2);
    assert_ne!(
        owner.observation().first_physical_work(),
        owner.observation().last_physical_work(),
        "bounded length discovery and exact range read retain distinct work identities"
    );
    let terminal = match waiter.denial() {
        RecordReadDenial::ResidencyUnavailable(failure)
            if failure.kind() == PhysicalRecordResidencyFailureKind::FrameLoadTerminated =>
        {
            failure
                .frame_load_terminal()
                .expect("coalesced failure carries its lower terminal")
        }
        denial => panic!("coalesced transient fault was misclassified as {denial:?}"),
    };
    assert_eq!(waiter.observation().physical_work_count(), 0);
    assert!(terminal.identity().ordinal() > 0);

    let residency_after = residency.counters();
    let media_after = serving.media_counters();
    assert_eq!(
        media_after.attempts_for(MediaOperationRole::ReadMetadata),
        media_before.attempts_for(MediaOperationRole::ReadMetadata) + 2,
        "one bounded length discovery and one exact-read bounds probe reach metadata"
    );
    assert_eq!(
        media_after.attempts_for(MediaOperationRole::PositionedRead),
        media_before.attempts_for(MediaOperationRole::PositionedRead) + 1
    );
    assert_eq!(residency_after.faults(), residency_before.faults() + 1);
    assert_eq!(
        residency_after.coalesced_waiters(),
        residency_before.coalesced_waiters() + 1
    );
    assert_eq!(
        residency_after.source_loads(),
        residency_before.source_loads() + 1
    );
    assert_eq!(
        serving
            .physical_signal_observation()
            .unwrap()
            .aspect_invalidation_count(),
        invalidations_before,
        "a coalesced transient denial must not invent projection damage"
    );

    let mut retry = serving
        .records()
        .expect("read protection admission")
        .open(record, limits)
        .expect("coalesced transient denial must leave Store health usable");
    let mut bytes = vec![0_u8; PAYLOAD.len()];
    assert_eq!(retry.read_next(&mut bytes).unwrap(), PAYLOAD.len());
    assert_eq!(bytes, PAYLOAD);
    drop(retry);

    let hot_media_before = serving.media_counters();
    let hot = serving
        .records()
        .expect("read protection admission")
        .open(record, limits)
        .expect("resolved bounded frame must remain hot");
    assert_eq!(
        hot.observation().physical_work_count(),
        0,
        "the resident frame carries segment-completeness proof across the hot open"
    );
    assert_eq!(hot.observation().first_physical_work(), None);
    assert_eq!(hot.observation().last_physical_work(), None);
    drop(hot);
    let hot_media_after = serving.media_counters();
    assert_eq!(
        hot_media_after.attempts_for(MediaOperationRole::ReadMetadata),
        hot_media_before.attempts_for(MediaOperationRole::ReadMetadata),
        "the bounded hit must add neither discovery nor segment-completeness media work"
    );
    assert_eq!(
        hot_media_after.attempts_for(MediaOperationRole::PositionedRead),
        hot_media_before.attempts_for(MediaOperationRole::PositionedRead)
    );
    assert!(!serving.close().residency().requires_inspection());
}

fn admitted_policy(
    format: worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
) -> worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy {
    use PhysicalOperationAllocationScope as Scope;
    use PhysicalSpeculativeWorkKind as Speculation;

    let resident_bytes = 64 * 1024;
    let metadata_bytes = 16 * 1024;
    let operation_bytes = 4 * 1024 * 1024;
    PhysicalRecordResidencyPolicy::builder()
        .total_bytes(nonzero_bytes(
            operation_bytes + metadata_bytes + (2 * resident_bytes),
        ))
        .resident_bytes(nonzero_bytes(resident_bytes))
        .metadata_bytes(nonzero_bytes(metadata_bytes))
        .frame_entries(nonzero_count(8))
        .pinned_frames(nonzero_count(8))
        .pin_leases(nonzero_count(2))
        .dirty_frames(nonzero_count(2))
        .dirty_replacement_bytes(nonzero_bytes(resident_bytes))
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

fn nonzero_bytes(value: u64) -> std::num::NonZeroU64 {
    std::num::NonZeroU64::new(value).unwrap()
}

fn nonzero_count(value: u32) -> std::num::NonZeroU32 {
    std::num::NonZeroU32::new(value).unwrap()
}

fn coordinate(offset: u64) -> RecordFrameCoordinate {
    RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, offset, FRAME_BYTES).unwrap()
}

fn pin_and_release(
    residency: &worth_store::physical_runtime::PhysicalResidencyCertification,
    coordinate: RecordFrameCoordinate,
) {
    let frame = residency.pin_exact(coordinate).unwrap();
    assert_eq!(frame.physical_work_count(), 1);
    drop(frame);
}

fn positioned_reads(serving: &worth_store::physical_runtime::ServingPhysicalRuntime) -> u64 {
    serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedRead)
}
