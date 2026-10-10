impl super::RelationalRuntime {
    /// Arms one deterministic append failure for boundary-level recovery
    /// tests through this runtime's in-process handle.
    #[doc(hidden)]
    pub fn fail_next_durable_append_for_test(&self) {
        self.durability.arm_append_failure();
    }
}
