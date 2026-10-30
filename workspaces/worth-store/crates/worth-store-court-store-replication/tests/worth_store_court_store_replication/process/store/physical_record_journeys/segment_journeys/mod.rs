#[allow(unused_imports)]
use crate::physical_record_journeys::segment_journeys::*;

use {
    worth_store_testkit_store::durable_publication::root::publish_single,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::read_record,
    worth_store_testkit_store::physical_record_journeys::scenario_configuration::dense_configuration,
    worth_store_testkit_store::physical_record_journeys::success,
};

#[test]
fn one_batch_rolls_across_four_segments_and_routes_without_scans() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = dense_configuration(2);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )
    }));
    let records = (0_u8..15)
        .map(|value| vec![value; 3_000])
        .collect::<Vec<_>>();
    let writeback_baseline =
        crate::physical_record_journeys::segment_journeys::SegmentWritebackBaseline::capture(
            &serving,
        );
    let published = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        publish_single(
            &serving,
            placement,
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([161; 32]),
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter(records.iter()).unwrap(),
        )
    }))
    .unwrap_or_else(|_| {
        panic!(
            "C5_PREDICATE:identity-placement-seam: persisted placements must retain admitted record identities"
        )
    });
    let member = &published.settled_members()[0];
    assert_eq!(member.persisted_records().len(), 15);
    assert_eq!(member.observation().segment_artifacts(), 4);
    assert_segment_frame_and_media_evidence(
        &serving,
        &writeback_baseline,
        member.observation().segment_artifacts(),
        published.current_artifacts().len() as u64,
    );
    assert_segment_work_and_signal_evidence(&serving, &writeback_baseline);
    assert_segment_artifact_lengths(&root);
    assert_inline_placement_truth(&root, format.declaration());
    assert!(
        worth_store_testkit_store::physical_record_journeys::manifest_fixture::decode_free_space_tree(&root, 2, format.declaration(), 128)
            .entries
            .is_empty(),
        "all eight inline pages are occupied, and C11 has no extent bump-frontier entry"
    );
    let epoch = member.record_id(0).unwrap().allocation_epoch();
    for index in 0..member.persisted_records().len() {
        let id = member.record_id(index).unwrap();
        assert_eq!(id.allocation_epoch(), epoch);
        assert_eq!(id.ordinal(), index as u64 + 1);
    }
    let request = segment_reader_request(serving.store_identity(), member);
    serving.close();
    assert_fresh_process_records(&root, &request);
}

#[test]
fn multi_block_manifest_lookup_has_logarithmic_path_and_exact_parity() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, _, access) = dense_configuration(2);
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .segment_pages(worth_store::physical_runtime::SegmentPageCount::new(2).unwrap())
        .extent_threshold(worth_store::physical_runtime::RecordByteLimit::new(8_000).unwrap())
        .page_fill(worth_store::physical_runtime::PageFillPercent::new(50).unwrap())
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(2).unwrap())
        .admit(format)
        .unwrap();
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )
    }));
    let payloads = (0_u8..9).map(|value| vec![value; 100]).collect::<Vec<_>>();
    let published = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([162; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter(payloads.iter()).unwrap(),
    );
    let member = &published.settled_members()[0];
    let offline =
        worth_store_testkit_store::physical_record_journeys::manifest_fixture::decode_routing_tree(
            &root,
            2,
            format.declaration(),
            2,
        );
    assert_eq!(offline.routing_level(), Some(3));
    assert_eq!(offline.placements().len(), payloads.len());
    assert!(offline.manifest_blocks() > 5);
    assert_eq!(offline.segment_pages().len(), 1);
    assert_eq!(offline.free_space().len(), 1);
    let free = offline.free_space()[0];
    assert_eq!(
        free.class(),
        worth_store_offline_verifier::OfflineAllocationClass::InlinePage
    );
    assert_eq!(free.owner(), 1);
    assert_eq!(free.first_unallocated(), 2);
    assert_eq!(free.unallocated_count(), 1);
    assert_eq!(free.generation(), 1);
    for (index, expected) in payloads.iter().enumerate().rev() {
        let session = serving
            .records()
            .expect("read protection admission")
            .open(
                member.record_id(index).unwrap(),
                worth_store::physical_runtime::RecordReadLimits::new(
                    worth_store::physical_runtime::RecordByteLimit::new(100).unwrap(),
                ),
            )
            .unwrap();
        let observation = session.observation();
        assert_eq!(observation.manifest_blocks(), 5);
        assert_eq!(
            observation.manifest_comparisons(),
            if index == 8 { 6 } else { 9 }
        );
        assert_eq!(
            observation.manifest_bytes(),
            (if index == 8 { 616 } else { 848 })
                + observation.manifest_blocks()
                    * worth_store_testkit_store::durable_frame_oracle::HEADER_BYTES as u64
        );
        assert_eq!(observation.touched_segments(), 1);
        assert_eq!(observation.touched_pages(), 1);
        assert_eq!(observation.touched_extents(), 0);
        assert_eq!(observation.payload_bytes(), 0);
        assert_eq!(read_record(session, 100).0, *expected);
        assert_eq!(
            offline.placements()[index].record().ordinal(),
            index as u64 + 1
        );
    }
    serving.close();
}
#[path = "page_reuse/mod.rs"]
pub(crate) mod page_reuse;
