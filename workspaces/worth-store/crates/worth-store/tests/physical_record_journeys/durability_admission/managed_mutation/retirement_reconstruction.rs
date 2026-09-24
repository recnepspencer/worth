use std::path::Path;
use std::thread;

use worth_store::physical_runtime::{
    certification::CertificationReadRootCaptureStage, ManifestEntryCapacity,
    PhysicalRecordInitialization, PhysicalRecordPlacementPolicy, PhysicalRetirementDenial,
    RecordByteLimit, RecordReadLimits, SegmentPageCount,
};
use worth_store_physical_format::{
    PhysicalCheckpointSource, CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};

use super::super::independent_wal_oracle::{
    produced_retirement_payloads, IndependentRetirementAction,
};
use super::published_segments::segment_names;
use super::selected_segment_rewrite::prepare_rewrite;
use super::*;

#[test]
fn reopened_store_keeps_cleanup_until_retirement_completion() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (format, placement, _) = configuration();
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    completed(prepare(&serving, placement, [81; 32], b"cleanup-source").execute());
    completed(prepare_rewrite(&serving, placement, [82; 32]).execute());
    let before = segment_names(&root);
    serving.certification_stop_after_retirement_delete();
    assert!(serving.retire_displaced_segment().is_err());
    assert!(
        before
            .iter()
            .any(|name| !segment_names(&root).contains(name)),
        "the seam stops after unlink"
    );
    serving.close();
    let serving = crate::serving_from_open(&root);
    assert!(serving.records().is_ok());
    let held = serving.certification_charged_growth_bytes();
    serving.retire_displaced_segment().unwrap();
    let released = serving.certification_charged_growth_bytes();
    assert_eq!(
        held - released,
        page_bytes,
        "completion releases the page kept after delete"
    );
    serving.close();
    let serving = crate::serving_from_open(&root);
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Absent)
    );
    serving.close();
}

#[test]
fn retirement_intent_survives_reopen_before_delete() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    completed(prepare(&serving, placement, [88; 32], b"intent-source").execute());
    completed(prepare_rewrite(&serving, placement, [89; 32]).execute());
    let before = segment_names(&root);
    serving.certification_stop_before_retirement_delete();
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Delete)
    );
    assert_eq!(segment_names(&root), before);
    assert_eq!(
        retirement_actions(&root),
        [IndependentRetirementAction::Intent]
    );
    serving.close();
    let serving = crate::serving_from_open(&root);
    assert!(serving.records().is_ok());
    serving.retire_displaced_segment().unwrap();
    assert!(before
        .iter()
        .any(|name| !segment_names(&root).contains(name)));
    serving.close();
}

#[test]
fn retirement_waits_for_the_scheduler_before_any_wal_byte() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    completed(prepare(&serving, placement, [92; 32], b"scheduler-source").execute());
    completed(prepare_rewrite(&serving, placement, [93; 32]).execute());
    let before = segment_names(&root);
    serving.certification_owe_background_turn();
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Waiting)
    );
    assert_eq!(segment_names(&root), before);
    assert!(produced_retirement_payloads(&root).is_empty());
    serving.certification_release_owed_background_turn();
    serving.retire_displaced_segment().unwrap();
    assert!(before
        .iter()
        .any(|name| !segment_names(&root).contains(name)));
    serving.close();
}

#[test]
fn retirement_namespace_sync_follows_unlink_before_completion() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (format, placement, _) = configuration();
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    completed(prepare(&serving, placement, [90; 32], b"namespace-source").execute());
    completed(prepare_rewrite(&serving, placement, [91; 32]).execute());
    let before = segment_names(&root);
    serving.certification_fail_next_removal_directory_sync();
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Delete)
    );
    assert!(
        before
            .iter()
            .any(|name| !segment_names(&root).contains(name)),
        "unlink happened and the segment directory sync did not complete"
    );
    serving.close();
    let serving = crate::serving_from_open(&root);
    assert!(serving.records().is_ok());
    let held = serving.certification_charged_growth_bytes();
    serving.retire_displaced_segment().unwrap();
    assert_eq!(
        held - serving.certification_charged_growth_bytes(),
        page_bytes
    );
    serving.close();
    let serving = crate::serving_from_open(&root);
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Absent)
    );
    serving.close();
}

#[test]
fn unresolved_publication_blocks_retirement_until_it_settles() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    completed(prepare(&serving, placement, [83; 32], b"unresolved-source").execute());
    completed(prepare_rewrite(&serving, placement, [84; 32]).execute());
    let before = segment_names(&root);
    let gate = serving.certification_pause_physical_mutation_at(
        CertificationPhysicalMutationCheckpoint::BeforeWalAppend,
    );
    let handle = prepare(&serving, placement, [85; 32], b"unresolved-append").start();
    assert!(gate.await_arrival());
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Unresolved)
    );
    assert_eq!(segment_names(&root), before);
    gate.release();
    handle.wait();
    serving.retire_displaced_segment().unwrap();
    let after = segment_names(&root);
    assert!(before.iter().any(|name| !after.contains(name)));
    serving.close();
}

