pub(crate) const MODEL: LifecycleMaelstromModel = LifecycleMaelstromModel {
    disjoint_read_effects: 2,
    retry_write_attempts: 2,
    retry_write_completions: 1,
    exact_writeback_effects: 1,
    fresh_root_generation: 1,
    fresh_records: &[],
};

pub(crate) struct LifecycleMaelstromModel {
    pub(crate) disjoint_read_effects: u64,
    pub(crate) retry_write_attempts: u64,
    pub(crate) retry_write_completions: u64,
    pub(crate) exact_writeback_effects: u64,
    pub(crate) fresh_root_generation: u64,
    pub(crate) fresh_records: &'static [&'static [u8]],
}
