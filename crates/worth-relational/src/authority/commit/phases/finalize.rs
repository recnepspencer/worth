#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct PublicationPhaseTiming {
    pub(crate) storage_commit_micros: u64,
    pub(crate) index_refresh_micros: u64,
    pub(crate) history_publish_micros: u64,
    pub(crate) visibility_pin_micros: u64,
    pub(crate) retention_reconcile_micros: u64,
    pub(crate) compaction_micros: u64,
    pub(crate) bundle_publish_micros: u64,
    pub(crate) retention_pass_micros: u64,
    pub(crate) post_commit_consumer_micros: u64,
}
