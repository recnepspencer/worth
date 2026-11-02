//! Certification-only fault and occupancy controls over the installed Bank
//! runtime. Controls expose counts, faults or their owned scheduling guard; none returns a Query runtime,
//! so no caller can redeem a commit receipt through them.

// Certification-only faults/backstops: only the sealed *_for_test doors arm
// these controls. The boundary guard closes ordinary callers; their intentional
// panic reports a violated test scenario, never an ordinary product refusal.
use super::BankIdentityRuntime;

impl BankIdentityRuntime {
    /// Counts the subscriptions held by real open live consumers.
    #[doc(hidden)]
    pub fn active_live_consumers_for_test(&self) -> usize {
        self.application_program()
            .runtime()
            .active_live_consumers_for_test()
    }

    /// Delays the next output-readiness delivery, once.
    #[doc(hidden)]
    pub fn delay_next_output_readiness_delivery_for_test(&self) {
        self.application_program()
            .runtime()
            .delay_next_output_readiness_delivery_for_test();
    }

    /// Drops the rebuildable workflow-instance progress projection.
    #[doc(hidden)]
    pub fn release_workflow_instance_progress_for_test(&self) {
        self.application_program()
            .runtime()
            .release_workflow_instance_progress_for_test();
    }

    /// Unwinds the next World product comparison, once.
    #[doc(hidden)]
    pub fn panic_before_product_compare_once_for_test(&self) {
        self.application_program()
            .runtime()
            .world_operation_control_for_test()
            .panic_before_product_compare_once_for_test();
    }
}

impl BankIdentityRuntime {
    /// Drops only reconstructible index generations; returns the declared inventory.
    #[doc(hidden)]
    pub fn release_primary_index_generations_for_test(&self) -> usize {
        let application = self.application_program().runtime();
        application
            .with_host_advancement(|phase| {
                application
                    .release_primary_index_generations_for_test(&phase)
                    .expect("control borrows its own runtime")
            })
            .expect("control opens its host call")
    }
}

impl BankIdentityRuntime {
    /// Passive build/update callback attempts and changed occurrences. No handles escape.
    #[doc(hidden)]
    pub fn account_index_work_for_test(&self) -> (usize, usize, usize) {
        self.account_index_work.snapshot()
    }
}

impl BankIdentityRuntime {
    /// Schedules the actual receiver's next managed cleanup on a foreign worker.
    /// The guard grants no business result or substitute cleanup owner.
    #[doc(hidden)]
    pub fn schedule_next_managed_cleanup_on_worker_for_test(
        &self,
    ) -> Result<
        worth_query_host::facade::primary_graph::WorthQueryManagedCleanupScheduleForTest,
        worth_query_host::facade::primary_graph::WorthQueryManagedCleanupRegistrationDenial,
    > {
        self.application_program()
            .runtime()
            .schedule_next_managed_cleanup_on_worker_for_test()
    }
}
