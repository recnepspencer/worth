use worth_proof::{NonEmpty, TransitionOutcome};
use worth_store::physical_runtime::{
    ManifestEntryCapacity, PageFillPercent, PhysicalManifestCapacityTransition,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationPreparationDenial,
    PhysicalMutationPreparationSuccess, PhysicalRecordInitialization, PhysicalRecordOpen,
    PhysicalRecordPlacementPolicy, PhysicalWalGroupAppendFailureCause,
    PhysicalWalGroupAppendOutcome, PhysicalWalReservationDenial, RecordAppendBatch,
    RecordAppendDenial, RecordByteLimit, RecordReadDenial, RecordReadLimits,
    RecordServingTerminalPosture, SegmentPageCount, StalePhysicalRecordPlacement,
};
use worth_store_physical_backend::MediaOperationRole;

use super::{configuration, durable_publication, media, success};

#[path = "segment_truth/generation_damage.rs"]
mod generation_damage;

#[test]
fn segment_filename_and_header_disagreement_is_denied_before_record_decode() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, _, access) = configuration();
    let placement = PhysicalRecordPlacementPolicy::builder()
        .segment_pages(SegmentPageCount::new(1).unwrap())
        .page_fill(PageFillPercent::new(50).unwrap())
        .extent_threshold(RecordByteLimit::new(8_000).unwrap())
        .manifest_capacity(ManifestEntryCapacity::new(16).unwrap())
        .admit(format)
        .unwrap();
    let serving = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let payloads = [vec![1_u8; 4_000], vec![2_u8; 4_000], vec![3_u8; 4_000]];
    let published = durable_publication::publish_single(
        &serving,
        placement,
        durable_publication::certification_material("segment-header-disagreement", 1),
        RecordAppendBatch::try_from_iter(payloads.iter()).unwrap(),
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
        PhysicalRecordOpen::new(format, access, durability)
    }));
    assert!(matches!(
        reopened.records().open(
            first,
            RecordReadLimits::new(RecordByteLimit::new(4_000).unwrap())
        ),
        Err(error)
            if error.denial() == RecordReadDenial::StalePlacement(
                StalePhysicalRecordPlacement::PageIdentity
            )
                && error.observation().generation_checks() == 3
                && error.observation().generation_rejections() == 1
    ));
    assert_inspection_denies_preparation(&reopened, placement, 206);
    assert_eq!(
        reopened.abort().records().posture(),
        RecordServingTerminalPosture::InspectionRequired
    );
}

