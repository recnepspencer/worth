#[allow(unused_imports)] use crate::physical_record_journeys::foundational_evidence::*;
use {worth_foundational::compare_canonical_basis, worth_foundational::prepare_canonical_basis_sequence, worth_foundational::prepare_canonical_comparison, worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalComparisonOutcome, worth_foundational::CanonicalEquivalenceBasis, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalMismatchKind};
use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::lower_record_operation_performance_receipt, worth_store::physical_runtime::lower_record_publication_canonical_basis, worth_store::physical_runtime::PhysicalRecordAccessSummary, worth_store::physical_runtime::PhysicalRecordPerformanceContract, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordLocatePerformanceExpectation, worth_store::physical_runtime::RecordManifestPerformanceExpectation, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanPerformanceExpectation, worth_store::physical_runtime::RecordScanRequest, worth_store::physical_runtime::RecordTransferPerformanceExpectation};

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::durable_publication, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

#[test]
fn runtime_and_offline_topology_have_canonical_parity() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, _) = configuration();
    let serving = serving_from_initialization(&root);
    let inline = (0..70).map(|ordinal| vec![ordinal as u8; 257]);
    worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material("foundational-topology-parity", 1),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter(inline).unwrap(),
    );
    worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material("foundational-topology-parity", 2),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([vec![0x91; 20_000]]).unwrap(),
    );

    let runtime = serving.certification_publication_summary().unwrap();
    let offline = worth_store_offline_verifier::walk_current_durable_record_manifest(
        &root,
        format.declaration(),
    )
    .unwrap();
    let runtime = success(lower_record_publication_canonical_basis(&runtime));
    let offline = success(
        worth_store_offline_verifier::lower_offline_record_publication_canonical_basis(&offline),
    );
    assert!(matches!(
        compare(runtime.clone(), offline),
        worth_foundational::CanonicalComparisonOutcome::Equivalent(_)
    ));

    let mut changed = runtime.payload().entries().to_vec();
    let target = changed
        .iter_mut()
        .find(|entry| entry.locus() == &worth_foundational::CanonicalBasisLocus::Named("root.generation".into()))
        .unwrap();
    *target = worth_foundational::CanonicalBasisEntry::new(
        target.domain(),
        target.locus().clone(),
        target.kind(),
        worth_foundational::CanonicalBasisValue::UnsignedInteger {
            width: worth_foundational::CanonicalIntegerWidth::Bits64,
            value: 99,
        },
    );
    let divergent = success(prepare_canonical_basis_sequence(
        runtime.payload().version().clone(),
        runtime.payload().domain(),
        changed,
    ));
    match compare(runtime, divergent) {
        worth_foundational::CanonicalComparisonOutcome::Mismatched(mismatch) => {
            assert_eq!(mismatch.kind(), worth_foundational::CanonicalMismatchKind::ValueMismatch);
            assert_eq!(
                mismatch.left_locus(),
                Some(&worth_foundational::CanonicalBasisLocus::Named("root.generation".into()))
            );
        }
        other => panic!("one-field topology divergence escaped: {other:?}"),
    }
    serving.close();
}

#[test]
fn counter_receipt_rejects_missing_duplicate_and_mismatched_rows() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (_, placement, _) = configuration();
    let serving = serving_from_initialization(&root);
    let published = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material("foundational-counter-receipt", 1),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"counter receipt".as_slice()]).unwrap(),
    );
    let record = published.settled_members()[0].record_id(0).unwrap();
    serving
        .certification_physical_residency()
        .drain_unpinned_clean_frames();
    let mut read = serving
        .records()
        .expect("read protection admission")
        .open(
            record,
            worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(128).unwrap()),
        )
        .unwrap();
    let mut scratch = [0_u8; 64];
    while read.read_next(&mut scratch).unwrap() != 0 {}
    let read = worth_store::physical_runtime::PhysicalRecordAccessSummary::from_completed_read(read.observation()).unwrap();
    assert!(matches!(
        lower_record_operation_performance_receipt(
            worth_store::physical_runtime::PhysicalRecordPerformanceContract::locate(zero_locate_expectation()),
            read,
        ),
        Err(worth_store::physical_runtime::RecordPerformanceEvidenceDenial::Receipt(_))
    ));
    let read = lower_record_operation_performance_receipt(
        worth_store::physical_runtime::PhysicalRecordPerformanceContract::locate(RecordLocatePerformanceExpectation {
            payload_bytes: 15,
            manifest: manifest_expectation(
                2,
                208 + 2 * worth_store_testkit_store::durable_frame_oracle::HEADER_BYTES as u64,
                2,
            ),
            transfer: transfer_expectation(1, 16_384, 1, 15, 0),
            frames_traversed: 3,
        }),
        read,
    )
    .unwrap();
    assert_eq!(read.counter_rows().len(), 20);
    assert_rows_fail_closed(&read);

    let mut scan = serving
        .records()
        .expect("read protection admission")
        .scan(worth_store::physical_runtime::RecordScanRequest::from_start())
        .unwrap();
    let completed = loop {
        match scan.read_next_into(&mut scratch).unwrap() {
            worth_store::physical_runtime::RecordScanOutcome::Batch(_) => {}
            worth_store::physical_runtime::RecordScanOutcome::Completed(completed) => break completed,
        }
    };
    let scan = worth_store::physical_runtime::PhysicalRecordAccessSummary::from_completed_scan(completed);
    lower_record_operation_performance_receipt(
        worth_store::physical_runtime::PhysicalRecordPerformanceContract::scan(RecordScanPerformanceExpectation {
            records: 1,
            payload_bytes: 15,
            manifest: manifest_expectation(
                2,
                208 + 2 * worth_store_testkit_store::durable_frame_oracle::HEADER_BYTES as u64,
                1,
            ),
            transfer: transfer_expectation(1, 16_384, 1, 15, 0),
            frames_traversed: 3,
        }),
        scan,
    )
    .unwrap();
    serving.close();
}
