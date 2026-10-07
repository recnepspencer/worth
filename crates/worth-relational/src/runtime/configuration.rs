use super::RelationalRuntime;

impl RelationalRuntime {
    /// Put a durability mode in force for a runtime this crate is rebuilding.
    ///
    /// Recovery replays against the in-memory canonical log and restores the
    /// configured mode once the rebuilt runtime is finalized. It is owner
    /// authority like any other reconfiguration, so it goes through the same
    /// single installation route rather than reaching into the configuration.
    pub(crate) fn set_durability_mode(&mut self, mode: crate::durability::data::DurabilityMode) {
        self.reconfigure(|configuration| configuration.set_durability_mode(mode));
    }
}
