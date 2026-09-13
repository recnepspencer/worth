use super::*;
use worth_store::physical_runtime::{
    ManagedPhysicalIntegrityScrubProgress as Progress,
    ManagedPhysicalIntegrityScrubRequest as Request, PhysicalIntegrityScrubTarget as Target,
};
use worth_store_physical_format::{PhysicalArtifactReadTarget, PhysicalCheckpointIdentity};
use worth_store_physical_integrity::{
    CheckpointStreamHeaderScopeIdentity, IndeterminatePhysicalIntegrityCause,
    PhysicalArtifactScope, PhysicalByteRange, PhysicalIntegrityObservationOutcome as Observation,
    PhysicalIntegrityRejection as Rejection,
};

fn publish(serving: &ServingPhysicalRuntime, key: u8) -> PhysicalCheckpointIdentity {
    match start(serving, checkpoint_request(key)).wait() {
        PhysicalCheckpointOutcome::Completed(completed) => completed.basis().identity(),
        outcome => panic!("real checkpoint publication: {outcome:?}"),
    }
}

fn header(identity: PhysicalCheckpointIdentity) -> Target {
    Target::new(
        PhysicalArtifactReadTarget::Checkpoint(identity),
        PhysicalArtifactScope::checkpoint_stream_header(
            CheckpointStreamHeaderScopeIdentity::known(identity),
            PhysicalByteRange::new(0, 164).unwrap(),
        ),
    )
    .unwrap()
}

#[test]
fn scrub_checkpoint_publication_between_windows_is_source_change_not_damage() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_with_durable_wal(&root, 131);
    let first = publish(&serving, 131);
    let length = std::fs::metadata(root.join("families/checkpoint.current"))
        .unwrap()
        .len();
    let footer = Target::new(
        PhysicalArtifactReadTarget::Checkpoint(first),
        PhysicalArtifactScope::checkpoint_footer(
            first,
            PhysicalByteRange::new(length - 156, 156).unwrap(),
        ),
    )
    .unwrap();
    let request = Request::new(
        serving.store_identity(),
        [header(first), footer],
        4096,
        8192,
        Duration::from_secs(30),
    )
    .unwrap();
    let mut handle = serving.start_physical_integrity_scrub(request).unwrap();
    assert!(
        matches!(handle.next_window(), Progress::WindowInspected(observation) if matches!(observation.outcome, Observation::Intact(_)))
    );
    let second = publish(&serving, 132);
    assert_ne!(first, second);
    let progress = handle.next_window();
    assert!(
        matches!(progress, Progress::WindowInspected(observation)
        if matches!(observation.outcome, Observation::Rejected(Rejection::Indeterminate(posture)) if posture.cause() == IndeterminatePhysicalIntegrityCause::SourceChangedDuringInspection)
        && observation.quarantine.is_none() && observation.validation_counters.inspected_frames() == 0),
        "{progress:?}"
    );
    assert!(
        matches!(handle.next_window(), Progress::Indeterminate(counters) if counters.damaged_windows == 0)
    );
    serving.close();
}

#[test]
fn scrub_stale_checkpoint_request_is_unknown_without_quarantine() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_with_durable_wal(&root, 133);
    let first = publish(&serving, 133);
    publish(&serving, 134);
    let request = Request::new(
        serving.store_identity(),
        [header(first)],
        4096,
        4096,
        Duration::from_secs(30),
    )
    .unwrap();
    let mut handle = serving.start_physical_integrity_scrub(request).unwrap();
    assert!(
        matches!(handle.next_window(), Progress::WindowInspected(observation)
        if matches!(observation.outcome, Observation::Rejected(Rejection::Unknown(_))) && observation.quarantine.is_none())
    );
    serving.close();
}

#[test]
fn scrub_stale_checkpoint_body_without_header_cannot_manufacture_damage() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_with_durable_wal(&root, 135);
    let first = publish(&serving, 135);
    publish(&serving, 136);
    // This fixture has no dirty pages; the first record after its 164-byte
    // header is the independently specified 36-byte compaction record.
    let compaction = Target::new(
        PhysicalArtifactReadTarget::Checkpoint(first),
        PhysicalArtifactScope::checkpoint_binding_compaction(
            first,
            PhysicalByteRange::new(164, 36).unwrap(),
        ),
    )
    .unwrap();
    let request = Request::new(
        serving.store_identity(),
        [compaction],
        4096,
        4096,
        Duration::from_secs(30),
    )
    .unwrap();
    let mut handle = serving.start_physical_integrity_scrub(request).unwrap();
    assert!(
        matches!(handle.next_window(), Progress::WindowInspected(observation)
        if matches!(observation.outcome, Observation::Rejected(Rejection::Unknown(_))) && observation.quarantine.is_none())
    );
    serving.close();
}
