use crate::merge::data::{MergePolicyDecisionBoundary, MergePolicyOwnershipSurface};
use worth_relational_contracts::internal_port::diagnostics::data::fields::RelationalDiagnosticValue;

pub(super) fn merge_policy_proof_boundary_fields(
    boundary: crate::merge::data::MergePolicyProofBoundary,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "ownership_surface",
            merge_policy_ownership_surface(boundary.ownership_surface),
        ),
        (
            "decision_boundary",
            merge_policy_decision_boundary(boundary.decision_boundary),
        ),
    ])
}

fn merge_policy_ownership_surface(
    surface: MergePolicyOwnershipSurface,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::string(format!("{surface:?}"))
}

fn merge_policy_decision_boundary(
    boundary: MergePolicyDecisionBoundary,
) -> RelationalDiagnosticValue {
    match boundary {
        MergePolicyDecisionBoundary::AutoResolved => RelationalDiagnosticValue::object([(
            "kind",
            RelationalDiagnosticValue::string("auto_resolved"),
        )]),
        MergePolicyDecisionBoundary::RequiresManualResolution { class } => {
            RelationalDiagnosticValue::object([
                (
                    "kind",
                    RelationalDiagnosticValue::string("requires_manual_resolution"),
                ),
                (
                    "class",
                    RelationalDiagnosticValue::string(format!("{class:?}")),
                ),
            ])
        }
        MergePolicyDecisionBoundary::Reject { class } => RelationalDiagnosticValue::object([
            ("kind", RelationalDiagnosticValue::string("reject")),
            (
                "class",
                RelationalDiagnosticValue::string(format!("{class:?}")),
            ),
        ]),
    }
}
