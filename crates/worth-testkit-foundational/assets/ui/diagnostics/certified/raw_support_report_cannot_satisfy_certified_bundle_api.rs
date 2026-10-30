use worth_foundational::{
    FoundationalCertifiedDiagnosticBundle, FoundationalDiagnosticSupportReport,
    CurrentBasisCommitReceiptArtifact,
};

fn require_certified_bundle(
    _bundle: &FoundationalCertifiedDiagnosticBundle<
        CurrentBasisCommitReceiptArtifact,
        FoundationalDiagnosticSupportReport,
    >,
) {
}

fn main() {
    let report = impossible_report();
    require_certified_bundle(&report);
}

fn impossible_report() -> FoundationalDiagnosticSupportReport {
    loop {
        std::hint::spin_loop();
    }
}
