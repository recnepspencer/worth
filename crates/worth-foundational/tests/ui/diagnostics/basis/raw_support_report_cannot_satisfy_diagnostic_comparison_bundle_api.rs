use worth_foundational::FoundationalDiagnosticComparisonBundle;
use worth_foundational::{
    materialize_diagnostic_support_report, DiagnosticRichnessProfile,
    FoundationalDiagnosticDeliveryClass, FoundationalDiagnosticPartiality,
    FoundationalDiagnosticSupportClaimStrength, SupportPostureProfile,
};

#[path = "../../../worth_foundational/fixtures/certification/diagnostics/materialization_support.rs"]
mod materialization_support;

fn require_comparison_bundle(_bundle: &FoundationalDiagnosticComparisonBundle) {}

fn main() {
    let report = materialize_diagnostic_support_report(
        materialization_support::support_input(
            FoundationalDiagnosticSupportClaimStrength::DescriptiveOnly,
            FoundationalDiagnosticPartiality::Complete,
            vec![],
        ),
        materialization_support::profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::SupportReady,
        ),
        FoundationalDiagnosticDeliveryClass::CanDefer,
    )
    .expect("support report");

    require_comparison_bundle(&report);
}
