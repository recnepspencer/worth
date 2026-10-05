use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebExecutionReportSummary {
    pub resolved_posture: String,
    pub charged_work: u64,
    pub charged_span: u64,
    pub active_workers_high_watermark: u64,
    pub peak_charged_memory_bytes: u64,
    pub peak_queue_width: u64,
    pub discarded_in_flight_work: u64,
    pub fallback: Option<String>,
}

impl From<worth_foundational::ExecutionReport> for WebExecutionReportSummary {
    fn from(report: worth_foundational::ExecutionReport) -> Self {
        let physical = report.physical();
        Self {
            resolved_posture: format!("{:?}", report.resolved_posture()),
            charged_work: report.charged_work(),
            charged_span: report.charged_span(),
            active_workers_high_watermark: physical.active_workers_high_watermark() as u64,
            peak_charged_memory_bytes: physical.peak_charged_memory_bytes(),
            peak_queue_width: physical.peak_queue_width() as u64,
            discarded_in_flight_work: physical.discarded_in_flight_work(),
            fallback: report.fallback().map(|cause| format!("{cause:?}")),
        }
    }
}
