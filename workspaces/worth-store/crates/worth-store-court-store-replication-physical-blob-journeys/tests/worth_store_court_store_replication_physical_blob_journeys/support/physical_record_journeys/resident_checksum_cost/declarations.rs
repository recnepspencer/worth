pub(crate) use worth_store_physical_format::certification_crc32c_invocations;
pub(crate) use {
    worth_store::physical_runtime::ExternalPhysicalRecordLocator,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

pub(crate) use {
    worth_store_testkit_store::durable_publication,
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::read_record,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
    worth_store_testkit_store::physical_record_journeys::serving_from_open,
};

pub(crate) fn read_and_count_checksums(
    serving: &ServingPhysicalRuntime,
    locator: ExternalPhysicalRecordLocator,
    expected: &[u8],
) -> u64 {
    let before = certification_crc32c_invocations();
    let session = serving
        .records()
        .expect("read protection admission")
        .open_external(
            locator,
            worth_store::physical_runtime::RecordReadLimits::new(
                worth_store::physical_runtime::RecordByteLimit::new(1_024).unwrap(),
            ),
        )
        .unwrap();
    assert_eq!(read_record(session, expected.len()).0, expected);
    certification_crc32c_invocations() - before
}
