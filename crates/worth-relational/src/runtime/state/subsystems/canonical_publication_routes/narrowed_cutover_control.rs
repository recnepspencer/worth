//! Scoped scheduling control over the original canonical reservation.
impl crate::runtime::RelationalRuntime {
    /// Hold the exact reservation lifecycle transitions use. This creates no
    /// transition or publication fact; dropping the callback releases the hold.
    #[doc(hidden)]
    pub fn with_transition_reservation_for_test<R>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        run: impl FnOnce() -> R,
    ) -> Result<R, worth_execution::WorkCeilingDenial> {
        let routes = self.history.canonical_checkpoint_gate();
        routes.hold_transition_reservation_for_test(execution, run)
    }

    /// Read the actual queued publisher tickets without issuing any authority.
    #[doc(hidden)]
    pub fn waiting_patch_position_publishers_for_test(&self) -> usize {
        self.history
            .canonical_checkpoint_gate()
            .waiting_publisher_count_for_test()
    }
}
