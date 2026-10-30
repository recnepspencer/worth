//! A syntactically valid, durably WAL-bound expiry cannot invent a later
//! completed checkpoint than the selected Store actually has.

pub(crate) use {std::fs, std::process::Command, std::time::Duration};

pub(crate) use worth_store::physical_runtime::BlobResumeToken;

pub(crate) use {
    crate::physical_blob_journeys::blob_crash,
    worth_store_testkit_store::fixture::serving_from_open,
};

pub(crate) fn recovery_executable() -> std::path::PathBuf {
    worth_store_test_support::FRESH_PROCESS_RECOVERY.path()
}
