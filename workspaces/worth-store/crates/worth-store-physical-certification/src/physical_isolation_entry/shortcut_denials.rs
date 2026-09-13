use worth_store_physical_isolation::{
    PhysicalIsolationEntryDenial, PhysicalIsolationEntryRebindRequired,
};

use super::admission::PhysicalIsolationEntryCheckedOutcome;

pub const fn reject_copied_recovery_fields_as_physical_isolation_entry(
) -> Result<(), PhysicalIsolationEntryDenial> {
    Err(PhysicalIsolationEntryDenial::CopiedRecoveryFields)
}

pub const fn reject_live_runtime_state_as_physical_isolation_entry(
) -> Result<(), PhysicalIsolationEntryDenial> {
    Err(PhysicalIsolationEntryDenial::LiveRuntimeState)
}

pub const fn reject_terminal_projection_as_physical_isolation_entry(
) -> Result<(), PhysicalIsolationEntryDenial> {
    Err(PhysicalIsolationEntryDenial::TerminalProjection)
}

pub const fn reject_semantic_snapshot_as_physical_isolation_entry(
) -> Result<(), PhysicalIsolationEntryDenial> {
    Err(PhysicalIsolationEntryDenial::SemanticSnapshot)
}

pub const fn reject_json_authority_as_physical_isolation_entry(
) -> Result<(), PhysicalIsolationEntryDenial> {
    Err(PhysicalIsolationEntryDenial::JsonAuthority)
}

pub const fn reject_foundational_or_proof_projection_as_physical_isolation_entry(
) -> Result<(), PhysicalIsolationEntryDenial> {
    Err(PhysicalIsolationEntryDenial::FoundationalOrProofProjection)
}

pub const fn reject_stale_recovery_readiness_as_physical_isolation_entry(
) -> PhysicalIsolationEntryCheckedOutcome {
    PhysicalIsolationEntryCheckedOutcome::Stale(
        PhysicalIsolationEntryDenial::StaleRecoveryReadiness,
    )
}

pub const fn require_rebound_recovery_readiness_for_physical_isolation_entry(
) -> PhysicalIsolationEntryCheckedOutcome {
    PhysicalIsolationEntryCheckedOutcome::RebindRequired(
        PhysicalIsolationEntryRebindRequired::RecoveryReadinessMustBeRebound,
    )
}
