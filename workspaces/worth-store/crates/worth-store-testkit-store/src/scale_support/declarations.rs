pub use {worth_foundational::compare_canonical_basis, worth_foundational::prepare_canonical_comparison, worth_foundational::CanonicalComparisonOutcome, worth_foundational::CanonicalEquivalenceBasis};
pub use {worth_proof::TransitionOutcome};
pub use {worth_store::physical_runtime::lower_record_publication_canonical_basis, worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::AdmittedRecordAccessPolicy, worth_store::physical_runtime::AdmittedRecordPlacementPolicy, worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::PageFillPercent, worth_store::physical_runtime::PhysicalRecordAccessPolicy, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordScanCounterSnapshot, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest, worth_store::physical_runtime::SegmentPageCount, worth_store::physical_runtime::ServingPhysicalRuntime};

pub fn complete_scan(
    serving: &ServingPhysicalRuntime,
    width: u32,
    scratch_bytes: usize,
) -> RecordScanCounterSnapshot {
    let mut session = serving
        .records()
        .expect("read protection admission")
        .scan(
            worth_store::physical_runtime::RecordScanRequest::from_start().with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(width).unwrap()),
        )
        .unwrap();
    let mut scratch = vec![0_u8; scratch_bytes];
    loop {
        match session.read_next_into(&mut scratch).unwrap() {
            worth_store::physical_runtime::RecordScanOutcome::Batch(_) => {}
            worth_store::physical_runtime::RecordScanOutcome::Completed(completed) => return completed.observation(),
        }
    }
}

pub fn assert_canonical_parity(
    serving: &ServingPhysicalRuntime,
    offline: &worth_store_offline_verifier::OfflineDurableManifestWalk,
) {
    let runtime = transition_success(lower_record_publication_canonical_basis(
        &serving.certification_publication_summary().unwrap(),
    ));
    let offline = transition_success(
        worth_store_offline_verifier::lower_offline_record_publication_canonical_basis(offline),
    );
    let comparison = transition_success(prepare_canonical_comparison(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        runtime,
        offline,
    ));
    assert!(matches!(
        compare_canonical_basis(&comparison),
        worth_foundational::CanonicalComparisonOutcome::Equivalent(_)
    ));
}

pub fn format() -> AdmittedPhysicalRecordFormat {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder().admit().unwrap())
}

pub fn placement(
    format: AdmittedPhysicalRecordFormat,
    manifest_capacity: u16,
    segment_pages: u32,
    page_fill: u8,
) -> AdmittedRecordPlacementPolicy {
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .segment_pages(worth_store::physical_runtime::SegmentPageCount::new(segment_pages).unwrap())
        .extent_threshold(worth_store::physical_runtime::RecordByteLimit::new(8_000).unwrap())
        .page_fill(worth_store::physical_runtime::PageFillPercent::new(page_fill).unwrap())
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(manifest_capacity).unwrap())
        .admit(format)
        .unwrap()
}

pub fn access(
    format: AdmittedPhysicalRecordFormat,
    scan_records: u32,
) -> AdmittedRecordAccessPolicy {
    worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .transfer_limit(worth_store::physical_runtime::RecordByteLimit::new(16_384).unwrap())
        .scratch_limit(worth_store::physical_runtime::RecordByteLimit::new(131_072).unwrap())
        .scan_record_limit(worth_store::physical_runtime::RecordCountLimit::new(scan_records).unwrap())
        .admit(format)
        .unwrap()
}

pub fn transition_success<T, D>(outcome: TransitionOutcome<T, D>) -> T {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        _ => panic!("the courtroom evidence transition must succeed"),
    }
}