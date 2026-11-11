//! Ordinary home journeys: create, commit, close, reopen, continue.

pub(crate) use worth_execution::ExecutionRequest;

pub(crate) use {
    crate::durable_run::apply, crate::durable_run::assert_holds, crate::durable_run::open,
    crate::durable_run::steps_after, crate::durable_run::steps_before,
};
pub(crate) use {crate::workload, crate::workload::Script};

pub(crate) const SEED: u64 = 0x5eed_0201;
pub(crate) const COMMITS: usize = 18;
