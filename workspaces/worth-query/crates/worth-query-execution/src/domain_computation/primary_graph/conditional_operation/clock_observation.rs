use std::marker::PhantomData;

use worth_query_installation::facade::{ApplicationSchema, WorthQueryClockCoordinate};

use super::installation::WorthQueryConditionalClockHandle;
use super::lifecycle::WorthQueryConditionalOperationCell;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

mod erased;
mod product_admission;
pub(in crate::domain_computation::primary_graph) use erased::{
    ErasedClockObservationOutcome, ErasedClockObservationReceipt,
};

/// Why a clock observation port could not be opened.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConditionalClockObservationDenialKind {
    /// The clock handle is not installed in this runtime.
    ForeignRuntime,
    /// The clock handle's binding is not installed.
    BindingNotInstalled,
    /// The conditional binding could not be admitted on the selected product.
    ProductAdmission(super::installation::WorthQueryConditionalRuntimeInstallationDenialKind),
}

/// Refusal to open a clock observation port for a conditional operation.
///
/// No clock reading was taken. [`Self::kind`] says why and [`Self::subject`]
/// names the binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryConditionalClockObservationDenial {
    kind: WorthQueryConditionalClockObservationDenialKind,
    subject: String,
}

impl WorthQueryConditionalClockObservationDenial {
    fn new(
        kind: WorthQueryConditionalClockObservationDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub fn kind(&self) -> WorthQueryConditionalClockObservationDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

/// Why a clock observation failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConditionalClockObservationFailureKind {
    /// The clock source was unavailable.
    SourceUnavailable,
    /// The clock source failed to produce a reading.
    ObservationFailed,
    /// The clock source panicked.
    SourcePanicked,
    /// The runtime rejected the reading or could not process it.
    RuntimeRejected,
    /// The installed limit on concurrently active snapshots was reached.
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    /// No capacity remains to retain a basis.
    RetentionCapacityExhausted,
    /// The runtime ran out of basis-retention identities.
    RetentionIdentityExhausted,
    /// The runtime ran out of snapshot identities.
    SnapshotIdentityExhausted,
}

/// A clock observation that failed, with its kind and a detail message.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryConditionalClockObservationFailure {
    kind: WorthQueryConditionalClockObservationFailureKind,
    detail: String,
}

impl WorthQueryConditionalClockObservationFailure {
    pub fn kind(&self) -> WorthQueryConditionalClockObservationFailureKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// Record of one accepted clock reading and what it caused.
///
/// Reports the observed time, the wakes that came due, how many were retained
/// and in which state, how many operations committed, failed, or ended
/// indeterminate, any capacity backpressure, and per-wake provenance. It is
/// descriptive and grants nothing.
pub struct WorthQueryConditionalClockObservationReceipt<Clock> {
    granular_invalidation_installation:
        crate::domain_computation::primary_graph::WorthQueryGranularInvalidationInstallation,
    granular_source_read_basis:
        Option<crate::domain_computation::primary_graph::WorthQueryGranularSourceReadBasis>,
    sequence: u64,
    observed_time: WorthQueryClockCoordinate<Clock>,
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
    execution_provenance: Vec<super::WorthQueryConditionalExecutionProvenance>,
    granular_invalidations: Vec<worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery>,
}

impl<Clock> WorthQueryConditionalClockObservationReceipt<Clock> {
    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn observed_time(&self) -> WorthQueryClockCoordinate<Clock> {
        WorthQueryClockCoordinate::from_nanoseconds(self.observed_time.nanoseconds())
    }

    pub fn due_wake_count(&self) -> usize {
        self.due_wake_count
    }

    pub fn due_work_remaining(&self) -> bool {
        self.due_work_remaining
    }

    /// Relevant authoritative commits examined before this clock advance.
    pub fn authoritative_commit_count(&self) -> usize {
        self.authoritative_commit_count
    }

    pub fn authoritative_work_remaining(&self) -> bool {
        self.authoritative_work_remaining
    }

    /// Wakes retained inside Query until governed operation re-entry consumes them.
    pub fn retained_due_wake_count(&self) -> usize {
        self.retained_due_wake_count
    }

