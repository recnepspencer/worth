use super::BridgePolicyRejection;
use super::BridgePolicyRejectionKind;
use worth_execution::report::{
    DenialClass::{self, *},
    Disposition::{self, *},
    ReportCode, Reportable,
};

impl std::fmt::Display for BridgePolicyRejection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Bridge policy was rejected.")
    }
}

impl std::error::Error for BridgePolicyRejection {}

impl Reportable for BridgePolicyRejection {
    fn denial_class(&self) -> DenialClass {
        report(self).0
    }

    fn disposition(&self) -> Disposition {
        report(self).1
    }

    fn report_code(&self) -> ReportCode {
        ReportCode::new(report(self).2).expect("valid static Bridge report code")
    }

    fn diagnostic_summary(&self) -> &'static str {
        "Bridge policy was rejected."
    }
}

fn report(denial: &BridgePolicyRejection) -> (DenialClass, Disposition, &'static str) {
    match denial.kind() {
        BridgePolicyRejectionKind::PolicySourceAmbiguity => {
            (Conflict, Final, "bridge.policy.policy_source_ambiguity")
        }
        BridgePolicyRejectionKind::UnsupportedExecutionMode => (
            InvalidInput,
            CorrectInput,
            "bridge.policy.unsupported_execution_mode",
        ),
        BridgePolicyRejectionKind::ReplayPolicyConflict => {
            (Conflict, Final, "bridge.policy.replay_policy_conflict")
        }
        BridgePolicyRejectionKind::DiagnosticsPolicyConflict => {
            (Conflict, Final, "bridge.policy.diagnostics_policy_conflict")
        }
        BridgePolicyRejectionKind::ArtifactRetentionConflict => {
            (Conflict, Final, "bridge.policy.artifact_retention_conflict")
        }
        BridgePolicyRejectionKind::PreviewPolicyBoundaryViolation => (
            Authorization,
            Reauthorize,
            "bridge.policy.preview_policy_boundary_violation",
        ),
        BridgePolicyRejectionKind::TruthViewPolicyConflict => {
            (Conflict, Final, "bridge.policy.truth_view_policy_conflict")
        }
        BridgePolicyRejectionKind::PolicyLoweringMismatch => (
            Invariant,
            Escalate,
            "bridge.policy.policy_lowering_mismatch",
        ),
    }
}
