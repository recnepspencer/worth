use super::*;

#[test]
fn cross_batch_page_reuse_is_cow_and_does_not_rebase_old_slots() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = dense_configuration(4);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let publish = |material, batch| {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            publish_single(&serving, placement, material, batch)
        }))
        .unwrap_or_else(|_| {
            panic!(
                "C5_PREDICATE:page-layout: copy-on-write publication must retain the admitted slot layout"
            )
        })
    };
    let first = publish(
        PhysicalMutationIdempotencyMaterial::new([163; 32]),
        RecordAppendBatch::try_from_iter([b"alpha".as_slice(), b"beta".as_slice()]).unwrap(),
    );
    let old_page = std::fs::read(
        root.join("families/records/segments/segment-0000000000000001-0000000000000001.pages"),
    )
    .unwrap();
    let old_offset = old_page[88..92].to_vec();
    let before_admission = serving.resident_admission_counters();
    let second = publish(
        PhysicalMutationIdempotencyMaterial::new([164; 32]),
        RecordAppendBatch::try_from_iter([b"gamma".as_slice(), b"delta".as_slice()]).unwrap(),
    );
    let after_admission = serving.resident_admission_counters();
    // The second publication reads one source page and two blocks from each
    // routing family (record, segment membership, and free space). Copying
    // the page also looks up the selected route of each of its two published
    // slots, so a retired slot is never republished. All nine reads cross
    // resident integrity admission.
    assert_eq!(
        after_admission.fresh_validations() + after_admission.exact_record_reuses(),
        before_admission.fresh_validations() + before_admission.exact_record_reuses() + 9,
    );
    assert_eq!(
        after_admission.owner_decoder_entries(),
        before_admission.owner_decoder_entries() + 9,
    );
    assert_eq!(
        after_admission.refusals_before_owner_entry(),
        before_admission.refusals_before_owner_entry(),
    );
    let new_page = std::fs::read(
        root.join("families/records/segments/segment-0000000000000001-0000000000000002.pages"),
    )
    .unwrap();
    assert_eq!(&new_page[88..92], old_offset, "C5_PREDICATE:page-layout");
    assert!(root
        .join("families/records/segments/segment-0000000000000001-0000000000000001.pages")
        .is_file());
    let old_record = serving
        .records()
        .expect("read protection admission")
        .open(
            first.settled_members()[0].record_id(0).unwrap(),
            RecordReadLimits::new(RecordByteLimit::new(32).unwrap()),
        )
        .unwrap();
    assert_eq!(read_record(old_record, 5).0, b"alpha");
    let appended = serving
        .records()
        .expect("read protection admission")
        .open(
            second.settled_members()[0].record_id(0).unwrap(),
            RecordReadLimits::new(RecordByteLimit::new(32).unwrap()),
        )
        .unwrap();
    assert_eq!(read_record(appended, 5).0, b"gamma");
    serving.close();
}
