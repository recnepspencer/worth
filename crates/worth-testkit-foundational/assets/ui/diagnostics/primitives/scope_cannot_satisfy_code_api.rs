use worth_foundational::{foundational_diagnostic_code, foundational_diagnostic_scope, FoundationalDiagnosticCodeId};

fn needs_code(_code: FoundationalDiagnosticCodeId) {}

fn main() {
    let _code = foundational_diagnostic_code("merge.conflict").expect("valid code");
    let scope = foundational_diagnostic_scope("transition.merge").expect("valid scope");

    needs_code(scope);
}
