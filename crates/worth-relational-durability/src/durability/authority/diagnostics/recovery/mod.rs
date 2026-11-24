pub mod authority_continuity_fields;
pub mod authority_continuity_mismatch_fields;
pub mod durable_identity_fields;

use crate::durability::authority::diagnostics::recovery::authority_continuity_fields::{
    recovery_authority_continuity_evaluated_fields, recovery_checkpoint_selected_fields,
    recovery_range_replayed_fields,
};
use crate::durability::data::RecoveryPlan;
use worth_relational_contracts::internal_port::diagnostics::data::artifacts::{
    DiagnosticCode, RelationalDiagnosticsEntry,
};
use worth_relational_contracts::internal_port::publication::patch::data::patch_position::PatchStreamPosition;

pub(in crate::internal_port::durability::authority) fn recovery_authority_continuity_evaluated(
    plan: &RecoveryPlan,
) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::DurableRecoveryAuthorityContinuityEvaluated,
        "durable recovery authority continuity evaluated before recovery execution",
        recovery_authority_continuity_evaluated_fields(plan),
    )
}

pub fn recovery_checkpoint_selected(
    checkpoint_covered_position: Option<PatchStreamPosition>,
) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::RecoveryCheckpointSelected,
        "recovery checkpoint selected",
        recovery_checkpoint_selected_fields(checkpoint_covered_position),
    )
}

pub fn recovery_range_replayed(tail_commits: usize) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::RecoveryRangeReplayed,
        "durable tail replayed",
        recovery_range_replayed_fields(tail_commits),
    )
}
