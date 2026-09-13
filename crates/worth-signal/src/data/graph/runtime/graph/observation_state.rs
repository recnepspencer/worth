use serde::{Deserialize, Serialize};

use super::PerformedWorkBuffer;
use crate::data::handle::NodeId;
use crate::data::output::PartitionInterner;
use crate::data::telemetry::RuntimeTelemetry;
use crate::diagnostics::state::DiagnosticsState;
use crate::logic::transaction::{
    SignalObservationCompletion, SignalObservationDropCleanup, SignalObservationRequest,
    SignalObservationSurface,
};
use crate::runtime_policy::InstalledSignalRuntimePolicy;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use super::{
    BranchMutationRecord, InvalidationPerformedCounterState, PerformedWorkCaptureState,
    ReconstructionCounters, SignalGraph,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct RuntimeObservation {
    #[serde(skip, default)]
    pub(in crate::data::graph) telemetry: RuntimeTelemetry,
    #[serde(skip, default)]
    pub(in crate::data::graph) reconstruction_counters: ReconstructionCounters,
    #[serde(default)]
    pub(in crate::data::graph) partition_interner: PartitionInterner,
    #[serde(default)]
    pub(in crate::data::graph) branch_mutation_view:
        crate::data::persistent_ord_map::PersistentOrdMap<NodeId, BranchMutationRecord>,
    #[serde(default)]
    pub(in crate::data::graph) branch_mutation_records:
        crate::data::persistent_ord_map::PersistentOrdMap<NodeId, BranchMutationRecord>,
    #[serde(skip, default)]
    pub(in crate::data::graph) diagnostics: DiagnosticsState,
    #[serde(default)]
    pub(in crate::data::graph) installed_policy: InstalledSignalRuntimePolicy,
}

#[derive(Debug)]
pub(crate) struct ObservationCaptureCleanup {
    counters: Arc<[AtomicU64; 24]>,
    bindings: Arc<Mutex<PerformedWorkBuffer>>,
    completed_execution_boundaries: Arc<AtomicU64>,
    last_completion: Arc<AtomicU64>,
    storage_custody:
        Option<Arc<crate::data::retained_storage::SignalConditionalRetentionReservation>>,
}

impl ObservationCaptureCleanup {
    pub(crate) fn initial_heap_charge() -> Result<
        crate::data::retained_storage::RetainedStorageCharge,
        crate::data::retained_storage::RetainedStoragePreparationDenial,
    > {
        crate::data::retained_storage::arc_allocation_charge::<Self>()
    }

    pub(crate) fn with_storage_custody(
        mut self,
        custody: Option<Arc<crate::data::retained_storage::SignalConditionalRetentionReservation>>,
    ) -> Self {
        self.storage_custody = custody;
        self
    }

    pub(crate) fn new(
        counters: Arc<[AtomicU64; 24]>,
        bindings: Arc<Mutex<PerformedWorkBuffer>>,
        completed_execution_boundaries: Arc<AtomicU64>,
        last_completion: Arc<AtomicU64>,
    ) -> Self {
        Self {
            counters,
            bindings,
            completed_execution_boundaries,
            last_completion,
            storage_custody: None,
        }
    }
}

impl SignalObservationDropCleanup for ObservationCaptureCleanup {
    fn clear(&self, request: SignalObservationRequest) {
        if request.includes(SignalObservationSurface::PerformedCounters) {
            for value in self.counters.iter() {
                value.store(0, Ordering::Release);
            }
        }
        if request.includes(SignalObservationSurface::PerformedWork) {
            self.bindings
                .lock()
                .expect("performed work observation poisoned")
                .clear();
        }
        self.completed_execution_boundaries
            .store(0, Ordering::Release);
        self.last_completion.store(
            u64::from(SignalObservationCompletion::Abandoned.code()),
            Ordering::Release,
        );
    }
}

impl RuntimeObservation {
    pub(crate) fn operational_clone(&self) -> Self {
        Self {
            telemetry: self.telemetry,
            reconstruction_counters: self.reconstruction_counters.clone(),
            partition_interner: self.partition_interner.operational_clone(),
            branch_mutation_view: self.branch_mutation_view.operational_clone(),
            branch_mutation_records: self.branch_mutation_records.operational_clone(),
            diagnostics: self.diagnostics.clone(),
            installed_policy: self.installed_policy,
        }
    }

    pub(crate) fn fork_branch_local(&mut self) -> Self {
        Self {
            telemetry: self.telemetry,
            reconstruction_counters: self.reconstruction_counters.clone(),
            partition_interner: self.partition_interner.fork_persistent(),
            branch_mutation_view: Default::default(),
            branch_mutation_records: Default::default(),
            diagnostics: self.diagnostics.fork_branch_carrier(),
            installed_policy: self.installed_policy,
        }
    }

    pub(crate) fn partition_interner_mut(&mut self) -> &mut PartitionInterner {
        &mut self.partition_interner
    }

    pub(crate) fn partition_interner(&self) -> &PartitionInterner {
        &self.partition_interner
    }

    #[cfg(test)]
    pub(crate) fn fork_storage_identity(&self) -> PartitionInterner {
        self.partition_interner.fork_storage_identity()
    }

    pub(crate) fn installed_policy(&self) -> InstalledSignalRuntimePolicy {
        self.installed_policy
    }

    pub(crate) fn install_policy(&mut self, policy: InstalledSignalRuntimePolicy) {
        self.installed_policy = policy;
    }
}

impl SignalGraph {
    /// Rebuild the runtime-only observation bundle after graph construction or
    /// deserialization. All optional stores must share this exact session gate
    /// so continuous defaults and explicit sessions select the same surfaces.
    pub(crate) fn rebind_observation_capture_state(&mut self) {
        self.observation_sessions.set_default_surface_mask(
            self.installed_runtime_policy()
                .observation_capture_plan()
                .default_surface_mask(),
        );
        let capture_gate = self.observation_sessions.capture_gate();
        self.invalidation_performed_counters =
            InvalidationPerformedCounterState::with_capture_gate(capture_gate.clone());
        self.invalidation_performed_work =
            PerformedWorkCaptureState::with_capture_gate(capture_gate);
        self.observation_capture_cleanup = Some(Arc::new(ObservationCaptureCleanup::new(
            self.invalidation_performed_counters.shared_values(),
            self.invalidation_performed_work.shared_bindings(),
            self.observation_sessions
                .shared_completed_execution_boundaries(),
            self.observation_sessions.shared_last_completion(),
        )));
    }
}
