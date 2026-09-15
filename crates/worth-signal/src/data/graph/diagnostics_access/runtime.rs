use crate::data::aspect::Aspect;
use crate::data::graph::signal_graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::proof::{
    FrontierDiagnosticsSidecar, InvalidationPlanningEstimate, InvalidationTraceRecord,
};
use crate::diagnostics::policy::OrdinaryAccessLane;
use crate::diagnostics::profile::DiagnosticsTier;
use crate::diagnostics::summary::GraphSummary;
use crate::runtime_policy::SignalRuntimePolicy;
use crate::runtime_policy::{
    compile_signal_runtime_policy, SignalRuntimePolicyCompilationDenial, SignalRuntimePolicyRequest,
};

impl SignalGraph {
    /// Execute any bounded maintenance work required before entering a pure
    /// observation phase.
    pub fn prepare_for_observation(&mut self) {
        self.run_gc_epoch();
    }

    pub(crate) fn telemetry(&self) -> &crate::data::telemetry::RuntimeTelemetry {
        &self.observation.telemetry
    }

    pub fn telemetry_mut(
        &mut self,
    ) -> Option<crate::data::telemetry::RuntimeTelemetryMutation<'_>> {
        if !self.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        ) {
            return None;
        }
        Some(crate::data::telemetry::RuntimeTelemetryMutation::active(
            &mut self.observation.telemetry,
        ))
    }

    pub(crate) fn with_telemetry(
        &mut self,
        update: impl FnOnce(&mut crate::data::telemetry::RuntimeTelemetry),
    ) {
        if let Some(mut telemetry) = self.telemetry_mut() {
            update(&mut telemetry);
        }
    }

    pub fn reset_telemetry(&mut self) {
        self.observation.telemetry = crate::data::telemetry::RuntimeTelemetry::default();
    }

    pub(crate) fn diagnostics_profile(&self) -> DiagnosticsTier {
        self.observation.diagnostics.tier()
    }

    pub(crate) fn runtime_policy(&self) -> SignalRuntimePolicy {
        self.observation.installed_policy().requested_policy()
    }

    pub(crate) fn installed_runtime_policy(
        &self,
    ) -> crate::runtime_policy::InstalledSignalRuntimePolicy {
        self.observation.installed_policy()
    }

    pub(crate) fn resolved_performance_policy(
        &self,
    ) -> crate::data::performance::ResolvedPerformancePolicy {
        self.observation.installed_policy().performance()
    }

    /// Reset graph diagnostics to the stock policy for one diagnostics tier.
    ///
    /// This is a lower-level convenience reset. If the caller means to keep
    /// custom retention or replay overrides, they should apply a full
    /// `SignalRuntimePolicy` instead.
    pub(crate) fn reset_runtime_policy_to_tier(&mut self, profile: DiagnosticsTier) {
        self.set_runtime_policy(SignalRuntimePolicy::for_tier(profile));
    }

    /// Apply the full runtime policy bundle to the graph diagnostics state.
    pub(crate) fn set_runtime_policy(&mut self, policy: SignalRuntimePolicy) {
        self.try_set_runtime_policy(policy)
            .expect("runtime policy request must be admitted before installation");
    }

    pub(crate) fn try_set_runtime_policy(
        &mut self,
        policy: SignalRuntimePolicy,
    ) -> Result<(), SignalRuntimePolicyCompilationDenial> {
        if self.observation_session_active_generation() != 0 {
            return Err(SignalRuntimePolicyCompilationDenial::ObservationSessionActive);
        }
        let installed = compile_signal_runtime_policy(SignalRuntimePolicyRequest::new(policy))?;
        self.observation.diagnostics.set_request_mirror(policy);
        self.observation.diagnostics.set_installed_policy(installed);
        self.observation.install_policy(installed);
        self.configure_observation_capture();
        Ok(())
    }

    pub(crate) fn install_compiled_runtime_policy(
        &mut self,
        policy: SignalRuntimePolicy,
        installed: crate::runtime_policy::InstalledSignalRuntimePolicy,
    ) {
        self.observation.diagnostics.set_request_mirror(policy);
        self.observation.diagnostics.set_installed_policy(installed);
        self.observation.install_policy(installed);
        self.configure_observation_capture();
    }

    pub(crate) fn install_rollback_runtime_policy(
        &mut self,
        installed: crate::runtime_policy::InstalledSignalRuntimePolicy,
    ) {
        self.observation.diagnostics.set_installed_policy(installed);
        self.observation.install_policy(installed);
        self.configure_observation_capture();
    }

    fn configure_observation_capture(&self) {
        self.set_default_observation_surface_mask(
            self.installed_runtime_policy()
                .observation_capture_plan()
                .default_surface_mask(),
        );
    }

    #[cfg(any(test, doctest))]
    pub(crate) fn diagnostics_summary(&self, profile: DiagnosticsTier) -> GraphSummary {
        if self.diagnostics_state().has_pending_change_input() {
            if let Some(summary) = self.diagnostics_state().pending_graph_summary() {
                return summary.with_profile(profile);
            }
        } else if let Some(summary) = self.diagnostics_state().latest_graph_summary() {
            return summary.with_profile(profile);
        }
        GraphSummary::from_graph(
            self,
            profile,
            self.installed_runtime_policy()
                .retention_budget()
                .detail_limit,
            OrdinaryAccessLane,
        )
    }

    pub(crate) fn diagnostics_state(&self) -> &crate::diagnostics::state::DiagnosticsState {
        &self.observation.diagnostics
    }

    pub(crate) fn diagnostics_state_mut(
        &mut self,
    ) -> &mut crate::diagnostics::state::DiagnosticsState {
        &mut self.observation.diagnostics
    }

    pub(crate) fn note_change_input(
        &mut self,
        node: NodeId,
        aspect: Aspect,
        changed_regions: &[crate::data::output::ChangedRegion],
    ) {
        if !self.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        ) {
            return;
        }
        let causality_kind = self
            .causality_of(node)
            .ok()
            .flatten()
            .map(|causality| causality.kind.clone());
        let performed_baseline = self.invalidation_performed_counters();
        self.observation.diagnostics.note_change_input(
            node,
            aspect,
            changed_regions,
            causality_kind,
            performed_baseline,
        );
    }

    pub(crate) fn clear_pending_diagnostics_input(&mut self) {
        self.observation.diagnostics.clear_pending_input();
    }

    pub(crate) fn captures_failure_diagnostics(&self) -> bool {
        self.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::DescriptiveFacts,
        ) && self
            .installed_runtime_policy()
            .retention_budget()
            .retain_latest_failure_context
    }

    pub(crate) fn captures_rollback_diagnostics(&self) -> bool {
        self.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::DescriptiveFacts,
        ) && self
            .installed_runtime_policy()
            .retention_budget()
            .retain_history_details
    }

    pub(crate) fn record_frontier_execution_diagnostics(
        &mut self,
        planning_estimate: InvalidationPlanningEstimate,
        summary: FrontierDiagnosticsSidecar,
        trace_records: Vec<InvalidationTraceRecord>,
    ) {
        if !self.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::FrontierTrace,
        ) {
            return;
        }
        if self.diagnostics_state().pending_graph_summary().is_none() {
            if let Some(summary) = self.diagnostics_state().latest_graph_summary().cloned() {
                self.observation
                    .diagnostics
                    .set_pending_graph_summary(summary.with_profile(self.diagnostics_profile()));
            } else {
                let retention_budget = self.installed_runtime_policy().retention_budget();
                self.observation
                    .diagnostics
                    .set_pending_graph_summary(GraphSummary::from_graph(
                        self,
                        self.diagnostics_profile(),
                        retention_budget.detail_limit,
                        OrdinaryAccessLane,
                    ));
            }
        }
        self.observation.diagnostics.record_frontier_execution(
            planning_estimate,
            summary,
            trace_records,
        );
    }
}
