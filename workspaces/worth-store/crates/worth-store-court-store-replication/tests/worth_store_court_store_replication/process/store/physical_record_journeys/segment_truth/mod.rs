#[allow(unused_imports)]
use crate::physical_record_journeys::segment_truth::*;
use worth_store_physical_backend::MediaOperationRole;
use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
use {
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PageFillPercent,
    worth_store::physical_runtime::PhysicalManifestCapacityTransition,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationPreparationDenial,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::PhysicalWalGroupAppendFailureCause,
    worth_store::physical_runtime::PhysicalWalGroupAppendOutcome,
    worth_store::physical_runtime::PhysicalWalReservationDenial,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordAppendDenial,
    worth_store::physical_runtime::RecordBootstrapDenial,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadDenial,
    worth_store::physical_runtime::RecordReadLimits,
    worth_store::physical_runtime::RecordServingTerminalPosture,
    worth_store::physical_runtime::SegmentPageCount,
    worth_store::physical_runtime::StalePhysicalRecordPlacement,
};

use {
    worth_store_testkit_store::durable_publication,
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::success,
};

#[test]
fn segment_filename_and_header_disagreement_is_denied_before_record_decode() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, _, access) = configuration();
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .segment_pages(worth_store::physical_runtime::SegmentPageCount::new(1).unwrap())
        .page_fill(worth_store::physical_runtime::PageFillPercent::new(50).unwrap())
        .extent_threshold(worth_store::physical_runtime::RecordByteLimit::new(8_000).unwrap())
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(16).unwrap())
        .admit(format)
        .unwrap();
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )
    }));
    let payloads = [vec![1_u8; 4_000], vec![2_u8; 4_000], vec![3_u8; 4_000]];
    let published = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "segment-header-disagreement",
            1,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter(payloads.iter()).unwrap(),
    );
    let first = published.settled_members()[0].record_id(0).unwrap();
    serving.close();

    let segments = root.join("families/records/segments");
    let wrong_header =
        std::fs::read(segments.join("segment-0000000000000002-0000000000000001.pages")).unwrap();
    std::fs::write(
        segments.join("segment-0000000000000001-0000000000000001.pages"),
        wrong_header,
    )
    .unwrap();

    let reopened = success(open_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
    }));
    assert!(matches!(
        reopened.records().expect("read protection admission").open(
            first,
            worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(4_000).unwrap())
        ),
        Err(error)
            if error.denial() == worth_store::physical_runtime::RecordReadDenial::StalePlacement(
                worth_store::physical_runtime::StalePhysicalRecordPlacement::PageIdentity
            )
                && error.observation().generation_checks() == 3
                && error.observation().generation_rejections() == 1
    ));
    assert_inspection_denies_preparation(&reopened, placement, 206);
    assert_eq!(
        reopened.abort().records().posture(),
        worth_store::physical_runtime::RecordServingTerminalPosture::InspectionRequired
    );
}

#[test]
fn checksum_valid_slot_generation_drift_preserves_exact_read_observation() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("slot-generation-drift");
    let (format, placement, access) = configuration();
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )
    }));
    let published = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "slot-generation-drift",
            1,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"record".as_slice()])
            .unwrap(),
    );
    let record = published.settled_members()[0].record_id(0).unwrap();
    serving.close();

    let page =
        root.join("families/records/segments/segment-0000000000000001-0000000000000001.pages");
    let mut bytes = std::fs::read(&page).unwrap();
    worth_store_testkit_store::durable_frame_oracle::payload_mut(&mut bytes)[56..64]
        .copy_from_slice(&2_u64.to_le_bytes());
    worth_store_testkit_store::durable_frame_oracle::reseal(&mut bytes);
    std::fs::write(page, bytes).unwrap();

    let reopened = success(open_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
    }));
    let error = match reopened.records().expect("read protection admission").open(
        record,
        worth_store::physical_runtime::RecordReadLimits::new(
            worth_store::physical_runtime::RecordByteLimit::new(32).unwrap(),
        ),
    ) {
        Err(error) => error,
        Ok(_) => panic!("slot generation drift cannot open a clean record session"),
    };
    assert_eq!(
        error.denial(),
        worth_store::physical_runtime::RecordReadDenial::StalePlacement(
            worth_store::physical_runtime::StalePhysicalRecordPlacement::SlotGeneration
        )
    );
    assert_eq!(error.observation().generation_checks(), 4);
    assert_eq!(error.observation().generation_rejections(), 1);
    reopened.abort();
}

