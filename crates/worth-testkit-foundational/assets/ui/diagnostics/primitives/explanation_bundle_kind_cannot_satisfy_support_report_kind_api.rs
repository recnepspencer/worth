use worth_foundational::{
    foundational_explanation_bundle_artifact_kind, FoundationalSupportReportArtifactKind,
};

fn needs_support_report_kind(_kind: FoundationalSupportReportArtifactKind) {}

fn main() {
    let explanation = foundational_explanation_bundle_artifact_kind();
    needs_support_report_kind(explanation);
}
