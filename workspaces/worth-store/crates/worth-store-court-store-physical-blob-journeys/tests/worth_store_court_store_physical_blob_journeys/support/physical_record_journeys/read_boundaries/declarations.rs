pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::AdmittedRecordAccessPolicy, worth_store::physical_runtime::ExternalPhysicalRecordLocator, worth_store::physical_runtime::PhysicalRecordAccessPolicy, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRootProtocolRoute, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordBootstrapDenial, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadDenial, worth_store::physical_runtime::RecordReadLimits};
pub(crate) use {worth_store_physical_backend::MediaOperationRole};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::durable_publication::root::publish_single, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::serving_from_initialization, worth_store_testkit_store::physical_record_journeys::success};

pub(crate) fn permissive_access() -> (AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy) {
    let (format, _, _) = configuration();
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .transfer_limit(worth_store::physical_runtime::RecordByteLimit::new(u32::MAX).unwrap())
        .scratch_limit(worth_store::physical_runtime::RecordByteLimit::new(u32::MAX).unwrap())
        .admit(format)
        .unwrap();
    (format, access)
}