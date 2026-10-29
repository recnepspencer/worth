pub(crate) use {worth_foundational::compare_canonical_basis, worth_foundational::prepare_canonical_basis_sequence, worth_foundational::prepare_canonical_comparison, worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisValue, worth_foundational::CanonicalComparisonOutcome, worth_foundational::CanonicalEquivalenceBasis, worth_foundational::CanonicalIntegerWidth, worth_foundational::CanonicalMismatchKind};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::lower_record_operation_performance_receipt, worth_store::physical_runtime::lower_record_publication_canonical_basis, worth_store::physical_runtime::PhysicalRecordAccessSummary, worth_store::physical_runtime::PhysicalRecordPerformanceContract, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordLocatePerformanceExpectation, worth_store::physical_runtime::RecordManifestPerformanceExpectation, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanPerformanceExpectation, worth_store::physical_runtime::RecordScanRequest, worth_store::physical_runtime::RecordTransferPerformanceExpectation};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::durable_publication, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

pub(crate) fn zero_locate_expectation() -> RecordLocatePerformanceExpectation {
    RecordLocatePerformanceExpectation {
        payload_bytes: 0,
        manifest: manifest_expectation(0, 0, 0),
        transfer: transfer_expectation(0, 0, 0, 0, 0),
        frames_traversed: 0,
    }
}

pub(crate) const fn manifest_expectation(
    blocks: u64,
    bytes: u64,
    comparisons: u64,
) -> RecordManifestPerformanceExpectation {
    RecordManifestPerformanceExpectation {
        blocks,
        bytes,
        comparisons,
    }
}

pub(crate) const fn transfer_expectation(
    transfers: u64,
    peak_transfer_bytes: u64,
    explicit_copies: u64,
    copied_bytes: u64,
    peak_scratch_bytes: u64,
) -> RecordTransferPerformanceExpectation {
    RecordTransferPerformanceExpectation {
        transfers,
        peak_transfer_bytes,
        explicit_copies,
        copied_bytes,
        peak_scratch_bytes,
    }
}

pub(crate) fn compare(
    left: worth_foundational::CanonicalBasisReadyArtifact,
    right: worth_foundational::CanonicalBasisReadyArtifact,
) -> CanonicalComparisonOutcome {
    let ready = success(prepare_canonical_comparison(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ));
    compare_canonical_basis(&ready)
}

pub(crate) fn success<T, D>(outcome: TransitionOutcome<T, D>) -> T {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        _ => panic!("evidence construction failed"),
    }
}

pub(crate) fn assert_rows_fail_closed(receipt: &worth_store::physical_runtime::StoreRecordPerformanceReceipt) {
    use {worth_foundational::performance_api::lower_lane::receipts};
    use {worth_foundational::FoundationalCounterBackedPerformanceReceiptConstructionDenial as Denial, worth_foundational::FoundationalPerformanceCounterRow};

    assert!(matches!(
        receipts::counter_backed_performance_receipt(receipt.bundle().clone()).finish(),
        Err(Denial::MissingCounterRowForSpec)
    ));
    let first = receipt.counter_rows()[0].clone();
    assert!(matches!(
        receipts::counter_backed_performance_receipt(receipt.bundle().clone())
            .attach_counter_row(first.clone())
            .attach_counter_row(first)
            .finish(),
        Err(Denial::DuplicateCounterRow)
    ));
    let mut mismatched = receipts::counter_backed_performance_receipt(receipt.bundle().clone());
    for (index, row) in receipt.counter_rows().iter().enumerate() {
        mismatched = mismatched.attach_counter_row(FoundationalPerformanceCounterRow::new(
            row.name().clone(),
            row.observed_count() + u64::from(index == 0),
        ));
    }
    assert!(matches!(
        mismatched.finish(),
        Err(Denial::CounterValueMismatch)
    ));
}