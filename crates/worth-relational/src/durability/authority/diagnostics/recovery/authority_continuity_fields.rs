use crate::diagnostics::data::{RelationalDiagnosticFields, RelationalDiagnosticValue};
use crate::durability::authority::diagnostics::recovery::authority_continuity_mismatch_fields::authority_continuity_mismatch_fields;
use crate::durability::authority::diagnostics::recovery::durable_identity_fields::verification_layer_value;
use crate::durability::data::{RecoveryAuthorityParity, RecoveryPlan, RecoveryVerificationOutcome};
use crate::publication::patch::data::PatchStreamPosition;

pub(super) fn recovery_authority_continuity_evaluated_fields(
    plan: &RecoveryPlan,
) -> RelationalDiagnosticFields {
    let verification =
        VerificationOutcomeDiagnostic::from(&plan.authority_continuity.verification_outcome);

    RelationalDiagnosticValue::object([
        (
            "verification_mode",
            RelationalDiagnosticValue::string(format!("{:?}", plan.verification_mode())),
        ),
        (
            "verification_layer",
            verification_layer_value(verification.layer),
        ),
        (
            "verification_rejected",
            RelationalDiagnosticValue::Bool(verification.rejected),
        ),
        (
            "verification_detail",
            RelationalDiagnosticValue::optional(
                verification.detail.map(RelationalDiagnosticValue::string),
            ),
        ),
        (
            "descriptor_semantics_version",
            RelationalDiagnosticValue::DescriptorSemanticsVersion(
                plan.descriptor_semantics_version,
            ),
        ),
        (
            "first_mismatch",
            RelationalDiagnosticValue::optional(
                plan.authority_continuity
                    .first_mismatch
                    .as_ref()
                    .map(authority_continuity_mismatch_fields),
            ),
        ),
        (
            "schema_parity",
            recovery_authority_parity_value(plan.authority_continuity.schema_parity),
        ),
        (
            "profile_parity",
            recovery_authority_parity_value(plan.authority_continuity.profile_parity),
        ),
        (
            "runtime_name_parity",
            recovery_authority_parity_value(plan.authority_continuity.runtime_name_parity),
        ),
        (
            "descriptor_version_parity",
            recovery_authority_parity_value(plan.authority_continuity.descriptor_version_parity),
        ),
        (
            "schema_transition_parity",
            recovery_authority_parity_value(plan.authority_continuity.schema_transition_parity),
        ),
        (
            "continuation_descriptor_parity",
            recovery_authority_parity_value(
                plan.authority_continuity.continuation_descriptor_parity,
            ),
        ),
        (
            "reconciliation_descriptor_parity",
            recovery_authority_parity_value(
                plan.authority_continuity.reconciliation_descriptor_parity,
            ),
        ),
        (
            "schema_lineage_parity",
            recovery_authority_parity_value(plan.authority_continuity.schema_lineage_parity),
        ),
    ])
    .into()
}

pub(super) fn recovery_checkpoint_selected_fields(
    checkpoint_covered_position: Option<PatchStreamPosition>,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([(
        "checkpoint_covered_position",
        RelationalDiagnosticValue::optional(
            checkpoint_covered_position.map(RelationalDiagnosticValue::PatchStreamPosition),
        ),
    )])
    .into()
}

pub(super) fn recovery_range_replayed_fields(tail_commits: usize) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([(
        "tail_commits",
        RelationalDiagnosticValue::unsigned(tail_commits),
    )])
    .into()
}

struct VerificationOutcomeDiagnostic<'a> {
    layer: crate::replay::data::ReplayVerificationLayer,
    rejected: bool,
    detail: Option<&'a str>,
}

impl<'a> From<&'a RecoveryVerificationOutcome> for VerificationOutcomeDiagnostic<'a> {
    fn from(outcome: &'a RecoveryVerificationOutcome) -> Self {
        match outcome {
            RecoveryVerificationOutcome::VerifiedAtLayer(layer) => Self {
                layer: *layer,
                rejected: false,
                detail: None,
            },
            RecoveryVerificationOutcome::Rejected { layer, detail } => Self {
                layer: *layer,
                rejected: true,
                detail: Some(detail),
            },
        }
    }
}

fn recovery_authority_parity_value(parity: RecoveryAuthorityParity) -> RelationalDiagnosticValue {
    match parity {
        RecoveryAuthorityParity::VerifiedAtLayer(layer) => RelationalDiagnosticValue::object([
            (
                "parity",
                RelationalDiagnosticValue::string("VerifiedAtLayer"),
            ),
            ("verification_layer", verification_layer_value(layer)),
        ]),
        RecoveryAuthorityParity::Drift => RelationalDiagnosticValue::object([(
            "parity",
            RelationalDiagnosticValue::string("Drift"),
        )]),
    }
}
