pub use {std::path::Path};

pub use {worth_signal::facade::TemporalDuration};
pub use {worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRecordId, worth_store::physical_runtime::PhysicalRetirementDenial, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::ServingPhysicalRuntime};
pub use {worth_store_physical_backend::MediaOperationRole};

pub use {crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::produced_retirement_payloads, crate::physical_record_journeys::durability_admission::independent_wal_oracle::rewrite_redo::produced_rewrite_payloads, crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind, crate::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction};
pub use {worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause, worth_store::physical_runtime::PreparedPhysicalMutation, worth_proof::TransitionOutcome, crate::physical_record_journeys::durability_admission::managed_mutation::completed, crate::physical_record_journeys::configuration, crate::physical_record_journeys::durability_admission::managed_mutation::prepare, crate::physical_record_journeys::serving_from_initialization};
pub use {crate::physical_record_journeys::manifest_fixture::current_extent_route, crate::physical_record_journeys::manifest_fixture::IndependentExtentRoute};
/// Larger than one page, so the record lives in a multi-chunk extent.
pub const EXTENT_PAYLOAD_BYTES: usize = 40_000;











pub use {crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::extent_frame_fixture::arena_file, crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::extent_frame_fixture::arena_range_bytes, crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::extent_frame_fixture::extent_payload};

pub fn prepare_extent_rewrite(
    serving: &ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: [u8; 32],
    record: PhysicalRecordId,
) -> PreparedPhysicalMutation {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    match submission
        .rewrite_selected_extent_record(placement, request(key), record)
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("an extent record rewrite must prepare"),
    }
}

pub fn request(
    key: worth_store::physical_runtime::PhysicalMutationIdempotencyKey,
) -> PhysicalMutationRequest {
    worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
        key,
        worth_store::physical_runtime::PhysicalMutationDeadline::at(worth_signal::facade::TemporalDuration::temporal_duration(1_000).unwrap()),
    )
}

pub fn read_record(serving: &ServingPhysicalRuntime, record: PhysicalRecordId) -> Vec<u8> {
    let reader = serving.records().unwrap();
    read_from_reader(&reader, record)
}

pub fn read_from_reader(
    reader: &worth_store::physical_runtime::PhysicalRecordReader,
    record: PhysicalRecordId,
) -> Vec<u8> {
    let mut session = reader
        .open(
            record,
            worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(EXTENT_PAYLOAD_BYTES as u32).unwrap()),
        )
        .unwrap();
    let mut bytes = Vec::new();
    let mut buffer = vec![0_u8; 4096];
    loop {
        let count = session.read_next(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
    bytes
}