#[test]
fn checksum_valid_slot_generation_drift_preserves_exact_read_observation() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("slot-generation-drift");
    let (format, placement, access) = configuration();
    let serving = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let published = durable_publication::publish_single(
        &serving,
        placement,
        durable_publication::certification_material("slot-generation-drift", 1),
        RecordAppendBatch::try_from_iter([b"record".as_slice()]).unwrap(),
    );
    let record = published.settled_members()[0].record_id(0).unwrap();
    serving.close();

    let page =
        root.join("families/records/segments/segment-0000000000000001-0000000000000001.pages");
    let mut bytes = std::fs::read(&page).unwrap();
    super::durable_frame_oracle::payload_mut(&mut bytes)[56..64]
        .copy_from_slice(&2_u64.to_le_bytes());
    super::durable_frame_oracle::reseal(&mut bytes);
    std::fs::write(page, bytes).unwrap();

    let reopened = success(open_record_store!(media(&root), |durability| {
        PhysicalRecordOpen::new(format, access, durability)
    }));
    let error = match reopened.records().open(
        record,
        RecordReadLimits::new(RecordByteLimit::new(32).unwrap()),
    ) {
        Err(error) => error,
        Ok(_) => panic!("slot generation drift cannot open a clean record session"),
    };
    assert_eq!(
        error.denial(),
        RecordReadDenial::StalePlacement(StalePhysicalRecordPlacement::SlotGeneration)
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
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let payload = vec![7_u8; 40_000];
    let published = durable_publication::publish_single(
        &serving,
        placement,
        durable_publication::certification_material("extent-generation-drift", 1),
        RecordAppendBatch::try_from_iter([payload.as_slice()]).unwrap(),
    );
    let record = published.settled_members()[0].record_id(0).unwrap();
    serving.close();

    let manifest = root.join(
        "families/records/extent-manifests/extent-0000000000000001-0000000000000001.manifest",
    );
    let mut bytes = std::fs::read(&manifest).unwrap();
    bytes[28..36].copy_from_slice(&2_u64.to_le_bytes());
    super::durable_frame_oracle::reseal(&mut bytes);
    std::fs::write(manifest, bytes).unwrap();

    let reopened = success(open_record_store!(media(&root), |durability| {
        PhysicalRecordOpen::new(format, access, durability)
    }));
    let error = match reopened.records().open(
        record,
        RecordReadLimits::new(RecordByteLimit::new(payload.len() as u32).unwrap()),
    ) {
        Err(error) => error,
        Ok(_) => panic!("extent generation drift cannot open a clean record session"),
    };
    assert_eq!(
        error.denial(),
        RecordReadDenial::StalePlacement(StalePhysicalRecordPlacement::ExtentMembership)
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
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    durable_publication::publish_single(
        &serving,
        placement,
        durable_publication::certification_material("segment-manifest-damage", 1),
        RecordAppendBatch::try_from_iter([b"published".as_slice()]).unwrap(),
    );
    serving.close();

    let manifest = root.join("families/records/roots/root-0000000000000002.manifest");
    let mut bytes = std::fs::read(&manifest).unwrap();
    super::durable_frame_oracle::payload_mut(&mut bytes)[304..312]
        .copy_from_slice(&2_u64.to_le_bytes());
    super::durable_frame_oracle::reseal(&mut bytes);
    std::fs::write(&manifest, bytes).unwrap();
    assert_eq!(
        worth_store_offline_verifier::walk_current_durable_record_manifest(
            &root,
            format.declaration()
        ),
        Err(worth_store_offline_verifier::OfflineDurableManifestDenial::InvalidTreeShape)
    );

    let reopened = success(open_record_store!(media(&root), |durability| {
        PhysicalRecordOpen::new(format, access, durability)
    }));
    let before = reopened.media_counters();
    let submission = reopened.certification_record_submission();
    let prepared = match durable_publication::prepare_single(
        &submission,
        placement,
        PhysicalManifestCapacityTransition::PreserveCurrent,
        PhysicalMutationIdempotencyMaterial::new([207; 32]),
        RecordAppendBatch::try_from_iter([b"candidate".as_slice()]).unwrap(),
    )
    .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("damaged published layout is discovered during canonical data planning"),
    };
    assert!(matches!(
        submission.append_prepared_wal_group(NonEmpty::new(prepared, Vec::new())),
        PhysicalWalGroupAppendOutcome::NotAdmitted {
            cause: PhysicalWalGroupAppendFailureCause::Reservation(
                PhysicalWalReservationDenial::DataPlanning(
                    RecordAppendDenial::PublishedLayoutDamaged
                )
            ),
            ..
        }
    ));
    let after = reopened.media_counters();
    assert_eq!(
        after.attempts_for(MediaOperationRole::CreateNew),
        before.attempts_for(MediaOperationRole::CreateNew)
    );
    assert_inspection_denies_preparation(&reopened, placement, 208);
    assert_eq!(
        reopened.abort().records().posture(),
        RecordServingTerminalPosture::InspectionRequired
    );
}

#[test]
fn unsorted_duplicate_and_cross_root_entries_fail_before_membership() {
    let parent = tempfile::tempdir().unwrap();
    for corruption in [
        RoutingCorruption::Unsorted,
        RoutingCorruption::Duplicate,
        RoutingCorruption::CrossRoot,
    ] {
        exercise_routing_corruption(parent.path(), corruption);
    }
}

#[derive(Clone, Copy, Debug)]
enum RoutingCorruption {
    Unsorted,
    Duplicate,
    CrossRoot,
}

