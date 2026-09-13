impl super::RelationalRuntime {
    /// Arms one deterministic append failure for boundary-level recovery
    /// tests. This surface exists only when the explicit test feature is set.
    #[doc(hidden)]
    pub fn fail_next_durable_append_for_test(&self) {
        self.durability.arm_append_failure();
    }
}
