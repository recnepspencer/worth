//! Sealed test doors over this module's private members.
use super::*;

impl ErasedClockObservationOutcome {
    #[doc(hidden)]
    pub fn typed_for_test<Clock>(
        self,
        installation: crate::domain_computation::primary_graph::WorthQueryGranularInvalidationInstallation,
        source_read_basis: Option<
            crate::domain_computation::primary_graph::WorthQueryGranularSourceReadBasis,
        >,
    ) -> super::super::WorthQueryConditionalClockObservationOutcome<Clock> {
        Self::typed::<Clock>(self, installation, source_read_basis)
    }
}

impl ErasedClockObservationReceipt {
    #[doc(hidden)]
    pub fn typed_for_test<Clock>(
        self,
        granular_invalidation_installation: crate::domain_computation::primary_graph::WorthQueryGranularInvalidationInstallation,
        granular_source_read_basis: Option<
            crate::domain_computation::primary_graph::WorthQueryGranularSourceReadBasis,
        >,
    ) -> super::super::WorthQueryConditionalClockObservationReceipt<Clock> {
        Self::typed::<Clock>(
            self,
            granular_invalidation_installation,
            granular_source_read_basis,
        )
    }
}

impl ErasedClockObservationReceipt {
    #[doc(hidden)]
    pub fn new_for_test(
        sequence: u64,
        observed_coordinate: u64,
        due_wake_count: usize,
        due_work_remaining: bool,
        authoritative_commit_count: usize,
        authoritative_work_remaining: bool,
        retained_due_wake_count: usize,
        retained_eligible_wake_count: usize,
        retained_suppressed_wake_count: usize,
        retained_deferred_wake_count: usize,
        retained_failed_wake_count: usize,
        committed_operation_count: usize,
        already_committed_operation_count: usize,
        failed_operation_count: usize,
        indeterminate_operation_count: usize,
        snapshot_capacity_backpressure: Option<usize>,
        retention_capacity_backpressure: bool,
        execution_provenance: Vec<super::super::super::WorthQueryConditionalExecutionProvenance>,
        granular_invalidations: Vec<
            worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery,
        >,
        granular_invalidation_coverage: crate::domain_computation::primary_graph::WorthQueryGranularInvalidationCoverage,
    ) -> Self {
        Self {
            sequence,
            observed_coordinate,
            due_wake_count,
            due_work_remaining,
            authoritative_commit_count,
            authoritative_work_remaining,
            retained_due_wake_count,
            retained_eligible_wake_count,
            retained_suppressed_wake_count,
            retained_deferred_wake_count,
            retained_failed_wake_count,
            committed_operation_count,
            already_committed_operation_count,
            failed_operation_count,
            indeterminate_operation_count,
            snapshot_capacity_backpressure,
            retention_capacity_backpressure,
            execution_provenance,
            granular_invalidations,
            granular_invalidation_coverage,
        }
    }
}
