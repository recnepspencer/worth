use worth_foundational::FoundationalDiagnosticExplanationBundle;
use worth_foundational::FoundationalDiagnosticRow;

fn needs_explanation_bundle(_bundle: FoundationalDiagnosticExplanationBundle) {}

fn main() {
    let rows: Vec<FoundationalDiagnosticRow> = Vec::new();
    needs_explanation_bundle(rows);
}
