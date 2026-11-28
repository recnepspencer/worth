use super::*;

impl BridgeDiagnosticsFacade {
    pub(crate) fn new(
        policy: BridgeRuntimePolicy,
        diagnostic_sink: Arc<dyn worth_execution::report::DiagnosticSink>,
    ) -> Self {
        let retention_budget = policy.retention_budget();
        Self {
            operator_reports: super::operator_reports::BridgeOperatorReports::new(
                policy.diagnostics_tier(),
                diagnostic_sink,
            ),
            config: Arc::new(BridgeDiagnosticsConfig {
                tier: policy.diagnostics_tier(),
                records_enabled: policy.record_route_artifacts(),
                replay_enabled: policy.allow_replay_artifacts(),
                route_record_limit: retention_budget.route_record_limit(),
                failure_record_limit: retention_budget.failure_record_limit(),
            }),
            state: Arc::new(RwLock::new(BridgeDiagnosticsState::default())),
        }
    }

    pub fn tier(&self) -> BridgeDiagnosticsTier {
        self.config.tier
    }

    pub fn records_enabled(&self) -> bool {
        self.config.records_enabled
    }

    pub fn route_record_limit(&self) -> usize {
        self.config.route_record_limit
    }

    pub fn failure_record_limit(&self) -> usize {
        self.config.failure_record_limit
    }
}
