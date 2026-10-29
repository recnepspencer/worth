pub(crate) use {std::fs};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_signal::facade::TemporalDuration};
pub(crate) use {worth_store::physical_runtime::PhysicalCheckpointCaptureFailureKind, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointProvenNoEffectCause, worth_store::physical_runtime::PhysicalCheckpointRequest};
pub(crate) use {worth_store_physical_format::DurablePhysicalRootManifest, worth_store_physical_format::PhysicalCheckpointSource, worth_store_physical_format::CHECKPOINT_STREAM_HEADER_RECORD_BYTES};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::selected_segment_rewrite::prepare_rewrite};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

pub(crate) fn newest_root(root: &std::path::Path) -> std::path::PathBuf {
    std::fs::read_dir(root.join("families/records/roots"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("manifest"))
        .max()
        .unwrap()
}

pub(crate) fn checkpoint_request() -> PhysicalCheckpointRequest {
    worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([63; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(1_000).expect("deadline is positive"),
        ),
    )
}