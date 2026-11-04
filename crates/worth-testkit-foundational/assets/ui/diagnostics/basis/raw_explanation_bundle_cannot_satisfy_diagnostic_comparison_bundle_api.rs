use worth_foundational::{FoundationalDiagnosticComparisonBundle, FoundationalDiagnosticExplanationBundle};

fn require_comparison_bundle(_bundle: &FoundationalDiagnosticComparisonBundle) {}

fn rejects_raw_value(bundle: FoundationalDiagnosticExplanationBundle) {
    require_comparison_bundle(&bundle);
}

fn main() {}
