pub(crate) use {std::path::Path};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::AdmissionError, worth_store::physical_runtime::FilesystemMediaAdmission, worth_store::physical_runtime::MediaShutdownOutcome, worth_store::physical_runtime::ObservationError, worth_store::physical_runtime::PhysicalRuntimeAdmission, worth_store::physical_runtime::PhysicalStore};
pub(crate) use {worth_store_physical_backend::FilesystemAccessPosture};









pub(crate) use {crate::physical_media_journeys::process_contender::run_contender, crate::physical_media_journeys::process_contender::spawn_lease_holder};

pub(crate) fn media_admission() -> FilesystemMediaAdmission {
    worth_store::physical_runtime::FilesystemMediaAdmission::production(worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount)
}

pub(crate) fn admit_runtime(root: &Path) -> worth_store::physical_runtime::AdmittedPhysicalRuntime {
    worth_store::physical_runtime::PhysicalStore::admit(worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap()).unwrap()
}

pub(crate) fn assert_foundational_rows_fail_closed(
    receipt: &worth_store::physical_runtime::certification::StoreMediaPerformanceReceipt,
) {
    use {worth_foundational::performance_api::lower_lane::basis, worth_foundational::performance_api::lower_lane::receipts};
    use {worth_foundational::FoundationalCounterBackedPerformanceReceiptConstructionDenial as Denial, worth_foundational::FoundationalPerformanceCounterRow};

    let missing = receipts::counter_backed_performance_receipt(receipt.bundle().clone()).finish();
    assert!(matches!(missing, Err(Denial::MissingCounterRowForSpec)));

    let first = receipt.counter_rows()[0].clone();
    let duplicate = receipts::counter_backed_performance_receipt(receipt.bundle().clone())
        .attach_counter_row(first.clone())
        .attach_counter_row(first)
        .finish();
    assert!(matches!(duplicate, Err(Denial::DuplicateCounterRow)));

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

    let mut unexpected = receipts::counter_backed_performance_receipt(receipt.bundle().clone());
    for row in receipt.counter_rows() {
        unexpected = unexpected.attach_counter_row(row.clone());
    }
    let extra = basis::FoundationalPerformanceCounterName::new("store.media.unexpected").unwrap();
    let unexpected = unexpected
        .attach_counter_row(FoundationalPerformanceCounterRow::new(extra, 0))
        .finish();
    assert!(matches!(unexpected, Err(Denial::UnexpectedCounterRow)));
}