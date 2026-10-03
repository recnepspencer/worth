use worth_query_installation::facade::WorthQueryClockCoordinate;

pub(in crate::domain_computation::primary_graph) struct ErasedClockObservationReceipt {
    pub(in crate::domain_computation::primary_graph::conditional_operation) sequence: u64,
    pub(in crate::domain_computation::primary_graph::conditional_operation) observed_coordinate:
        u64,
    pub(in crate::domain_computation::primary_graph::conditional_operation) due_wake_count: usize,
    pub(in crate::domain_computation::primary_graph::conditional_operation) due_work_remaining:
        bool,
    pub(in crate::domain_computation::primary_graph::conditional_operation) authoritative_commit_count:
        usize,
    pub(in crate::domain_computation::primary_graph::conditional_operation) authoritative_work_remaining:
        bool,
    pub(in crate::domain_computation::primary_graph::conditional_operation) retained_due_wake_count:
        usize,
    pub(in crate::domain_computation::primary_graph::conditional_operation) retained_eligible_wake_count:
        usize,
    pub(in crate::domain_computation::primary_graph::conditional_operation) retained_suppressed_wake_count:
        usize,
    pub(in crate::domain_computation::primary_graph::conditional_operation) retained_deferred_wake_count:
        usize,
    pub(in crate::domain_computation::primary_graph::conditional_operation) retained_failed_wake_count:
        usize,
    pub(in crate::domain_computation::primary_graph::conditional_operation) committed_operation_count:
        usize,
    pub(in crate::domain_computation::primary_graph::conditional_operation) already_committed_operation_count:
        usize,
    pub(in crate::domain_computation::primary_graph::conditional_operation) failed_operation_count:
        usize,
    pub(in crate::domain_computation::primary_graph::conditional_operation) indeterminate_operation_count:
        usize,
    pub(in crate::domain_computation::primary_graph::conditional_operation) snapshot_capacity_backpressure:
        Option<usize>,
    pub(in crate::domain_computation::primary_graph::conditional_operation) retention_capacity_backpressure:
        bool,
    pub(in crate::domain_computation::primary_graph::conditional_operation) execution_provenance:
        Vec<super::super::WorthQueryConditionalExecutionProvenance>,
    pub(in crate::domain_computation::primary_graph::conditional_operation) granular_invalidations:
        Vec<worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery>,
    pub(in crate::domain_computation::primary_graph::conditional_operation) granular_invalidation_coverage:
        crate::domain_computation::primary_graph::WorthQueryGranularInvalidationCoverage,
}

pub(in crate::domain_computation::primary_graph) enum ErasedClockObservationOutcome {
    Accepted(ErasedClockObservationReceipt),
    Duplicate(ErasedClockObservationReceipt),
    Stale,
    Reordered,
    Closed,
    Failed {
        kind: super::WorthQueryConditionalClockObservationFailureKind,
        detail: String,
    },
}

impl ErasedClockObservationOutcome {
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn typed<Clock>(
        self,
        installation: crate::domain_computation::primary_graph::WorthQueryGranularInvalidationInstallation,
        source_read_basis: Option<
            crate::domain_computation::primary_graph::WorthQueryGranularSourceReadBasis,
        >,
    ) -> super::WorthQueryConditionalClockObservationOutcome<Clock> {
        match self {
            Self::Accepted(receipt) => {
                super::WorthQueryConditionalClockObservationOutcome::Accepted(
                    receipt.typed(installation, source_read_basis),
                )
            }
            Self::Duplicate(receipt) => {
                super::WorthQueryConditionalClockObservationOutcome::Duplicate(
                    receipt.typed(installation, source_read_basis),
                )
            }
            Self::Stale => super::WorthQueryConditionalClockObservationOutcome::Stale,
            Self::Reordered => super::WorthQueryConditionalClockObservationOutcome::Reordered,
            Self::Closed => super::WorthQueryConditionalClockObservationOutcome::Closed,
            Self::Failed { kind, detail } => {
                super::WorthQueryConditionalClockObservationOutcome::Failed(
                    super::WorthQueryConditionalClockObservationFailure { kind, detail },
                )
            }
        }
    }
}

impl ErasedClockObservationReceipt {
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn typed<Clock>(
        self,
        granular_invalidation_installation: crate::domain_computation::primary_graph::WorthQueryGranularInvalidationInstallation,
        granular_source_read_basis: Option<
            crate::domain_computation::primary_graph::WorthQueryGranularSourceReadBasis,
        >,
    ) -> super::WorthQueryConditionalClockObservationReceipt<Clock> {
        super::WorthQueryConditionalClockObservationReceipt {
            granular_invalidation_installation,
            granular_source_read_basis,
            sequence: self.sequence,
            observed_time: WorthQueryClockCoordinate::from_nanoseconds(self.observed_coordinate),
            due_wake_count: self.due_wake_count,
            due_work_remaining: self.due_work_remaining,
            authoritative_commit_count: self.authoritative_commit_count,
            authoritative_work_remaining: self.authoritative_work_remaining,
            retained_due_wake_count: self.retained_due_wake_count,
            retained_eligible_wake_count: self.retained_eligible_wake_count,
            retained_suppressed_wake_count: self.retained_suppressed_wake_count,
            retained_deferred_wake_count: self.retained_deferred_wake_count,
            retained_failed_wake_count: self.retained_failed_wake_count,
            committed_operation_count: self.committed_operation_count,
            already_committed_operation_count: self.already_committed_operation_count,
            failed_operation_count: self.failed_operation_count,
            indeterminate_operation_count: self.indeterminate_operation_count,
            snapshot_capacity_backpressure: self.snapshot_capacity_backpressure,
            retention_capacity_backpressure: self.retention_capacity_backpressure,
            execution_provenance: self.execution_provenance,
            granular_invalidations: self.granular_invalidations,
            granular_invalidation_coverage: self.granular_invalidation_coverage,
        }
    }
}
