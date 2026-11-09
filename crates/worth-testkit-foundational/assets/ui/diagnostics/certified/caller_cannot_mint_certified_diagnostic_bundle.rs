use worth_foundational::{
    CurrentBasisCommitReceiptArtifact, FoundationalCertifiedDiagnosticBundle,
    FoundationalDiagnosticSupportReport,
};

fn main() {
    let _ = FoundationalCertifiedDiagnosticBundle::<
        CurrentBasisCommitReceiptArtifact,
        FoundationalDiagnosticSupportReport,
    > {
        inner: impossible_inner(),
    };
}

fn impossible_inner<T>() -> T {
    loop {
        std::hint::spin_loop();
    }
}