fn exercise_routing_corruption(parent: &std::path::Path, corruption: RoutingCorruption) {
    let root = parent.join(format!("{corruption:?}"));
    let (format, placement, access) = configuration();
    let serving = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let published = durable_publication::publish_single(
        &serving,
        placement,
        durable_publication::certification_material("segment-stale-placement", 1),
        RecordAppendBatch::try_from_iter([
            b"record-1".as_slice(),
            b"record-2".as_slice(),
            b"record-3".as_slice(),
            b"record-4".as_slice(),
        ])
        .unwrap(),
    );
    let record = published.settled_members()[0].record_id(0).unwrap();
    serving.close();

    let block_path =
        root.join("families/records/roots/root-0000000000000002-block-0000000000000001.manifest");
    let mut block = std::fs::read(&block_path).unwrap();
    match corruption {
        RoutingCorruption::Unsorted => {
            let payload = super::durable_frame_oracle::payload_mut(&mut block);
            let second = payload[128..216].to_vec();
            let third = payload[216..304].to_vec();
            payload[128..216].copy_from_slice(&third);
            payload[216..304].copy_from_slice(&second);
        }
        RoutingCorruption::Duplicate => {
            let payload = super::durable_frame_oracle::payload_mut(&mut block);
            let second = payload[128..216].to_vec();
            payload[216..304].copy_from_slice(&second);
        }
        RoutingCorruption::CrossRoot => {
            let payload = super::durable_frame_oracle::payload_mut(&mut block);
            let tree_identity = u64::from_le_bytes(payload[..8].try_into().unwrap());
            payload[..8].copy_from_slice(&(tree_identity + 1).to_le_bytes());
        }
    }
    reseal_frame(&mut block);
    std::fs::write(&block_path, &block).unwrap();

    let root_path = root.join("families/records/roots/root-0000000000000002.manifest");
    let mut root_bytes = std::fs::read(&root_path).unwrap();
    let block_checksum = super::durable_frame_oracle::independent_crc32c(&[&block]);
    super::durable_frame_oracle::payload_mut(&mut root_bytes)[68..72]
        .copy_from_slice(&block_checksum.to_le_bytes());
    reseal_frame(&mut root_bytes);
    std::fs::write(&root_path, root_bytes).unwrap();

    let expected_offline = match corruption {
        RoutingCorruption::Unsorted | RoutingCorruption::Duplicate => {
            worth_store_offline_verifier::OfflineDurableManifestDenial::InvalidTreeShape
        }
        RoutingCorruption::CrossRoot => {
            worth_store_offline_verifier::OfflineDurableManifestDenial::ReferenceMismatch
        }
    };
    assert_eq!(
        worth_store_offline_verifier::walk_current_durable_record_manifest(
            &root,
            format.declaration()
        ),
        Err(expected_offline)
    );
    let reopened = success(open_record_store!(media(&root), |durability| {
        PhysicalRecordOpen::new(format, access, durability)
    }));
    let error = match reopened.records().open(
        record,
        RecordReadLimits::new(RecordByteLimit::new(8).unwrap()),
    ) {
        Ok(_) => panic!("{corruption:?} routing block must not admit a read session"),
        Err(error) => error,
    };
    assert_eq!(error.denial(), RecordReadDenial::ArtifactDamaged);
    assert_eq!(error.observation().manifest_blocks(), 1);
    assert_eq!(error.observation().manifest_bytes(), block.len() as u64);
    assert_eq!(error.observation().touched_pages(), 0);
    assert_eq!(error.observation().touched_extents(), 0);
    reopened.abort();
}

fn reseal_frame(bytes: &mut [u8]) {
    super::durable_frame_oracle::reseal(bytes);
}

fn assert_inspection_denies_preparation(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: u8,
) {
    assert!(matches!(
        durable_publication::prepare_single(
            &serving.certification_record_submission(),
            placement,
            PhysicalManifestCapacityTransition::PreserveCurrent,
            PhysicalMutationIdempotencyMaterial::new([material; 32]),
            RecordAppendBatch::try_from_iter([b"retry".as_slice()]).unwrap(),
        )
        .into_raw(),
        TransitionOutcome::Denied(PhysicalMutationPreparationDenial::RecordAppend(
            RecordAppendDenial::ServingRequiresInspection
        ))
    ));
}
