use super::{InvalidationPerformedCounterState, SignalGraph};

impl SignalGraph {
    pub(crate) fn set_default_observation_surface_mask(&self, surface_mask: u8) {
        self.observation_sessions
            .set_default_surface_mask(surface_mask);
    }

    #[cfg(test)]
    pub(crate) fn reset_invalidation_performed_counters(&mut self) {
        self.invalidation_performed_counters.reset();
        self.invalidation_performed_work.reset();
        self.pending_repeated_invalidation_admissions.clear();
    }

    pub(crate) fn invalidation_performed_counters(
        &self,
    ) -> crate::data::telemetry::SignalInvalidationRealizedCounters {
        self.invalidation_performed_counters.snapshot()
    }

    pub(crate) fn invalidation_performed_work(
        &self,
    ) -> Vec<crate::data::proof::invalidation::progression::InvalidationWorkBindingAxes> {
        self.invalidation_performed_work.snapshot()
    }

    pub(crate) const fn invalidation_performed_counter_state(
        &self,
    ) -> &InvalidationPerformedCounterState {
        &self.invalidation_performed_counters
    }

    pub(crate) fn observation_session_active_generation(&self) -> u64 {
        self.observation_sessions.active_generation()
    }

    pub(crate) fn observation_session_liveness(
        &self,
    ) -> std::sync::Arc<std::sync::atomic::AtomicU64> {
        self.observation_sessions.liveness()
    }

    pub(crate) fn finish_observation_generation(&self, generation: u64) -> bool {
        self.observation_sessions.finish(generation)
    }

    pub(crate) fn record_observation_execution_boundary(&self) {
        self.observation_sessions
            .record_completed_execution_boundary();
    }

    pub(crate) fn completed_observation_execution_boundaries(&self) -> u64 {
        self.observation_sessions.completed_execution_boundaries()
    }

    pub fn last_observation_completion(
        &self,
    ) -> Option<crate::logic::transaction::SignalObservationCompletion> {
        self.observation_sessions.last_completion()
    }

    pub(crate) fn captures_observation_surface(
        &self,
        surface: crate::logic::transaction::SignalObservationSurface,
    ) -> bool {
        self.observation_sessions.capture_gate().captures(surface)
    }

    pub(crate) fn begin_invalidation_readiness_epoch(
        &mut self,
    ) -> crate::data::proof::invalidation::progression::InvalidationReadinessEpoch {
        self.invalidation_readiness_epoch = self.invalidation_readiness_epoch.saturating_add(1);
        crate::data::proof::invalidation::progression::InvalidationReadinessEpoch(
            self.invalidation_readiness_epoch,
        )
    }

    pub(crate) const fn current_invalidation_readiness_epoch(
        &self,
    ) -> crate::data::proof::invalidation::progression::InvalidationReadinessEpoch {
        crate::data::proof::invalidation::progression::InvalidationReadinessEpoch(
            self.invalidation_readiness_epoch,
        )
    }

    pub(crate) fn record_hot_path_artifact_reconstruction(&self) {
        self.observation
            .reconstruction_counters
            .record_hot_path_artifact_reconstruction();
    }

    pub(crate) fn hot_path_artifact_reconstruction_count(&self) -> u64 {
        self.observation
            .reconstruction_counters
            .hot_path_artifact_reconstruction_count()
    }

    pub(crate) fn record_explicit_cold_materialization_request(&self) {
        self.observation
            .reconstruction_counters
            .record_explicit_cold_materialization_request();
    }

    pub(crate) fn explicit_cold_materialization_request_count(&self) -> u64 {
        self.observation
            .reconstruction_counters
            .explicit_cold_materialization_request_count()
    }

    pub(crate) fn record_retained_forensic_read(&self) {
        self.observation
            .reconstruction_counters
            .record_retained_forensic_read();
    }

    pub(crate) fn retained_forensic_read_count(&self) -> u64 {
        self.observation
            .reconstruction_counters
            .retained_forensic_read_count()
    }

    pub(crate) fn record_cold_explanation_reconstruction(&self) {
        self.observation
            .reconstruction_counters
            .record_cold_explanation_reconstruction();
    }

    pub(crate) fn cold_explanation_reconstruction_count(&self) -> u64 {
        self.observation
            .reconstruction_counters
            .cold_explanation_reconstruction_count()
    }

    pub(crate) fn record_cold_provenance_reconstruction(&self) {
        self.observation
            .reconstruction_counters
            .record_cold_provenance_reconstruction();
    }

    pub(crate) fn cold_provenance_reconstruction_count(&self) -> u64 {
        self.observation
            .reconstruction_counters
            .cold_provenance_reconstruction_count()
    }

    pub(crate) fn record_retained_artifact_read(&self) {
        self.observation
            .reconstruction_counters
            .record_retained_artifact_read();
    }

    pub(crate) fn retained_artifact_read_count(&self) -> u64 {
        self.observation
            .reconstruction_counters
            .retained_artifact_read_count()
    }

    pub(crate) fn reconstructed_artifact_read_count(&self) -> u64 {
        self.observation
            .reconstruction_counters
            .reconstructed_artifact_read_count()
    }

    pub(crate) fn checkpoint_reconstruction_count(&self) -> u64 {
        self.observation
            .reconstruction_counters
            .checkpoint_reconstruction_count()
    }

    pub(crate) fn record_denied_reconstruction_by_budget(&self, explanation_api: bool) {
        self.observation
            .reconstruction_counters
            .record_denied_reconstruction_by_budget(explanation_api);
    }

    pub(crate) fn denied_reconstruction_by_budget_count(&self) -> u64 {
        self.observation
            .reconstruction_counters
            .denied_reconstruction_by_budget_count()
    }

    pub(crate) fn record_denied_reconstruction_by_tier(&self, explanation_api: bool) {
        self.observation
            .reconstruction_counters
            .record_denied_reconstruction_by_tier(explanation_api);
    }

    pub(crate) fn denied_reconstruction_by_tier_count(&self) -> u64 {
        self.observation
            .reconstruction_counters
            .denied_reconstruction_by_tier_count()
    }

    pub(crate) fn denied_reconstruction_explanation_api_count(&self) -> u64 {
        self.observation
            .reconstruction_counters
            .denied_reconstruction_explanation_api_count()
    }

    pub(crate) fn denied_reconstruction_provenance_api_count(&self) -> u64 {
        self.observation
            .reconstruction_counters
            .denied_reconstruction_provenance_api_count()
    }
}
