use std::thread;

use worth_store::physical_runtime::{
    ManifestEntryCapacity, PhysicalRecordInitialization, PhysicalRecordPlacementPolicy,
    PhysicalRetirementDenial, RecordByteLimit, SegmentPageCount,
};

use super::super::independent_wal_oracle::produced_retirement_payloads;
use super::published_segments::segment_names;
use super::selected_segment_rewrite::prepare_rewrite;
use super::*;

#[test]
fn another_segments_generation_does_not_drop_an_unresolved_retirement() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, _, access) = configuration();
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    let placement = PhysicalRecordPlacementPolicy::builder()
        .segment_pages(SegmentPageCount::new(2).unwrap())
        .extent_threshold(RecordByteLimit::new(16_000).unwrap())
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(format)
        .unwrap();
    let serving = crate::success(initialize_record_store!(
        crate::media(&root),
        |durability| PhysicalRecordInitialization::new(format, placement, access, durability)
    ));
    let payload = vec![0x41_u8; 12_000];
    completed(prepare(&serving, placement, [11; 32], &payload).execute());
    completed(prepare_rewrite(&serving, placement, [12; 32]).execute());
    serving.certification_stop_after_retirement_delete();
    assert!(serving.retire_displaced_segment().is_err());
    completed(prepare(&serving, placement, [13; 32], &payload).execute());
    completed(prepare(&serving, placement, [14; 32], &payload).execute());
    completed(prepare_rewrite(&serving, placement, [15; 32]).execute());
    let retained = segment_names(&root);
    let other_generation = "segment-0000000000000003-0000000000000001.pages";
    assert!(
        retained.iter().any(|name| name == other_generation),
        "a second segment's generation 1 must stay beside the unresolved retirement"
    );
    assert!(retained
        .iter()
        .all(|name| name != "segment-0000000000000001-0000000000000001.pages"));
    serving.close();

    let serving = crate::serving_from_open(&root);
    assert!(serving.records().is_ok());
    assert_eq!(segment_names(&root), retained);
    let held = serving.certification_charged_growth_bytes();
    serving.retire_displaced_segment().unwrap();
    assert!(segment_names(&root)
        .iter()
        .any(|name| name == other_generation));
    assert_eq!(
        held - serving.certification_charged_growth_bytes(),
        page_bytes
    );
    serving.retire_displaced_segment().unwrap();
    assert!(segment_names(&root)
        .iter()
        .all(|name| name != other_generation));
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Absent)
    );
    serving.close();
}

#[test]
fn a_second_caller_cannot_resurrect_a_completed_retirement() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (format, placement, _) = configuration();
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    completed(prepare(&serving, placement, [21; 32], b"exclusive-source").execute());
    completed(prepare_rewrite(&serving, placement, [22; 32]).execute());
    let before = segment_names(&root);
    let charged = serving.certification_charged_growth_bytes();
    serving.certification_pause_before_retirement_intent();
    thread::scope(|threads| {
        let owner = threads.spawn(|| serving.retire_displaced_segment());
        while !serving.certification_retirement_intent_arrived() {
            thread::yield_now();
        }
        assert_eq!(
            serving.retire_displaced_segment(),
            Err(PhysicalRetirementDenial::Waiting)
        );
        assert!(produced_retirement_payloads(&root).is_empty());
        serving.certification_release_retirement_intent();
        owner.join().unwrap().unwrap();
    });
    assert!(before
        .iter()
        .any(|name| !segment_names(&root).contains(name)));
    assert_eq!(
        charged - serving.certification_charged_growth_bytes(),
        page_bytes
    );
    serving.close();
    let serving = crate::serving_from_open(&root);
    let reopened = serving.certification_charged_growth_bytes();
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Absent)
    );
    assert_eq!(serving.certification_charged_growth_bytes(), reopened);
    serving.close();
    let serving = crate::serving_from_open(&root);
    assert_eq!(serving.certification_charged_growth_bytes(), reopened);
    serving.close();
}
