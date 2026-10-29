#[allow(unused_imports)] use crate::physical_record_journeys::generation_policy_truth::*;
use {std::path::Path};

use {worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::PageFillPercent, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::SegmentPageCount};
use {worth_store_offline_verifier::OfflineDurableManifestWalk, worth_store_offline_verifier::OfflineRecordPlacement};
use {worth_store_physical_backend::MediaOperationRole};
use {worth_store_physical_format::RecordAllocationClass};

use {worth_store_testkit_store::durable_publication, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::read_record, worth_store_testkit_store::physical_record_journeys::scenario_configuration::dense_configuration, worth_store_testkit_store::physical_record_journeys::success};

#[test]
fn multi_page_cow_preserves_untouched_page_generation_and_all_records() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = dense_configuration(4);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let first_payloads = [vec![1_u8; 3_000], vec![2_u8; 3_000], vec![3_u8; 3_000]];
    let first = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material("multi-page-cow-generation", 1),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter(first_payloads.iter()).unwrap(),
    );
    let first = &first.settled_members()[0];
    let old_root = decode_root(&root, 2, format.declaration());
    let fourth_payload = vec![4_u8; 3_000];
    serving
        .certification_physical_residency()
        .drain_unpinned_clean_frames();
    let before_cow = serving.media_counters();
    let fourth_publication = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material("multi-page-cow-generation", 2),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([fourth_payload.as_slice()]).unwrap(),
    );
    let fourth = &fourth_publication.settled_members()[0];
    let after_cow = serving.media_counters();
    assert_eq!(
        after_cow.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead)
            - before_cow.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead),
        4,
        "a cold COW plan faults only its required physical frames; metadata probes and untouched pages are not frame reads",
    );
    assert_eq!(
        fourth_publication
            .root_planning_observation()
            .manifest_blocks_read(),
        3
    );
    serving.close();

    let new_root = decode_root(&root, 3, format.declaration());
    let old_first = inline(&old_root, first.record_id(0).unwrap());
    let new_first = inline(&new_root, first.record_id(0).unwrap());
    let old_third = inline(&old_root, first.record_id(2).unwrap());
    let new_third = inline(&new_root, first.record_id(2).unwrap());
    let new_fourth = inline(&new_root, fourth.record_id(0).unwrap());
    assert_eq!(
        (old_first.segment_generation(), old_first.page_generation()),
        (1, 1)
    );
    assert_eq!(
        (new_first.segment_generation(), new_first.page_generation()),
        (1, 1)
    );
    assert_eq!(
        (old_third.segment_generation(), old_third.page_generation()),
        (1, 1)
    );
    assert_eq!(
        (new_third.segment_generation(), new_third.page_generation()),
        (2, 2)
    );
    assert_eq!(new_fourth.page_cell(), new_third.page_cell());
    assert!(root
        .join("families/records/segments/segment-0000000000000001-0000000000000001.pages")
        .is_file());
    assert_eq!(
        std::fs::metadata(
            root.join("families/records/segments/segment-0000000000000001-0000000000000002.pages")
        )
        .unwrap()
        .len(),
        16_384
    );
    assert!(root
        .join("families/records/segment-manifests/segments-0000000000000003-block-0000000000000002.manifest")
        .is_file());
    assert!(!root
        .join(
            "families/records/segment-manifests/segment-0000000000000001-0000000000000002.manifest"
        )
        .exists());

    let reopened = success(open_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
    }));
    for (index, payload) in first_payloads.iter().enumerate() {
        let session = reopened
            .records()
            .expect("read protection admission")
            .open(
                first.record_id(index).unwrap(),
                worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(3_000).unwrap()),
            )
            .unwrap();
        assert_eq!(read_record(session, payload.len()).0, *payload);
    }
    let session = reopened
        .records()
        .expect("read protection admission")
        .open(
            fourth.record_id(0).unwrap(),
            worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(3_000).unwrap()),
        )
        .unwrap();
    assert_eq!(read_record(session, fourth_payload.len()).0, fourth_payload);
    reopened.close();
}

