use super::*;

#[test]
fn physical_residency_serves_real_reads_and_candidate_writes() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (_, placement, _) = configuration();
    let serving = serving_from_initialization(&root);
    let first_publication = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([150; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"direct".as_slice()])
            .unwrap(),
    );
    let first = first_publication.settled_members()[0].record_id(0).unwrap();

    let counters = serving.certification_frame_port_observer();
    let mut read = serving
        .records()
        .expect("read protection admission")
        .open(
            first,
            worth_store::physical_runtime::RecordReadLimits::new(
                worth_store::physical_runtime::RecordByteLimit::new(64).unwrap(),
            ),
        )
        .unwrap();
    let mut bytes = [0_u8; 6];
    assert_eq!(read.read_next(&mut bytes).unwrap(), bytes.len());
    assert_eq!(&bytes, b"direct");
    drop(read);
    let second = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([151; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"wrapped".as_slice()])
            .unwrap(),
    );
    assert_eq!(second.current_root().generation(), 3);
    let mut retained_read = serving
        .records()
        .expect("read protection admission")
        .open(
            second.settled_members()[0].record_id(0).unwrap(),
            worth_store::physical_runtime::RecordReadLimits::new(
                worth_store::physical_runtime::RecordByteLimit::new(64).unwrap(),
            ),
        )
        .unwrap();
    let mut retained_bytes = [0_u8; 7];
    assert_eq!(
        retained_read.read_next(&mut retained_bytes).unwrap(),
        retained_bytes.len()
    );
    assert_eq!(&retained_bytes, b"wrapped");
    drop(retained_read);
    let snapshot = counters.snapshot();
    assert!(snapshot.loads() > 0);
    assert!(snapshot.residency_faults() > 0);
    assert!(snapshot.residency_hits() > 0);
    assert_eq!(snapshot.writebacks(), 0);
    assert_eq!(
        snapshot.candidate_publications(),
        snapshot.candidate_frames()
    );
    assert!(snapshot.loads() > 0);
    assert!(snapshot.candidate_submissions() >= 1);
    assert!(snapshot.candidate_frames() >= 5);
    assert!(snapshot.candidate_bytes() > b"wrapped".len() as u64);
    assert_eq!(
        snapshot.declared_candidate_frames(),
        snapshot.candidate_frames()
    );
    assert_eq!(
        snapshot.declared_candidate_bytes(),
        snapshot.candidate_bytes()
    );
    assert_eq!(snapshot.wrapper_frames(), 0);
    assert!(snapshot.peak_retained_candidate_frames() >= 1);
    let shutdown = serving.close();
    assert!(!shutdown.residency().requires_inspection());
}
