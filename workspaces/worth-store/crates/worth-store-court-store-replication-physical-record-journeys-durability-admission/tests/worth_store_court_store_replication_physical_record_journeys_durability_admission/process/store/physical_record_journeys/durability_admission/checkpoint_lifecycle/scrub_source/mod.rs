#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::checkpoint_lifecycle::scrub_source::*;
use worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_lifecycle::serving_with_durable_wal;

#[test]
fn scrub_checkpoint_publication_between_windows_is_source_change_not_damage() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_with_durable_wal(&root, 131);
    let first = publish(&serving, 131);
    let length = std::fs::metadata(root.join("families/checkpoint.current"))
        .unwrap()
        .len();
    let footer = worth_store::physical_runtime::PhysicalIntegrityScrubTarget::new(
        worth_store_physical_format::PhysicalArtifactReadTarget::Checkpoint(first),
        worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_footer(
            first,
            worth_store_physical_integrity::PhysicalByteRange::new(length - 156, 156).unwrap(),
        ),
    )
    .unwrap();
    let request = worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest::new(
        serving.store_identity(),
        [header(first), footer],
        4096,
        8192,
        std::time::Duration::from_secs(30),
    )
    .unwrap();
    let mut handle = serving.start_physical_integrity_scrub(request).unwrap();
    assert!(
        matches!(handle.next_window(), worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(observation) if matches!(observation.outcome, worth_store_physical_integrity::PhysicalIntegrityObservationOutcome::Intact(_)))
    );
    let second = publish(&serving, 132);
    assert_ne!(first, second);
    let progress = handle.next_window();
    assert!(
        matches!(progress, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(observation)
        if matches!(observation.outcome, worth_store_physical_integrity::PhysicalIntegrityObservationOutcome::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Indeterminate(posture)) if posture.cause() == worth_store_physical_integrity::IndeterminatePhysicalIntegrityCause::SourceChangedDuringInspection)
        && observation.quarantine.is_none() && observation.validation_counters.inspected_frames() == 0),
        "{progress:?}"
    );
    assert!(
        matches!(handle.next_window(), worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Indeterminate(counters) if counters.damaged_windows == 0)
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
    let request = worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest::new(
        serving.store_identity(),
        [header(first)],
        4096,
        4096,
        std::time::Duration::from_secs(30),
    )
    .unwrap();
    let mut handle = serving.start_physical_integrity_scrub(request).unwrap();
    assert!(
        matches!(handle.next_window(), worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(observation)
        if matches!(observation.outcome, worth_store_physical_integrity::PhysicalIntegrityObservationOutcome::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Unknown(_))) && observation.quarantine.is_none())
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
    let compaction = worth_store::physical_runtime::PhysicalIntegrityScrubTarget::new(
        worth_store_physical_format::PhysicalArtifactReadTarget::Checkpoint(first),
        worth_store_physical_integrity::PhysicalArtifactScope::checkpoint_binding_compaction(
            first,
            worth_store_physical_integrity::PhysicalByteRange::new(164, 36).unwrap(),
        ),
    )
    .unwrap();
    let request = worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest::new(
        serving.store_identity(),
        [compaction],
        4096,
        4096,
        std::time::Duration::from_secs(30),
    )
    .unwrap();
    let mut handle = serving.start_physical_integrity_scrub(request).unwrap();
    assert!(
        matches!(handle.next_window(), worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(observation)
        if matches!(observation.outcome, worth_store_physical_integrity::PhysicalIntegrityObservationOutcome::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Unknown(_))) && observation.quarantine.is_none())
    );
    serving.close();
}
