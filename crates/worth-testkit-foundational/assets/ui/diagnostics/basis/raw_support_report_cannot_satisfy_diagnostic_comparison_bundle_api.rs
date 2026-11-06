use worth_foundational::{FoundationalDiagnosticComparisonBundle, FoundationalDiagnosticSupportReport};

fn require_comparison_bundle(_bundle: &FoundationalDiagnosticComparisonBundle) {}

fn rejects_raw_value(report: FoundationalDiagnosticSupportReport) {
    require_comparison_bundle(&report);
}

fn main() {}
