//! What an advancement runs against: the World owner and relational source whose phases it
//! accepts, so request execution never names the runtimes, runs and observations that lend them.

/// A runtime, run or observation that lends and accepts advancement phases.
pub trait WorthQueryAdvancementSource {
    /// The World owner whose phases this source accepts, if it has one.
    fn advancement_owner(&self) -> Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>;
    /// The relational runtime instance this source's phases read.
    fn advancement_source_instance(&self) -> u64;
}
