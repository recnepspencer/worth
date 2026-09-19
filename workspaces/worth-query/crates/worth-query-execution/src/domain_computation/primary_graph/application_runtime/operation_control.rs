//! Feature-only access to the installed World's real operation boundary.

use super::WorthQueryPrimaryGraphApplicationRuntime;

mod attempt_registration;
pub(in crate::domain_computation::primary_graph) use attempt_registration::WorthQueryApplicationAttemptOperationControl;
pub use attempt_registration::WorthQueryApplicationAttemptRegistrationPause;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Parks real provider-registered application attempts for certification.
    #[doc(hidden)]
    pub fn pause_after_application_attempt_registration_for_test(
        &self,
        attempts: std::num::NonZeroUsize,
    ) -> WorthQueryApplicationAttemptRegistrationPause {
        self.primary_provider
            .application_attempt_operation_control
            .pause_after_registration(attempts)
    }

    /// Returns deterministic scheduling control for certification builds.
    ///
    /// The control can only park real World publication transitions. It cannot
    /// create an attempt, publish a product, or mint application authority.
    #[doc(hidden)]
    pub fn world_operation_control_for_test(
        &self,
    ) -> worth_runtime_world::facade::RuntimeWorldOperationControl {
        self.product_runtime.operation_control()
    }

    /// Inspects the World's active-attempt obligations in this installed
    /// runtime in certification builds.
    #[doc(hidden)]
    pub fn world_active_publication_attempts_for_test(&self) -> usize {
        self.world_retention_snapshot_for_test()
            .active_publication_attempts()
    }

    /// Returns the number of real registered application attempts, including
    /// attempts waiting for the provider's product-publication turn.
    #[doc(hidden)]
    pub fn active_application_attempts_for_test(&self) -> usize {
        self.primary_provider
            .active_application_attempt_count_for_test()
    }

    /// Returns the number of subscriptions held by real open live consumers.
    #[doc(hidden)]
    pub fn active_live_consumers_for_test(&self) -> usize {
        self.primary_provider.active_live_consumer_count_for_test()
    }

    /// Samples the installed World's real retention owner in certification builds.
    #[doc(hidden)]
    pub fn world_retention_snapshot_for_test(
        &self,
    ) -> worth_runtime_world::facade::RuntimeWorldRetentionSnapshot {
        self.product_runtime
            .owner
            .inspection_port()
            .retention_snapshot()
            .expect("the installed World remains inspectable")
    }

    /// Samples the installed World's real history owner in certification builds.
    #[doc(hidden)]
    pub fn world_history_snapshot_for_test(
        &self,
    ) -> worth_runtime_world::facade::RuntimeWorldHistorySnapshot {
        self.product_runtime
            .owner
            .inspection_port()
            .history_snapshot()
            .expect("the installed World remains inspectable")
    }

    /// Counts branches installed in Query's bounded activation index.
    #[doc(hidden)]
    pub fn indexed_product_branch_count_for_test(&self) -> usize {
        self.product_runtime.activations.installed_branch_count()
    }

    /// Counts definitions retained by the sealed Bridge owner.
    #[doc(hidden)]
    pub fn installed_conditional_definition_count_for_test(&self) -> usize {
        self.bridge
            .conditional()
            .installed_conditional_definition_count()
    }

    /// Counts Bridge target references retained by installed definitions.
    #[doc(hidden)]
    pub fn installed_conditional_target_reference_count_for_test(&self) -> usize {
        self.bridge
            .conditional()
            .installed_conditional_target_reference_count()
    }

    /// Counts active Signal nodes observed through the sealed Bridge owner.
    #[doc(hidden)]
    pub fn owned_signal_active_node_count_for_test(&self) -> usize {
        self.bridge
            .conditional()
            .owned_signal_active_node_count()
            .expect("the installed Signal owner remains inspectable")
    }
}