#[test]
fn checksum_valid_extent_generation_drift_is_stale_membership() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("extent-generation-drift");
    let (format, placement, access) = configuration();
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )
    }));
    let payload = vec![7_u8; 40_000];
    let published = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "extent-generation-drift",
            1,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([payload.as_slice()])
            .unwrap(),
    );
    let record = published.settled_members()[0].record_id(0).unwrap();
    serving.close();

    let manifest = root.join("families/records/arenas/arena-0000000000000001.data");
    let mut bytes = std::fs::read(&manifest).unwrap();
    bytes[28..36].copy_from_slice(&2_u64.to_le_bytes());
    worth_store_testkit_store::durable_frame_oracle::reseal(&mut bytes[..104]);
    std::fs::write(manifest, bytes).unwrap();

    let reopened = success(open_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
    }));
    let error = match reopened.records().expect("read protection admission").open(
        record,
        worth_store::physical_runtime::RecordReadLimits::new(
            worth_store::physical_runtime::RecordByteLimit::new(payload.len() as u32).unwrap(),
        ),
    ) {
        Err(error) => error,
        Ok(_) => panic!("extent generation drift cannot open a clean record session"),
    };
    assert_eq!(
        error.denial(),
        worth_store::physical_runtime::RecordReadDenial::StalePlacement(
            worth_store::physical_runtime::StalePhysicalRecordPlacement::ExtentMembership
        )
    );
    assert_eq!(error.observation().generation_checks(), 1);
    assert_eq!(error.observation().generation_rejections(), 1);
    reopened.abort();
}

#[test]
fn dishonest_inline_tail_owner_is_denied_before_candidate_effects() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = configuration();
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )
    }));
    worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "segment-manifest-damage",
            1,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"published".as_slice()])
            .unwrap(),
    );
    serving.close();

    let manifest = root.join("families/records/roots/root-0000000000000002.manifest");
    let mut bytes = std::fs::read(&manifest).unwrap();
    worth_store_testkit_store::durable_frame_oracle::payload_mut(&mut bytes)[320..328]
        .copy_from_slice(&2_u64.to_le_bytes());
    worth_store_testkit_store::durable_frame_oracle::reseal(&mut bytes);
    std::fs::write(&manifest, bytes).unwrap();
    assert_eq!(
        worth_store_offline_verifier::walk_current_durable_record_manifest(
            &root,
            format.declaration()
        ),
        Err(worth_store_offline_verifier::OfflineDurableManifestDenial::InvalidTreeShape)
    );

    let reopened = success(open_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
    }));
    let before = reopened.media_counters();
    let submission = reopened.certification_record_submission();
    let prepared = match worth_store_testkit_store::durable_publication::wal_data::prepare_single(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([207; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"candidate".as_slice()])
            .unwrap(),
    )
    .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        _ => panic!("damaged published layout is discovered during canonical data planning"),
    };
    assert!(matches!(
        submission.append_prepared_wal_group(worth_proof::NonEmpty::new(prepared, Vec::new())),
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::NotAdmitted {
            cause: worth_store::physical_runtime::PhysicalWalGroupAppendFailureCause::Reservation(
                worth_store::physical_runtime::PhysicalWalReservationDenial::DataPlanning(
                    worth_store::physical_runtime::RecordAppendDenial::PublishedLayoutDamaged
                )
            ),
            ..
        }
    ));
    let after = reopened.media_counters();
    assert_eq!(
        after.attempts_for(worth_store_physical_backend::MediaOperationRole::CreateNew),
        before.attempts_for(worth_store_physical_backend::MediaOperationRole::CreateNew)
    );
    assert_inspection_denies_preparation(&reopened, placement, 208);
    assert_eq!(
        reopened.abort().records().posture(),
        worth_store::physical_runtime::RecordServingTerminalPosture::InspectionRequired
    );
}

#[test]
fn unsorted_duplicate_and_cross_root_entries_fail_before_membership() {
    let parent = tempfile::tempdir().unwrap();
    for corruption in [
        crate::physical_record_journeys::segment_truth::RoutingCorruption::Unsorted,
        crate::physical_record_journeys::segment_truth::RoutingCorruption::Duplicate,
        crate::physical_record_journeys::segment_truth::RoutingCorruption::CrossRoot,
    ] {
        exercise_routing_corruption(parent.path(), corruption);
    }
}
#[path = "generation_damage/mod.rs"]
pub(crate) mod generation_damage;