#[test]
fn segment_target_drift_opens_a_new_policy_honest_segment() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, _, access) = dense_configuration(2);
    let first_policy = placement(format, 2);
    let second_policy = placement(format, 3);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(format, first_policy, access, durability)
    }));
    let first = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        first_policy,
        worth_store_testkit_store::durable_publication::material::certification_material("segment-target-policy-drift", 1),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"first".as_slice()]).unwrap(),
    );
    let first = &first.settled_members()[0];
    let checkpoint =
        worth_store_testkit_store::durable_publication::mutable_reopen::checkpoint_for_mutable_reopen(&serving, "segment-target-policy-drift");
    assert!(
        checkpoint
            .retained_wal_tail()
            .checkpoint_boundary_lsn()
            .get()
            > 0,
        "a mutable fresh reopen requires an exact namespace-durable WAL cutoff"
    );
    serving.close();

    let serving = success(open_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
    }));
    let second = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        second_policy,
        worth_store_testkit_store::durable_publication::material::certification_material("segment-target-policy-drift", 2),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"second".as_slice()]).unwrap(),
    );
    let second = &second.settled_members()[0];
    serving.close();

    let current = decode_root(&root, 3, format.declaration());
    let first_placement = inline(&current, first.record_id(0).unwrap());
    let second_placement = inline(&current, second.record_id(0).unwrap());
    assert_eq!(
        (
            first_placement.segment(),
            first_placement.segment_page_capacity()
        ),
        (1, 2)
    );
    assert_eq!(
        (
            second_placement.segment(),
            second_placement.segment_page_capacity()
        ),
        (2, 3)
    );
    assert_eq!(
        (
            first_placement.segment_generation(),
            second_placement.segment_generation()
        ),
        (1, 1)
    );

    let free = worth_store_testkit_store::physical_record_journeys::manifest_fixture::decode_free_space_tree(&root, 3, format.declaration(), 128);
    let inline_ranges = free
        .entries
        .iter()
        .filter(|entry| entry.class() == worth_store_physical_format::RecordAllocationClass::InlinePage)
        .map(|entry| {
            (
                entry.owner(),
                entry.inline_free_frontier().unwrap().first_unallocated(),
                entry.inline_free_frontier().unwrap().unallocated_count(),
                entry.generation(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(inline_ranges, vec![(1, 2, 1, 1), (2, 2, 2, 1)]);

    let reopened = success(open_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
    }));
    for (record, expected) in [
        (first.record_id(0).unwrap(), b"first".as_slice()),
        (second.record_id(0).unwrap(), b"second".as_slice()),
    ] {
        let session = reopened
            .records()
            .expect("read protection admission")
            .open(
                record,
                worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(16).unwrap()),
            )
            .unwrap();
        assert_eq!(read_record(session, expected.len()).0, expected);
    }
    reopened.close();
}

#[test]
fn returning_to_an_older_policy_does_not_search_historical_segments() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, _, access) = dense_configuration(2);
    let two_page_policy = placement(format, 2);
    let three_page_policy = placement(format, 3);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(format, two_page_policy, access, durability)
    }));
    let first = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        two_page_policy,
        worth_store_testkit_store::durable_publication::material::certification_material("historical-segment-policy", 1),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"first".as_slice()]).unwrap(),
    );
    let first = &first.settled_members()[0];
    worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        three_page_policy,
        worth_store_testkit_store::durable_publication::material::certification_material("historical-segment-policy", 2),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"second".as_slice()]).unwrap(),
    );
    let third = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        two_page_policy,
        worth_store_testkit_store::durable_publication::material::certification_material("historical-segment-policy", 3),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"third".as_slice()]).unwrap(),
    );
    let third = &third.settled_members()[0];
    serving.close();

    let root_manifest = decode_root(&root, 4, format.declaration());
    let old = inline(&root_manifest, first.record_id(0).unwrap());
    let returned = inline(&root_manifest, third.record_id(0).unwrap());
    assert_eq!(old.segment(), 1);
    assert_eq!(returned.segment(), 3);
    assert_eq!(old.segment_generation(), 1);
    assert_eq!(returned.segment_generation(), 1);
    assert!(root
        .join("families/records/segments/segment-0000000000000003-0000000000000001.pages")
        .exists());
}
