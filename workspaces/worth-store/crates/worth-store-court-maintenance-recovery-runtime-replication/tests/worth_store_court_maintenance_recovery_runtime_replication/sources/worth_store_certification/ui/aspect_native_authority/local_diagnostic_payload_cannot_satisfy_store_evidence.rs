use worth_store_aspect_native::StoreDiagnosticSupportReportEvidence;

struct LocalDiagnosticPayload;

fn require_store_diagnostic_evidence(_evidence: StoreDiagnosticSupportReportEvidence) {}

fn main() {
    require_store_diagnostic_evidence(LocalDiagnosticPayload);
}
