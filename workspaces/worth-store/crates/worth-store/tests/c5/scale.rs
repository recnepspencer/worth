use worth_foundational::{
    compare_canonical_basis, prepare_canonical_comparison, CanonicalComparisonOutcome,
    CanonicalEquivalenceBasis,
};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    lower_record_publication_canonical_basis, AdmittedPhysicalRecordFormat,
    AdmittedRecordAccessPolicy, AdmittedRecordPlacementPolicy, ManifestEntryCapacity,
    PageFillPercent, PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration,
    PhysicalRecordPlacementPolicy, RecordByteLimit, RecordCountLimit, RecordScanCounterSnapshot,
    RecordScanOutcome, RecordScanRequest, SegmentPageCount, ServingPhysicalRuntime,
};

pub(super) fn complete_scan(
    serving: &ServingPhysicalRuntime,
    width: u32,
    scratch_bytes: usize,
) -> RecordScanCounterSnapshot {
    let mut session = serving
        .records()
        .expect("read protection admission")
        .scan(
            RecordScanRequest::from_start().with_batch_limit(RecordCountLimit::new(width).unwrap()),
        )
        .unwrap();
    let mut scratch = vec![0_u8; scratch_bytes];
    loop {
        match session.read_next_into(&mut scratch).unwrap() {
            RecordScanOutcome::Batch(_) => {}
            RecordScanOutcome::Completed(completed) => return completed.observation(),
        }
    }
}

pub(super) fn assert_canonical_parity(
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
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        runtime,
        offline,
    ));
    assert!(matches!(
        compare_canonical_basis(&comparison),
        CanonicalComparisonOutcome::Equivalent(_)
    ));
}

pub(super) fn format() -> AdmittedPhysicalRecordFormat {
    AdmittedPhysicalRecordFormat::admit(PhysicalRecordFormatDeclaration::builder().admit().unwrap())
}

pub(super) fn placement(
    format: AdmittedPhysicalRecordFormat,
    manifest_capacity: u16,
    segment_pages: u32,
    page_fill: u8,
) -> AdmittedRecordPlacementPolicy {
    PhysicalRecordPlacementPolicy::builder()
        .segment_pages(SegmentPageCount::new(segment_pages).unwrap())
        .extent_threshold(RecordByteLimit::new(8_000).unwrap())
        .page_fill(PageFillPercent::new(page_fill).unwrap())
        .manifest_capacity(ManifestEntryCapacity::new(manifest_capacity).unwrap())
        .admit(format)
        .unwrap()
}

pub(super) fn access(
    format: AdmittedPhysicalRecordFormat,
    scan_records: u32,
) -> AdmittedRecordAccessPolicy {
    PhysicalRecordAccessPolicy::builder()
        .transfer_limit(RecordByteLimit::new(16_384).unwrap())
        .scratch_limit(RecordByteLimit::new(131_072).unwrap())
        .scan_record_limit(RecordCountLimit::new(scan_records).unwrap())
        .admit(format)
        .unwrap()
}

fn transition_success<T, D>(outcome: TransitionOutcome<T, D>) -> T {
    match outcome {
        TransitionOutcome::Success(value) => value,
        _ => panic!("the courtroom evidence transition must succeed"),
    }
}
