pub(crate) use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::PhysicalPageSizeClass,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
};

pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::append,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::initialize_with_format,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::limits,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::placement_for,
};

pub(crate) fn publish(page: PhysicalPageSizeClass) {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .page_size(page)
            .admit()
            .unwrap(),
    );
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    let resident = page_bytes * 4;
    let serving = initialize_with_format(&root, format, resident);
    let payload = [0xAB; 32];
    let id = append(&serving, placement_for(format), 1, &payload);
    let mut session = serving.records().unwrap().open(id, limits()).unwrap();
    assert_eq!(session.next_chunk().unwrap().unwrap().bytes(), &payload);
    assert_eq!(format.declaration().page_size(), page);
    let peak = serving
        .certification_physical_residency()
        .counters()
        .peak_resident_bytes();
    assert!(peak <= resident, "resident frames peaked at {peak}");
    serving.close();
}
