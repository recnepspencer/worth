//! Two independently terminal custody attempts must clean one at a time.

pub(crate) use {
    std::num::NonZeroU16, std::num::NonZeroU64, std::thread, std::time::Duration,
    std::time::Instant,
};

pub(crate) use {
    worth_store::physical_runtime::production::PhysicalMutationCheckpoint,
    worth_store::physical_runtime::production::PhysicalMutationPauseGate,
    worth_store::physical_runtime::BlobAppendFailure,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimFailure,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause,
};
pub(crate) use {
    worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::PersistedRecordIdentity,
};

pub(crate) use {
    crate::physical_blob_journeys::blob_reclaim::abandoned_prefix,
    crate::physical_blob_journeys::blob_reclaim::request,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

/// Waits for the paused mutation to arrive at `gate`. A single bounded wait can
/// expire under parallel load and leave the gate holding the mutation with no
/// one to release it; the same long deadline the crash journeys use keeps the
/// arrival deterministic.
pub(crate) fn arrives(gate: &PhysicalMutationPauseGate) -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while std::time::Instant::now() < deadline {
        if gate.await_arrival() {
            return true;
        }
    }
    false
}