    pub fn retained_eligible_wake_count(&self) -> usize {
        self.retained_eligible_wake_count
    }

    pub fn retained_suppressed_wake_count(&self) -> usize {
        self.retained_suppressed_wake_count
    }

    pub fn retained_deferred_wake_count(&self) -> usize {
        self.retained_deferred_wake_count
    }

    pub fn retained_failed_wake_count(&self) -> usize {
        self.retained_failed_wake_count
    }

    pub fn committed_operation_count(&self) -> usize {
        self.committed_operation_count
    }

    pub fn already_committed_operation_count(&self) -> usize {
        self.already_committed_operation_count
    }

    pub fn failed_operation_count(&self) -> usize {
        self.failed_operation_count
    }

    pub fn indeterminate_operation_count(&self) -> usize {
        self.indeterminate_operation_count
    }

    /// The owner-local snapshot ceiling that deferred operation re-entry.
    pub fn snapshot_capacity_backpressure(&self) -> Option<usize> {
        self.snapshot_capacity_backpressure
    }

    /// Whether owner-local retention pressure deferred operation re-entry.
    pub fn retention_capacity_backpressure(&self) -> bool {
        self.retention_capacity_backpressure
    }

    pub fn execution_provenance(&self) -> &[super::WorthQueryConditionalExecutionProvenance] {
        &self.execution_provenance
    }

    /// Consume the exact lower-runtime deliveries observed while this clock
    /// observation reconsidered authoritative commits. The returned carrier
    /// is transport evidence only; Query still performs candidate selection
    /// and current admission.
    pub fn take_granular_invalidation_batch(
        &mut self,
    ) -> crate::domain_computation::primary_graph::WorthQueryGranularInvalidationDeliveryBatch {
        crate::domain_computation::primary_graph::granular_invalidation::collect_granular_invalidations(
            self.granular_invalidation_installation.clone(),
            std::mem::take(&mut self.granular_invalidations),
            self.granular_source_read_basis.clone(),
        )
    }
}

/// The result of one clock observation.
pub enum WorthQueryConditionalClockObservationOutcome<Clock> {
    /// The reading was accepted and processed.
    Accepted(WorthQueryConditionalClockObservationReceipt<Clock>),
    /// The reading repeats one already accepted; the receipt describes it.
    Duplicate(WorthQueryConditionalClockObservationReceipt<Clock>),
    /// The reading is older than one already accepted; nothing was processed.
    Stale,
    /// The reading arrived out of order; nothing was processed.
    Reordered,
    /// The clock binding or source is closed.
    Closed,
    /// The observation failed.
    Failed(WorthQueryConditionalClockObservationFailure),
}

/// A port for feeding clock readings to one installed conditional operation.
///
/// Open it with `conditional_clock` on a selected product. Each `observe`
/// reads the clock, processes any wakes that came due, and re-enters their
/// operations through fresh admission.
pub struct WorthQueryConditionalClockObservationPort<'runtime, Schema, Node, Clock> {
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    operation: WorthQueryConditionalOperationCell<Schema>,
    truth: super::signal_decision_reentry::WorthQueryConditionalTruthBasis,
    marker: PhantomData<fn() -> (Node, Clock)>,
}

impl<'runtime, Schema, Node, Clock>
    WorthQueryConditionalClockObservationPort<'runtime, Schema, Node, Clock>
where
    Schema: ApplicationSchema,
{
    pub fn observe(&mut self) -> WorthQueryConditionalClockObservationOutcome<Clock> {
        let granular_invalidation_installation = self.runtime.granular_invalidation_installation();
        let granular_source_read_basis = self.truth.granular_source_read_basis();
        let bridge_root = self.runtime.bridge.conditional_operations();
        let bridge = bridge_root
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let outcome = self
            .operation
            .observe_clock(&bridge, self.runtime, &self.truth);
        if outcome.routes_changed {
            let registry = self
                .runtime
                .conditional_operations
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .snapshot();
            registry.synchronize_commit_routes(self.runtime);
        }
        outcome.outcome.typed(
            granular_invalidation_installation,
            Some(granular_source_read_basis),
        )
    }
}

#[cfg(test)]
mod tests;
