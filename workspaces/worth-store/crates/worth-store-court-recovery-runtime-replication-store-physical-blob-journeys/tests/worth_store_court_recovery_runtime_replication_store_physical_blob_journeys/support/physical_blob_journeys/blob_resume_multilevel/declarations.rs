pub(crate) use {std::fs, std::num::NonZeroU64, std::time::Duration};

pub(crate) use {worth_store::physical_runtime::BlobResumeLimits, worth_store::physical_runtime::BlobResumeToken, worth_store::physical_runtime::PhysicalMutationDeadline};

pub(crate) use {crate::physical_blob_journeys::blob_crash::kill_at, crate::blob_crash::recovery::recover_closed_store_with_profile, crate::physical_blob_journeys::blob_crash::resume_token_path, crate::physical_blob_journeys::blob_crash::SCOPE_KEY, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_open};





pub(crate) use {};

// This scheduled proof serially syncs 4,097 real C.5 chunk publications on
// filesystem media before the child can reach its selected-interior marker.
// Keep the process watchdog and both mutation deadlines on the same bounded
// horizon; a 75-minute run missed the marker under competing I/O on Windows.
pub(crate) const SCHEDULED_WORKLOAD_MILLIS: u64 = 120 * 60 * 1_000;