#[test]
fn shared_generation_pages_stay_readable_after_a_tail_rewrite() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, _, access) = configuration();
    let placement = PhysicalRecordPlacementPolicy::builder()
        .segment_pages(SegmentPageCount::new(4).unwrap())
        .extent_threshold(RecordByteLimit::new(16_000).unwrap())
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(format)
        .unwrap();
    let serving = crate::success(initialize_record_store!(
        crate::media(&root),
        |durability| PhysicalRecordInitialization::new(format, placement, access, durability)
    ));
    let first = vec![0x31_u8; 7_500];
    let second = vec![0x32_u8; 7_500];
    let appended =
        completed(prepare_records(&serving, placement, [71; 32], &[&first, &second]).execute());
    let ids: Vec<_> = appended.into_acknowledgment().record_ids().collect();
    completed(prepare_rewrite(&serving, placement, [72; 32]).execute());
    let before = segment_names(&root);
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Absent)
    );
    assert_eq!(segment_names(&root), before);
    let reader = serving.records().unwrap();
    for (id, payload) in ids.iter().zip([&first, &second]) {
        let mut session = reader
            .open(
                *id,
                RecordReadLimits::new(RecordByteLimit::new(payload.len() as u32).unwrap()),
            )
            .unwrap();
        let mut bytes = vec![0_u8; payload.len()];
        let mut filled = 0;
        while filled < bytes.len() {
            let count = session.read_next(&mut bytes[filled..]).unwrap();
            assert!(count > 0);
            filled += count;
        }
        assert_eq!(bytes, *payload);
    }
    serving.close();
    let serving = crate::serving_from_open(&root);
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Absent)
    );
    assert_eq!(segment_names(&root), before);
    let reader = serving.records().unwrap();
    for (id, payload) in ids.iter().zip([&first, &second]) {
        let mut session = reader
            .open(
                *id,
                RecordReadLimits::new(RecordByteLimit::new(payload.len() as u32).unwrap()),
            )
            .unwrap();
        let mut bytes = vec![0_u8; payload.len()];
        let mut filled = 0;
        while filled < bytes.len() {
            let count = session.read_next(&mut bytes[filled..]).unwrap();
            assert!(count > 0);
            filled += count;
        }
        assert_eq!(bytes, *payload);
    }
    serving.close();
}

#[test]
fn observed_source_stays_protected_when_rewrite_waits_on_capture() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    completed(prepare(&serving, placement, [98; 32], b"race-source").execute());
    thread::scope(|threads| {
        let pause = serving.certification_pause_next_read_root_capture(
            CertificationReadRootCaptureStage::AfterObservationBeforeRegistration,
        );
        let capture = threads.spawn(|| serving.records().unwrap());
        assert!(pause.await_arrival());
        let rewrite = threads.spawn(|| prepare_rewrite(&serving, placement, [99; 32]).execute());
        assert!(pause.await_publication_lock_wait());
        pause.release();
        let reader = capture.join().unwrap();
        completed(rewrite.join().unwrap());
        let before = segment_names(&root);
        assert_eq!(
            serving.retire_displaced_segment(),
            Err(PhysicalRetirementDenial::Protected)
        );
        assert_eq!(segment_names(&root), before);
        drop(reader);
        serving.retire_displaced_segment().unwrap();
        assert!(before
            .iter()
            .any(|name| !segment_names(&root).contains(name)));
    });
    serving.close();
}

#[test]
fn retirement_barrier_waits_after_the_frame_is_written() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    completed(prepare(&serving, placement, [96; 32], b"barrier-source").execute());
    completed(prepare_rewrite(&serving, placement, [97; 32]).execute());
    let before = segment_names(&root);
    serving.certification_owe_before_retirement_barrier();
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Waiting)
    );
    assert_eq!(segment_names(&root), before);
    assert_eq!(
        retirement_actions(&root),
        [IndependentRetirementAction::Intent]
    );
    serving.certification_release_owed_background_turn();
    serving.retire_displaced_segment().unwrap();
    assert!(before
        .iter()
        .any(|name| !segment_names(&root).contains(name)));
    serving.close();
}

#[test]
fn retirement_deletes_only_after_a_checkpoint_of_the_successor_root() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    completed(prepare(&serving, placement, [86; 32], b"checkpoint-source").execute());
    completed(prepare_rewrite(&serving, placement, [87; 32]).execute());
    let current = serving.records().unwrap();
    let current_generation = current.protected_root().root().generation().get();
    drop(current);
    let before = segment_names(&root);
    serving.retire_displaced_segment().unwrap();
    assert!(before
        .iter()
        .any(|name| !segment_names(&root).contains(name)));
    assert_eq!(
        checkpoint_root_generation(&root),
        current_generation,
        "delete follows a checkpoint of the successor, not the displaced source"
    );
    serving.close();
}

#[test]
fn public_publication_cannot_remove_a_protected_segment() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    completed(prepare(&serving, placement, [84; 32], b"protected-source").execute());
    let reader = serving.records().unwrap();
    completed(prepare_rewrite(&serving, placement, [85; 32]).execute());
    let before = segment_names(&root);
    assert!(serving.certification_public_segment_removal_rejected());
    assert_eq!(segment_names(&root), before);
    drop(reader);
    serving.close();
}

fn checkpoint_root_generation(root: &Path) -> u64 {
    let bytes = std::fs::read(root.join("families").join("checkpoint.current")).unwrap();
    let header: [u8; CHECKPOINT_STREAM_HEADER_RECORD_BYTES] = bytes
        [..CHECKPOINT_STREAM_HEADER_RECORD_BYTES]
        .try_into()
        .unwrap();
    PhysicalCheckpointSource::decode_stream_header_record(&header)
        .unwrap()
        .root()
        .generation()
}

fn retirement_actions(root: &Path) -> Vec<IndependentRetirementAction> {
    produced_retirement_payloads(root)
        .iter()
        .map(|record| record.action)
        .collect()
}
