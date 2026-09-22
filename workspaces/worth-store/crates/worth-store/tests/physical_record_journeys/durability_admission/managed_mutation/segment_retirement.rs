use worth_store::physical_runtime::{
    ManifestEntryCapacity, PhysicalMutationOutcome, PhysicalRecordPlacementPolicy, SegmentPageCount,
};

use super::selected_segment_rewrite::prepare_rewrite;
use super::*;

#[test]
fn reopened_store_keeps_displaced_rewrite_garbage() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    completed(prepare(&serving, placement, [45; 32], b"displaced-source").execute());
    completed(prepare_rewrite(&serving, placement, [46; 32]).execute());
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = crate::serving_from_open(&root);
    assert_eq!(
        serving.certification_charged_growth_bytes(),
        charged,
        "reopen must keep the displaced source page charged"
    );
    serving.close();
}

#[test]
fn reopened_store_keeps_displaced_garbage_after_a_later_append() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    completed(prepare(&serving, placement, [47; 32], b"displaced-then-append").execute());
    completed(prepare_rewrite(&serving, placement, [48; 32]).execute());
    completed(prepare(&serving, placement, [49; 32], b"after-rewrite").execute());
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = crate::serving_from_open(&root);
    assert_eq!(
        serving.certification_charged_growth_bytes(),
        charged,
        "a later append must not drop the displaced source charge"
    );
    serving.close();
}

#[test]
fn reopened_store_keeps_each_publication_routing_bound() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    for ordinal in 0_u8..=64 {
        let mut material = [76_u8; 32];
        material[0] = ordinal;
        completed(prepare(&serving, placement, material, b"height-step").execute());
    }
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = crate::serving_from_open(&root);
    assert_eq!(
        serving.certification_charged_growth_bytes(),
        charged,
        "reopen must price each publication at the routing bound it sealed"
    );
    serving.close();
}

#[test]
fn reopened_store_keeps_a_rewrite_routing_bound_at_the_height_boundary() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, _, access) = configuration();
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(2).unwrap())
        .admit(format)
        .unwrap();
    let media = crate::media(&root);
    let durability = crate::durability(&media);
    let serving = crate::success(media.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        ),
    ));
    for ordinal in 0_u8..2 {
        let mut material = [77_u8; 32];
        material[0] = ordinal;
        completed(prepare(&serving, placement, material, b"rewrite-bound").execute());
    }
    completed(prepare_rewrite(&serving, placement, [78; 32]).execute());
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = crate::serving_from_open(&root);
    assert_eq!(
        serving.certification_charged_growth_bytes(),
        charged,
        "a rewrite must not raise routing height when it inserts no records"
    );
    serving.close();
}

#[test]
fn reopened_store_keeps_a_capacity_transition_routing_bound() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, _, access) = configuration();
    let wide = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(4).unwrap())
        .admit(format)
        .unwrap();
    let narrow = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(2).unwrap())
        .admit(format)
        .unwrap();
    let media = crate::media(&root);
    let durability = crate::durability(&media);
    let serving = crate::success(media.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, wide, access, durability,
        ),
    ));
    let mut records = Vec::new();
    for ordinal in 0_u8..8 {
        records.push(vec![ordinal, 9, 9, 9]);
    }
    let batch: Vec<&[u8]> = records.iter().map(Vec::as_slice).collect();
    completed(prepare_records(&serving, wide, [79; 32], &batch).execute());
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([80; 32]),
        )
        .unwrap();
    let prepared = match submission
        .prepare_durable_append_with_manifest_capacity_transition(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"narrow" as &[u8]])
                .unwrap(),
            narrow,
            worth_store::physical_runtime::PhysicalManifestCapacityTransition::ReconstructToRequested,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::at(
                    worth_signal::facade::TemporalDuration::temporal_duration(1_000).unwrap(),
                ),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        _ => panic!("capacity reconstruction must prepare"),
    };
    completed(prepared.execute());
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = crate::serving_from_open(&root);
    assert_eq!(
        serving.certification_charged_growth_bytes(),
        charged,
        "reopen must price the successor manifest capacity"
    );
    serving.close();
}

#[test]
fn reopened_store_keeps_every_segment_from_one_batch() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (format, _, _) = configuration();
    let placement = PhysicalRecordPlacementPolicy::builder()
        .segment_pages(SegmentPageCount::new(1).unwrap())
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(format)
        .unwrap();
    let payload = vec![9_u8; 7_500];
    completed(prepare_records(
        &serving,
        placement,
        [75; 32],
        &[&payload, &payload, &payload],
    )
    .execute());
    let segments = std::fs::read_dir(root.join("families/records/segments"))
        .unwrap()
        .flatten()
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("pages"))
        .count();
    assert_eq!(segments, 3, "one page per segment must publish three segments");
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = crate::serving_from_open(&root);
    assert_eq!(
        serving.certification_charged_growth_bytes(),
        charged,
        "reopen must charge every segment the batch published"
    );
    serving.close();
}

#[test]
fn reopened_store_keeps_a_multi_page_rewrite_charge() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let payload = vec![9_u8; 7_500];
    completed(prepare_records(&serving, placement, [71; 32], &[&payload, &payload]).execute());
    completed(prepare_rewrite(&serving, placement, [72; 32]).execute());
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = crate::serving_from_open(&root);
    assert_eq!(
        serving.certification_charged_growth_bytes(),
        charged,
        "reopen must keep every page the multi-page append and rewrite sealed"
    );
    serving.close();
}

#[test]
fn reopened_store_keeps_an_extent_published_after_a_reserved_gap() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    serving.certification_limit_candidate_growth_bytes(80_000);
    let oversized = vec![3_u8; 100_000];
    match prepare(&serving, placement, [73; 32], &oversized).execute() {
        PhysicalMutationOutcome::ProvenNoEffect(_) => {}
        PhysicalMutationOutcome::Completed(_) => {
            panic!("the oversized extent must be denied before it publishes")
        }
        PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("growth denial must prove no effect")
        }
    }
    let published = vec![7_u8; 8_192];
    completed(prepare(&serving, placement, [74; 32], &published).execute());
    let charged = serving.certification_charged_growth_bytes();
    let skipped = root.join(
        "families/records/extents/extent-0000000000000001-0000000000000001.data",
    );
    let published_extent = root.join(
        "families/records/extents/extent-0000000000000002-0000000000000001.data",
    );
    assert!(!skipped.exists(), "the denied reservation must not publish extent 1");
    assert!(published_extent.is_file(), "the later append must publish extent 2");
    serving.close();
    let serving = crate::serving_from_open(&root);
    assert_eq!(serving.certification_charged_growth_bytes(), charged);
    serving.close();
}

#[test]
fn reopened_store_keeps_the_published_extent_charge() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (format, placement, _) = configuration();
    let payload = vec![7_u8; usize::try_from(format.declaration().page_size().bytes() / 2).unwrap()];
    completed(prepare(&serving, placement, [70; 32], &payload).execute());
    let charged = serving.certification_charged_growth_bytes();
    let extent = root.join(
        "families/records/extents/extent-0000000000000001-0000000000000001.data",
    );
    assert!(extent.is_file(), "the payload must publish an extent");
    serving.close();
    let serving = crate::serving_from_open(&root);
    assert_eq!(
        serving.certification_charged_growth_bytes(),
        charged,
        "reopen must keep the published extent bytes charged"
    );
    serving.close();
}
