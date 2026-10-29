pub(crate) use {std::collections::BTreeSet};
pub(crate) use {std::fs};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::durable_publication, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};
pub(crate) use {crate::durability_admission::data_durability::mutation_world::append, crate::durability_admission::data_durability::mutation_world::synchronize};
pub(crate) use {crate::physical_record_journeys::durability_admission::data_durability::oracle::assert_encoded_wal_matches_claims, crate::physical_record_journeys::durability_admission::data_durability::oracle::assert_targets_absent, crate::physical_record_journeys::durability_admission::data_durability::oracle::frame_page_lsn_matches_basis, crate::physical_record_journeys::durability_admission::data_durability::oracle::verify_effect};
pub(crate) use {worth_store::physical_runtime::CertifiedPriorPageImage, worth_store::physical_runtime::PhysicalDataDispatchOutcome, worth_store::physical_runtime::PhysicalDataEffectSource, worth_store::physical_runtime::PhysicalDataFrameSubject, worth_store::physical_runtime::PhysicalDataSettlementOutcome, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::RecordAppendBatch};
pub(crate) use {worth_store_physical_format::encode_data_frame_page_lsn, worth_store_physical_format::DurableFrameKind, worth_store_physical_format::PhysicalPageLsn};

pub(crate) fn only_data_artifact(store_root: &std::path::Path, family: &str) -> std::path::PathBuf {
    let mut artifacts = std::fs::read_dir(store_root.join("families").join("records").join(family))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    artifacts.sort();
    assert_eq!(artifacts.len(), 1, "fixture requires one source artifact");
    artifacts.pop().unwrap()
}