use worth_proof::NonEmpty;
use worth_store::physical_runtime::{
    certification::{CertificationDurableMutationInput, CertificationPhysicalMutationCheckpoint},
    PhysicalDataDispatchFailureCause, PhysicalDataDispatchOutcome,
    PhysicalManifestCapacityTransition, PhysicalMutationIdempotencyMaterial,
    PhysicalRecordInitialization, PhysicalRecordOpen, PhysicalResidencyDimension,
    PhysicalResidencyRetryPosture, RecordAppendBatch, RecordByteLimit, RecordReadLimits,
};
use worth_store_physical_backend::MediaOperationRole;
use worth_store_physical_format::RecordFrameCoordinate;

use super::{configuration, media, success, two_append_writebehind_policy};
use crate::durable_publication;

#[test]
fn arena_pin_pressure_repeatedly_preserves_prefix_and_neighbour_without_deletion() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("arena-prefix-pressure");
    let (format, placement, access) = configuration();
    let payload = vec![0x71; format.declaration().page_size().bytes() as usize * 3];
    let seeded = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let neighbour = durable_publication::publish_single(
        &seeded,
        placement,
        PhysicalMutationIdempotencyMaterial::new([0xB1; 32]),
        RecordAppendBatch::try_from_iter([payload.as_slice()]).unwrap(),
    );
    let record = neighbour.settled_members()[0].record_id(0).unwrap();
    let limits = RecordReadLimits::new(RecordByteLimit::new(payload.len() as u32).unwrap());
    let coordinate = {
        let reader = seeded.records().unwrap();
        let mut stream = reader.open(record, limits).unwrap();
        let chunk = stream.next_chunk().unwrap().unwrap();
        chunk.basis().frame_coordinate()
    };
    assert!(matches!(
        coordinate.artifact(),
        worth_store_physical_format::RecordArtifactFile::ExtentArena { .. }
    ));
    assert!(!seeded.close().residency().requires_inspection());
    let arena_path = root
        .join("families/records/arenas")
        .join(coordinate.artifact().file_name());
    let neighbour_bytes = std::fs::read(&arena_path).unwrap();
    let original_arena = std::fs::File::open(&arena_path).unwrap();
    let serving = success(open_record_store!(media(&root), |durability| {
        PhysicalRecordOpen::new(format, access, durability)
            .with_residency_policy(two_append_writebehind_policy(format))
    },));
    let (group, durable) = serving.certification_prepare_wal_durable_group(
        placement,
        PhysicalManifestCapacityTransition::PreserveCurrent,
        NonEmpty::new(
            CertificationDurableMutationInput::new(
                PhysicalMutationIdempotencyMaterial::new([0xB2; 32]),
                RecordAppendBatch::try_from_iter([payload.as_slice()]).unwrap(),
            ),
            vec![],
        ),
    );
    let gate = serving.certification_pause_physical_mutation_at(
        CertificationPhysicalMutationCheckpoint::AfterDataFrameSettlement,
    );
    let submission = serving.certification_record_submission();
    let dispatch = std::thread::spawn(move || {
        submission.dispatch_wal_durable_data(durable.into_vec().pop().unwrap())
    });
    if !gate.await_arrival() {
        gate.release();
        panic!("arena dispatch must settle its first frame before applying pin pressure");
    }
    // Actual resident neighbour ranges consume the same pin budget as ordinary
    // reads. The checkpoint only schedules contention; it grants no capacity.
    let residency = serving.certification_physical_residency();
    let held = (0..4)
        .map(|ordinal| {
            let range = RecordFrameCoordinate::new(
                coordinate.artifact(),
                coordinate.offset() + ordinal * 8,
                8,
            )
            .unwrap();
            residency.pin_exact(range)
        })
        .collect::<Result<Vec<_>, _>>();
    if held.is_err() {
        gate.release();
    }
    let held = held.expect("real neighbour pins must consume the configured four-frame budget");
    assert_eq!(residency.counters().pinned_frames(), 4);
    let before_denial = serving.media_counters();
    gate.release();
    let suspended = match dispatch.join().unwrap() {
        PhysicalDataDispatchOutcome::Suspended(suspended) => suspended,
        _ => panic!("pressure after a settled arena frame must suspend, not lose its prefix"),
    };
    assert_eq!(suspended.completed_effects().len(), 1);
    let first = suspended.completed_effects()[0].clone();
    assert_eq!(first.coordinate().artifact(), coordinate.artifact());
    assert!(first.coordinate().offset() >= neighbour_bytes.len() as u64);
    let PhysicalDataDispatchFailureCause::PhysicalPressure(pressure) = suspended.cause() else {
        panic!("the denial must be actual residency contention");
    };
    assert_eq!(
        pressure.dimension(),
        PhysicalResidencyDimension::PinnedFrames
    );
    assert_eq!(pressure.requested(), 1);
    assert_eq!(pressure.admitted(), 4);
    assert_eq!(pressure.limit(), 4);
    assert_eq!(
        pressure.retry_posture(),
        PhysicalResidencyRetryPosture::AfterLeaseRelease
    );
    assert!(!pressure.effect_may_have_started());
    assert_eq!(serving.media_counters(), before_denial);
    let mut durable = suspended.into_durable();
    for _ in 0..2 {
        let (next, pressure) = match serving
            .certification_record_submission()
            .dispatch_wal_durable_data(durable)
        {
            PhysicalDataDispatchOutcome::Suspended(retry) => {
                assert_eq!(retry.completed_effects(), std::slice::from_ref(&first));
                let PhysicalDataDispatchFailureCause::PhysicalPressure(pressure) = retry.cause()
                else {
                    panic!("every retry must be denied by the retained real pins");
                };
                let pressure = *pressure;
                (retry.into_durable(), pressure)
            }
            PhysicalDataDispatchOutcome::NotStarted {
                durable,
                cause: PhysicalDataDispatchFailureCause::PhysicalPressure(pressure),
            } => (durable, pressure),
            PhysicalDataDispatchOutcome::NotStarted { cause, .. } => {
                panic!("unexpected repeat denial: {cause:?}")
            }
            PhysicalDataDispatchOutcome::Indeterminate(failure) => {
                panic!("repeat uncertain: {:?}", failure.cause())
            }
            PhysicalDataDispatchOutcome::Dispatched(_) => {
                panic!("unreleased pins must deny every retry before its next frame effect")
            }
        };
        assert_eq!(
            pressure.dimension(),
            PhysicalResidencyDimension::PinnedFrames
        );
        assert_eq!(pressure.admitted(), 4);
        assert_eq!(pressure.limit(), 4);
        assert_eq!(residency.counters().pinned_frames(), 4);
        assert_eq!(serving.media_counters(), before_denial);
        durable = next;
    }
    drop(held);
    assert_eq!(residency.counters().pinned_frames(), 0);
    let before_resume = serving.media_counters();
    let dispatched = match serving
        .certification_record_submission()
        .dispatch_wal_durable_data(durable)
    {
        PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
        _ => panic!("releasing real pins must permit the remaining arena frames"),
    };
    assert_eq!(dispatched.effects()[0], first);
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite)
            - before_resume.attempts_for(MediaOperationRole::PositionedWrite),
        dispatched.effects().len() as u64 - 1,
        "the completed prefix must never be written again",
    );
    // Successful work cleans its own recovery-obligation files, so a global
    // deletion count is not an arena-deletion oracle. The original open file
    // must grow with the resumed suffix and still match the named arena.
    assert_eq!(
        original_arena.metadata().unwrap().len(),
        std::fs::metadata(&arena_path).unwrap().len()
    );
    assert!(original_arena.metadata().unwrap().len() > neighbour_bytes.len() as u64);
    assert_eq!(
        &std::fs::read(&arena_path).unwrap()[..neighbour_bytes.len()],
        &neighbour_bytes
    );
    let completed =
        serving.certification_complete_dispatched_group(group, NonEmpty::new(dispatched, vec![]));
    assert_eq!(completed.settled_members().len(), 1);
    let reader = serving.records().unwrap();
    let mut stream = reader.open(record, limits).unwrap();
    let mut readback = Vec::new();
    while let Some(chunk) = stream.next_chunk().unwrap() {
        readback.extend_from_slice(chunk.bytes());
    }
    assert_eq!(readback, payload);
    drop(stream);
    drop(reader);
    assert!(!serving.close().residency().requires_inspection());
}
