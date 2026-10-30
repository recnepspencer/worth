use worth_foundational::FoundationalDiagnosticComparisonBundle;
use worth_foundational::{
    materialize_diagnostic_explanation_bundle, DiagnosticRichnessProfile,
    FoundationalDiagnosticDeliveryClass, FoundationalDiagnosticPartiality, SupportPostureProfile,
};

use worth_testkit_foundational::certification::ordinary_context::diagnostics::materialization_support;

fn require_comparison_bundle(_bundle: &FoundationalDiagnosticComparisonBundle) {}

fn main() {
    let bundle = materialize_diagnostic_explanation_bundle(
        materialization_support::explanation_input(FoundationalDiagnosticPartiality::Complete),
        materialization_support::profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::SupportReady,
        ),
        FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("explanation bundle");

    require_comparison_bundle(&bundle);
}
