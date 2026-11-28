use std::sync::Mutex;

use super::OperatorReport;

/// Receives reports after the emitting boundary selects their disclosure level.
pub trait DiagnosticSink: Send + Sync {
    fn emit(&self, report: &OperatorReport);
}

/// Discards emitted reports.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoopDiagnosticSink;

impl DiagnosticSink for NoopDiagnosticSink {
    fn emit(&self, _report: &OperatorReport) {}
}

/// In-memory capture for tests and finite diagnostic sessions.
#[derive(Default)]
pub struct RecordingDiagnosticSink {
    reports: Mutex<Vec<OperatorReport>>,
}

impl RecordingDiagnosticSink {
    /// Drains the capture in emission order. Concurrent emitters may interleave.
    pub fn take_reports(&self) -> Vec<OperatorReport> {
        core::mem::take(&mut *self.reports.lock().expect("diagnostic recorder poisoned"))
    }
}

impl DiagnosticSink for RecordingDiagnosticSink {
    fn emit(&self, report: &OperatorReport) {
        let report = report.clone();
        self.reports
            .lock()
            .expect("diagnostic recorder poisoned")
            .push(report);
    }
}
