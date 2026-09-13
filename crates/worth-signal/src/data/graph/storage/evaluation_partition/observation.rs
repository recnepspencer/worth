use std::sync::Arc;

use crate::data::graph::signal_graph::{
    InvalidationPerformedCounterState, ObservationCaptureCleanup, PerformedWorkCaptureState,
    SignalGraph,
};
use crate::logic::transaction::SignalObservationSessionState;

/// Capture gates and their cleanup targets travel together. A retained session
/// from another scope must never clear the newly active scope's evidence.
pub(super) struct EvaluationObservationStorage {
    sessions: SignalObservationSessionState,
    counters: InvalidationPerformedCounterState,
    work: PerformedWorkCaptureState,
    cleanup: Option<Arc<ObservationCaptureCleanup>>,
}

impl EvaluationObservationStorage {
    pub(super) fn initial_heap_charge() -> Result<
        crate::data::retained_storage::RetainedStorageCharge,
        crate::data::retained_storage::RetainedStoragePreparationDenial,
    > {
        SignalObservationSessionState::initial_heap_charge()?
            .checked_add(InvalidationPerformedCounterState::initial_heap_charge()?)?
            .checked_add(PerformedWorkCaptureState::initial_heap_charge()?)?
            .checked_add(ObservationCaptureCleanup::initial_heap_charge()?)
    }

    pub(super) fn new(
        policy: crate::runtime_policy::InstalledSignalRuntimePolicy,
        custody: Option<Arc<crate::data::retained_storage::SignalConditionalRetentionReservation>>,
    ) -> Self {
        let sessions = SignalObservationSessionState::default();
        sessions.set_default_surface_mask(policy.observation_capture_plan().default_surface_mask());
        let counters =
            InvalidationPerformedCounterState::with_capture_gate(sessions.capture_gate());
        let work = PerformedWorkCaptureState::with_capture_gate(sessions.capture_gate())
            .with_storage_custody(custody.clone());
        let cleanup = Some(Arc::new(
            ObservationCaptureCleanup::new(
                counters.shared_values(),
                work.shared_bindings(),
                sessions.shared_completed_execution_boundaries(),
                sessions.shared_last_completion(),
            )
            .with_storage_custody(custody),
        ));
        Self {
            sessions,
            counters,
            work,
            cleanup,
        }
    }

    pub(super) fn exchange(&mut self, graph: &mut SignalGraph) {
        std::mem::swap(&mut self.sessions, &mut graph.observation_sessions);
        std::mem::swap(
            &mut self.counters,
            &mut graph.invalidation_performed_counters,
        );
        std::mem::swap(&mut self.work, &mut graph.invalidation_performed_work);
        std::mem::swap(&mut self.cleanup, &mut graph.observation_capture_cleanup);
    }
}
