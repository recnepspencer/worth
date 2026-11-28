use std::sync::Arc;

use worth_execution::report::{DiagnosticDetail, DiagnosticSink, Reportable};

use super::{BridgeDiagnosticsFacade, BridgeDiagnosticsTier};

#[derive(Clone)]
pub(super) struct BridgeOperatorReports {
    detail: DiagnosticDetail,
    sink: Arc<dyn DiagnosticSink>,
}

impl std::fmt::Debug for BridgeOperatorReports {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BridgeOperatorReports")
            .field("detail", &self.detail)
            .finish_non_exhaustive()
    }
}

impl BridgeOperatorReports {
    pub(super) fn new(tier: BridgeDiagnosticsTier, sink: Arc<dyn DiagnosticSink>) -> Self {
        let detail = match tier {
            BridgeDiagnosticsTier::Minimal => DiagnosticDetail::Redacted,
            BridgeDiagnosticsTier::Standard => DiagnosticDetail::Standard,
            BridgeDiagnosticsTier::Exhaustive => DiagnosticDetail::Forensic,
        };
        Self { detail, sink }
    }
}

impl BridgeDiagnosticsFacade {
    pub(crate) fn emit_operator_report(&self, denial: &dyn Reportable) {
        let report = denial.to_operator_report(self.operator_reports.detail);
        self.operator_reports.sink.emit(&report);
    }
}
