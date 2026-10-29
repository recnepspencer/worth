pub(crate) use {std::path::Path};
pub(crate) use {std::thread};

pub(crate) use {worth_store::physical_runtime::certification::CertificationReadRootCaptureStage, worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::PhysicalRetirementDenial, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::SegmentPageCount};
pub(crate) use {worth_store_physical_format::PhysicalCheckpointSource, worth_store_physical_format::CHECKPOINT_STREAM_HEADER_RECORD_BYTES};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::produced_retirement_payloads, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction};
pub(crate) use {worth_store_testkit_store::durability_admission::managed_mutation::published_segments::segment_names};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::selected_segment_rewrite::prepare_rewrite};
pub(crate) use {worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare_records, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::retirement_charge_oracle::assert_one_page_released_net_of_wal, worth_store_testkit_store::physical_record_journeys::retirement_charge_oracle::wal_bytes};

pub(crate) fn checkpoint_root_generation(root: &Path) -> u64 {
    let bytes = std::fs::read(root.join("families").join("checkpoint.current")).unwrap();
    let header: [u8; CHECKPOINT_STREAM_HEADER_RECORD_BYTES] = bytes
        [..CHECKPOINT_STREAM_HEADER_RECORD_BYTES]
        .try_into()
        .unwrap();
    worth_store_physical_format::PhysicalCheckpointSource::decode_stream_header_record(&header)
        .unwrap()
        .root()
        .generation()
}

pub(crate) fn retirement_actions(root: &Path) -> Vec<IndependentRetirementAction> {
    produced_retirement_payloads(root)
        .iter()
        .map(|record| record.action)
        .collect()
}