use worth_foundational::{
    FoundationalCertifiedDiagnosticBundle, FoundationalDiagnosticSupportReport,
    CurrentBasisCommitReceiptArtifact,
};

fn main() {
    let _ = FoundationalCertifiedDiagnosticBundle::<
        CurrentBasisCommitReceiptArtifact,
        FoundationalDiagnosticSupportReport,
    > {
        inner: impossible_inner(),
    };
}

fn impossible_inner() -> ! {
    loop {
        std::hint::spin_loop();
    }
}
