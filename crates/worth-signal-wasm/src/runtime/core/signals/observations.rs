use js_sys::Function;
use worth_signal::facade::runtime::{
    ObservationHandle, ObservationListener, ObservationNotice, ObservationPolicy,
    ObservationReadContext,
};

use crate::boundary::errors::WorthSignalJsError;
use crate::expression::model::SignalValue;
use crate::runtime::compute_callbacks;
use crate::runtime::diagnostics_callbacks::DiagnosticsCallbackToken;
use crate::runtime::summaries::WebPerformanceSummary;
use crate::runtime::web_callbacks::{self, ObservationCallbackToken};

use super::super::state::{SharedStore, WebSignalKind};
use super::super::{RuntimeCore, WasmEffectListener, WasmWatchListener};

impl RuntimeCore {
    pub(crate) fn observe_signal_for_runtime_certification(
        &mut self,
        id: &str,
    ) -> Result<ObservationHandle, WorthSignalJsError> {
        let node = self.node_for_id(id)?;
        Ok(self.runtime.observe_nodes(
            ObservationPolicy::meaningful_change(),
            [node],
            Box::new(RuntimeCertificationObservationListener),
        ))
    }

    pub fn watch_signal(
        &mut self,
        id: &str,
        callback_token: ObservationCallbackToken,
    ) -> Result<ObservationHandle, WorthSignalJsError> {
        let node = self.node_for_id(id)?;
        Ok(self.runtime.observe_nodes(
            ObservationPolicy::meaningful_change(),
            [node],
            Box::new(WasmWatchListener {
                callback_scope_id: self.observation_callback_scope_id,
                callback_token,
                signal_id: id.to_owned(),
            }),
        ))
    }

    pub fn effect_signal(
        &mut self,
        id: &str,
        callback_token: ObservationCallbackToken,
    ) -> Result<ObservationHandle, WorthSignalJsError> {
        let node = self.node_for_id(id)?;
        Ok(self.runtime.observe_nodes(
            ObservationPolicy::meaningful_change(),
            [node],
            Box::new(WasmEffectListener {
                callback_scope_id: self.observation_callback_scope_id,
                callback_token,
            }),
        ))
    }

    pub fn register_wasm_watch_callback(&mut self, callback: Function) -> ObservationCallbackToken {
        web_callbacks::register_wasm_watch(self.observation_callback_scope_id, callback)
    }

    pub fn register_wasm_effect_callback(
        &mut self,
        callback: Function,
    ) -> ObservationCallbackToken {
        web_callbacks::register_wasm_effect(self.observation_callback_scope_id, callback)
    }

    pub fn dispose_observation_callback(&mut self, token: ObservationCallbackToken) -> bool {
        web_callbacks::dispose_callback(self.observation_callback_scope_id, token)
    }

    pub fn register_wasm_diagnostics_callback(
        &mut self,
        callback: Function,
    ) -> DiagnosticsCallbackToken {
        crate::runtime::diagnostics_callbacks::register_wasm_diagnostics_callback(
            self.diagnostics_callback_scope_id,
            callback,
        )
    }

    pub fn dispose_diagnostics_callback(&mut self, token: DiagnosticsCallbackToken) -> bool {
        crate::runtime::diagnostics_callbacks::dispose_diagnostics_callback(
            self.diagnostics_callback_scope_id,
            token,
        )
    }

    pub fn notify_diagnostics_subscribers(&mut self) {
        crate::runtime::diagnostics_callbacks::notify_diagnostics_callbacks(
            self.diagnostics_callback_scope_id,
        );
    }

    #[cfg(test)]
    pub fn register_native_watch_callback(
        &mut self,
        callback: Box<dyn Fn(web_callbacks::WebObservationNotice)>,
    ) -> ObservationCallbackToken {
        web_callbacks::register_native_watch(self.observation_callback_scope_id, callback)
    }

    #[cfg(test)]
    pub fn register_native_effect_callback(
        &mut self,
        callback: Box<dyn Fn()>,
    ) -> ObservationCallbackToken {
        web_callbacks::register_native_effect(self.observation_callback_scope_id, callback)
    }

    #[cfg(test)]
    pub fn register_native_diagnostics_callback(
        &mut self,
        callback: Box<dyn Fn()>,
    ) -> DiagnosticsCallbackToken {
        crate::runtime::diagnostics_callbacks::register_native_diagnostics_callback(
            self.diagnostics_callback_scope_id,
            callback,
        )
    }

    pub fn unobserve_handle(&mut self, handle: ObservationHandle) -> bool {
        self.runtime.unobserve(handle)
    }

    pub fn note_app_signal_serialization(&mut self, id: &str, value: &SignalValue) {
        if matches!(self.web_signals.get(id), Some(WebSignalKind::Output)) {
            self.record_output_serialization(value);
        }
    }

    pub fn note_compatibility_read(&mut self, breadth: usize) {
        self.web_metrics.compatibility_read_count =
            self.web_metrics.compatibility_read_count.saturating_add(1);
        self.web_metrics.compatibility_read_breadth = self
            .web_metrics
            .compatibility_read_breadth
            .saturating_add(breadth as u64);
    }

