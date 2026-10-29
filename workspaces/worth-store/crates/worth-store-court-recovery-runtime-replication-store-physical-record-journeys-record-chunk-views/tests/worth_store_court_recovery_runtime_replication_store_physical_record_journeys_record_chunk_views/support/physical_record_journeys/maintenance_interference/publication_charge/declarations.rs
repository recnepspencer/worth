pub(crate) use {std::num::NonZeroU32, std::num::NonZeroU64};

pub(crate) use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::PhysicalDataDispatchOutcome, worth_store::physical_runtime::PhysicalManifestCapacityTransition, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalRecordAccessPolicy, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome, worth_store::physical_runtime::PhysicalWalPolicy, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::WalSegmentByteLimit, worth_store::physical_runtime::WalSegmentInventoryLimit};
pub(crate) use {worth_store_physical_format::BOOTSTRAP_CATALOG_BYTES, worth_store_physical_format::DURABLE_FRAME_HEADER_BYTES, worth_store_physical_format::ROOT_SELECTOR_BYTES};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::maintenance_interference::reopen::open_interference};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::maintenance_interference::append, worth_store_testkit_store::physical_record_journeys::maintenance_interference::residency, worth_store_testkit_store::physical_record_journeys::maintenance_interference::RESIDENT_BYTES};


pub(crate) use {worth_store_testkit_store::publication_metadata_oracle as oracle};

pub(crate) static EXTENT_SEED: [u8; 20_000] = [0x51; 20_000];

pub(crate) fn require_dispatched(
    outcome: PhysicalDataDispatchOutcome,
) -> worth_store::physical_runtime::DataDispatchedPhysicalMutation {
    match outcome {
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Suspended(retry) => {
            panic!("group data suspended: {:?}", retry.cause())
        }
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::NotStarted { cause, .. } => {
            panic!("group data not started: {cause:?}")
        }
        worth_store::physical_runtime::PhysicalDataDispatchOutcome::Indeterminate(failure) => {
            panic!("group data indeterminate: {:?}", failure.cause())
        }
    }
}

pub(crate) fn prepared(
    submission: &worth_store::physical_runtime::PhysicalRecordSubmission,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    identity: [u8; 32],
    bytes: &[u8],
) -> worth_store::physical_runtime::PreparedPhysicalMutation {
    let outcome = worth_store_testkit_store::durable_publication::wal_data::prepare_single(
        submission,
        placement,
        worth_store::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(identity),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([bytes]).unwrap(),
    );
    match outcome.into_raw() {
        worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(value)) => value,
        _ => panic!("group member not prepared"),
    }
}

/// This is only the old conservative reservation oracle for choosing a finite
/// test budget. Actual metadata comes from literal emitted file lengths above.
pub(crate) fn full_member_metadata_ceiling(entries: u64) -> u64 {
    let capacity = 2_u64;
    let leaves = entries.div_ceil(capacity);
    let mut branches = 0;
    let mut nodes = leaves;
    while nodes > 1 {
        nodes = nodes.div_ceil(capacity);
        branches += nodes;
    }
    let header = DURABLE_FRAME_HEADER_BYTES as u64 + 40;
    let tree = |leaf_width, branch_width| {
        leaves * (header + capacity * leaf_width) + branches * (header + capacity * branch_width)
    };
    let fixed = (DURABLE_FRAME_HEADER_BYTES as u64 + 336)
        + (DURABLE_FRAME_HEADER_BYTES as u64 + 168)
        + (2 * ROOT_SELECTOR_BYTES + BOOTSTRAP_CATALOG_BYTES) as u64;
    fixed + tree(88, 72) + tree(40, 56) + tree(40, 72)
}