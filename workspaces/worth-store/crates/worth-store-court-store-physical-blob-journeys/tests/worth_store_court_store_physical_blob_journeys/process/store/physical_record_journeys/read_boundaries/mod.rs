#[allow(unused_imports)]
use crate::physical_record_journeys::read_boundaries::*;
use worth_proof::TransitionOutcome;
use worth_store_physical_backend::MediaOperationRole;
use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::AdmittedRecordAccessPolicy,
    worth_store::physical_runtime::ExternalPhysicalRecordLocator,
    worth_store::physical_runtime::PhysicalRecordAccessPolicy,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRootProtocolRoute,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordBootstrapDenial,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadDenial,
    worth_store::physical_runtime::RecordReadLimits,
};

use {
    worth_store_testkit_store::durable_publication::root::publish_single,
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
    worth_store_testkit_store::physical_record_journeys::success,
};

#[test]
fn permissive_access_policy_cannot_expand_fixed_page_reads() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (_, placement, _) = configuration();
    let serving = serving_from_initialization(&root);
    let publication = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([164; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"bounded".as_slice()])
            .unwrap(),
    );
    let record_id = publication.settled_members()[0].record_id(0).unwrap();
    let locator = worth_store::physical_runtime::ExternalPhysicalRecordLocator::new(
        serving.store_identity(),
        record_id,
    );
    serving.close();

    let page =
        root.join("families/records/segments/segment-0000000000000001-0000000000000001.pages");
    std::fs::OpenOptions::new()
        .write(true)
        .open(page)
        .unwrap()
        .set_len(16_385)
        .unwrap();
    let (format, access) = permissive_access();
    let reopened = success(open_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
    }));
    assert_eq!(
        reopened
            .root_protocol_counters()
            .root_entries(worth_store::physical_runtime::PhysicalRootProtocolRoute::OrdinaryOpen),
        2,
        "generation two ordinary open admits both current and previous addressed roots",
    );
    reopened
        .certification_physical_residency()
        .drain_unpinned_clean_frames();
    let before = reopened.media_counters();
    assert!(matches!(
        reopened.records().expect("read protection admission").open_external(
            locator,
            worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(1024).unwrap()),
        ),
        Err(error) if error.denial() == worth_store::physical_runtime::RecordReadDenial::ArtifactDamaged
    ));
    let after = reopened.media_counters();
    assert_eq!(
        after.completed_bytes_for(worth_store_physical_backend::MediaOperationRole::PositionedRead)
            - before.completed_bytes_for(
                worth_store_physical_backend::MediaOperationRole::PositionedRead
            ),
        208 + 2 * worth_store_testkit_store::durable_frame_oracle::HEADER_BYTES as u64,
        "the oversized data artifact is rejected by length before a frame allocation or page read",
    );
    reopened.close();
}

#[test]
fn permissive_access_policy_cannot_expand_current_root_bootstrap() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    serving_from_initialization(&root).close();
    let expected_catalog_bytes = worth_store_testkit_store::durable_frame_oracle::artifact_bytes(
        &root,
        &["families/records/bootstrap.catalog"],
    );
    std::fs::OpenOptions::new()
        .write(true)
        .open(root.join("families/records/roots/root-0000000000000001.manifest"))
        .unwrap()
        .set_len(16_385)
        .unwrap();

    let (format, access) = permissive_access();
    let open_media = media(&root);
    let before = open_media.media_counters();
    let denied = open_record_store!(open_media, |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
    })
    .into_raw();
    let worth_proof::TransitionOutcome::Denied(denial) = denied else {
        panic!("oversized current root must be denied");
    };
    assert_eq!(
        denial.reason(),
        worth_store::physical_runtime::RecordBootstrapDenial::CurrentRootDamaged
    );
    let returned = denial.into_runtime();
    let after = returned.media_counters();
    assert_eq!(
        after.completed_bytes_for(worth_store_physical_backend::MediaOperationRole::PositionedRead)
            - before.completed_bytes_for(
                worth_store_physical_backend::MediaOperationRole::PositionedRead
            ),
        expected_catalog_bytes
    );
    returned.close();
}