    pub fn note_compatibility_signal_serialization(&mut self, id: &str, value: &SignalValue) {
        if matches!(self.web_signals.get(id), Some(WebSignalKind::Output)) {
            self.record_output_serialization(value);
        }
    }

    pub fn web_performance_summary(&self) -> WebPerformanceSummary {
        let observation_callback_stats =
            web_callbacks::callback_stats(self.observation_callback_scope_id);
        let compute_callback_stats = compute_callbacks::compute_callback_stats();
        let transaction = self.runtime.telemetry().transaction;
        WebPerformanceSummary {
            active_handle_count: observation_callback_stats.active_observation_callback_count,
            active_callback_count: observation_callback_stats
                .active_observation_callback_count
                .saturating_add(compute_callback_stats.active_compute_callback_count),
            active_compute_callback_count: compute_callback_stats.active_compute_callback_count,
            active_compute_collector_count: compute_callback_stats.active_compute_collector_count,
            matched_watcher_breadth: transaction.staged_observation_match_count,
            delivered_observation_count: transaction.delivered_observation_count,
            rollback_suppressed_delivery_count: transaction.rollback_suppressed_observation_count,
            serial_executor_usage_count: self
                .runtime
                .telemetry()
                .execution
                .serial_executor_usage_count,
            execution_report: self
                .runtime
                .telemetry()
                .execution
                .last_execution_report
                .map(Into::into),
            output_serialization_count: self.web_metrics.output_serialization_count,
            output_serialization_breadth: self.web_metrics.output_serialization_breadth,
            js_callback_invocation_count: observation_callback_stats
                .observation_callback_invocation_count,
            js_callback_failure_count: observation_callback_stats
                .observation_callback_failure_count,
            observation_callback_registration_count: observation_callback_stats
                .observation_callback_registration_count,
            observation_callback_disposal_count: observation_callback_stats
                .observation_callback_disposal_count,
            observation_callback_generation_mismatch_denial_count: observation_callback_stats
                .observation_callback_generation_mismatch_denial_count,
            observation_callback_allocation_count: observation_callback_stats
                .observation_callback_allocation_count,
            observation_callback_reuse_count: observation_callback_stats
                .observation_callback_reuse_count,
            compute_callback_registration_count: compute_callback_stats
                .compute_callback_registration_count,
            compute_callback_disposal_count: compute_callback_stats.compute_callback_disposal_count,
            compute_callback_invocation_count: compute_callback_stats
                .compute_callback_invocation_count,
            compute_callback_failure_count: compute_callback_stats.compute_callback_failure_count,
            compute_callback_generation_mismatch_denial_count: compute_callback_stats
                .compute_callback_generation_mismatch_denial_count,
            compute_callback_self_read_denial_count: compute_callback_stats
                .compute_callback_self_read_denial_count,
            compute_callback_dynamic_cycle_denial_count: compute_callback_stats
                .compute_callback_dynamic_cycle_denial_count,
            compute_callback_promise_return_denial_count: compute_callback_stats
                .compute_callback_promise_return_denial_count,
            compute_callback_invalid_return_denial_count: compute_callback_stats
                .compute_callback_invalid_return_denial_count,
            compute_callback_collector_installation_count: compute_callback_stats
                .compute_callback_collector_installation_count,
            compute_callback_capture_count: compute_callback_stats.compute_callback_capture_count,
            compute_callback_captured_read_count: compute_callback_stats
                .compute_callback_captured_read_count,
            compute_callback_return_serialization_breadth: compute_callback_stats
                .compute_callback_return_serialization_breadth,
            compute_callback_allocation_count: compute_callback_stats
                .compute_callback_allocation_count,
            compute_callback_reuse_count: compute_callback_stats.compute_callback_reuse_count,
            compute_callback_dependency_patch_count: self
                .web_metrics
                .compute_callback_dependency_patch_count,
            compute_callback_dependency_patch_added_count: self
                .web_metrics
                .compute_callback_dependency_patch_added_count,
            compute_callback_dependency_patch_removed_count: self
                .web_metrics
                .compute_callback_dependency_patch_removed_count,
            compute_callback_dependency_patch_retained_count: self
                .web_metrics
                .compute_callback_dependency_patch_retained_count,
            compute_callback_runtime_read_breadth: self
                .web_metrics
                .compute_callback_runtime_read_breadth,
            compute_callback_constant_no_signal_read_classification_count: self
                .web_metrics
                .compute_callback_constant_no_signal_read_classification_count,
            compute_callback_signal_tracked_classification_count: self
                .web_metrics
                .compute_callback_signal_tracked_classification_count,
            compute_callback_missing_unavailability_count: self
                .web_metrics
                .compute_callback_missing_unavailability_count,
            compatibility_read_count: self.web_metrics.compatibility_read_count,
            compatibility_read_breadth: self.web_metrics.compatibility_read_breadth,
        }
    }
}

struct RuntimeCertificationObservationListener;

impl ObservationListener<(), (), (), SharedStore, ()> for RuntimeCertificationObservationListener {
    fn on_observation(
        &self,
        _ctx: ObservationReadContext<'_, (), (), (), SharedStore, ()>,
        _notice: &ObservationNotice<'_>,
    ) {
    }
}
