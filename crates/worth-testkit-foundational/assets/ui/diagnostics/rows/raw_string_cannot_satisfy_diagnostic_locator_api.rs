use worth_foundational::FoundationalDiagnosticLocator;

fn needs_diagnostic_locator(_locator: FoundationalDiagnosticLocator) {}

fn main() {
    needs_diagnostic_locator("transition.merge.conflict");
}
