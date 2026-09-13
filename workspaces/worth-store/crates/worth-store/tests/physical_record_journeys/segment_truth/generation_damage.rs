use super::*;

#[test]
fn checksum_valid_zero_generation_in_an_unrelated_slot_is_artifact_damage() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("zero-unrelated-slot-generation");
    let (format, placement, access) = configuration();
    let serving = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let published = durable_publication::publish_single(
        &serving,
        placement,
        durable_publication::certification_material("zero-unrelated-slot-generation", 1),
        RecordAppendBatch::try_from_iter([b"target".as_slice(), b"unrelated".as_slice()]).unwrap(),
    );
    let target = published.settled_members()[0].record_id(0).unwrap();
    serving.close();

    let page =
        root.join("families/records/segments/segment-0000000000000001-0000000000000001.pages");
    let mut bytes = std::fs::read(&page).unwrap();
    let second_slot_generation = 96..104;
    super::super::durable_frame_oracle::payload_mut(&mut bytes)[second_slot_generation]
        .copy_from_slice(&0_u64.to_le_bytes());
    super::super::durable_frame_oracle::reseal(&mut bytes);
    std::fs::write(page, bytes).unwrap();

    let reopened = success(open_record_store!(media(&root), |durability| {
        PhysicalRecordOpen::new(format, access, durability)
    }));
    let error = match reopened.records().open(
        target,
        RecordReadLimits::new(RecordByteLimit::new(32).unwrap()),
    ) {
        Err(error) => error,
        Ok(_) => panic!("zero generation in another slot cannot admit the clean page"),
    };
    assert_eq!(error.denial(), RecordReadDenial::ArtifactDamaged);
    assert_eq!(error.observation().generation_checks(), 2);
    assert_eq!(error.observation().generation_rejections(), 0);
    reopened.abort();
}

#[test]
fn checksum_valid_zero_page_generation_is_artifact_damage() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("zero-page-generation");
    let (format, placement, access) = configuration();
    let serving = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let published = durable_publication::publish_single(
        &serving,
        placement,
        durable_publication::certification_material("zero-page-generation", 1),
        RecordAppendBatch::try_from_iter([b"target".as_slice()]).unwrap(),
    );
    let target = published.settled_members()[0].record_id(0).unwrap();
    serving.close();

    let page =
        root.join("families/records/segments/segment-0000000000000001-0000000000000001.pages");
    let mut bytes = std::fs::read(&page).unwrap();
    bytes[28..36].copy_from_slice(&0_u64.to_le_bytes());
    super::super::durable_frame_oracle::reseal(&mut bytes);
    std::fs::write(page, bytes).unwrap();

    let reopened = success(open_record_store!(media(&root), |durability| {
        PhysicalRecordOpen::new(format, access, durability)
    }));
    let error = match reopened.records().open(
        target,
        RecordReadLimits::new(RecordByteLimit::new(32).unwrap()),
    ) {
        Err(error) => error,
        Ok(_) => panic!("zero page generation cannot admit the clean page"),
    };
    assert_eq!(error.denial(), RecordReadDenial::ArtifactDamaged);
    assert_eq!(error.observation().generation_checks(), 2);
    assert_eq!(error.observation().generation_rejections(), 0);
    reopened.abort();
}

#[test]
fn checksum_valid_zero_page_owner_identities_are_artifact_damage() {
    let parent = tempfile::tempdir().unwrap();
    for (name, certification, identity_range) in [
        ("segment", "zero-segment-identity", 48..56),
        ("page", "zero-page-identity", 56..64),
    ] {
        let root = parent.path().join(format!("zero-{name}-identity"));
        let (format, placement, access) = configuration();
        let serving = success(initialize_record_store!(media(&root), |durability| {
            PhysicalRecordInitialization::new(format, placement, access, durability)
        }));
        let published = durable_publication::publish_single(
            &serving,
            placement,
            durable_publication::certification_material(certification, 1),
            RecordAppendBatch::try_from_iter([b"target".as_slice()]).unwrap(),
        );
        let target = published.settled_members()[0].record_id(0).unwrap();
        serving.close();

        let page =
            root.join("families/records/segments/segment-0000000000000001-0000000000000001.pages");
        let mut bytes = std::fs::read(&page).unwrap();
        bytes[identity_range].copy_from_slice(&0_u64.to_le_bytes());
        super::super::durable_frame_oracle::reseal(&mut bytes);
        std::fs::write(page, bytes).unwrap();

        let reopened = success(open_record_store!(media(&root), |durability| {
            PhysicalRecordOpen::new(format, access, durability)
        }));
        let error = match reopened.records().open(
            target,
            RecordReadLimits::new(RecordByteLimit::new(32).unwrap()),
        ) {
            Err(error) => error,
            Ok(_) => panic!("zero {name} identity cannot admit the clean page"),
        };
        assert_eq!(error.denial(), RecordReadDenial::ArtifactDamaged);
        assert_eq!(error.observation().generation_checks(), 2);
        assert_eq!(error.observation().generation_rejections(), 0);
        reopened.abort();
    }
}
