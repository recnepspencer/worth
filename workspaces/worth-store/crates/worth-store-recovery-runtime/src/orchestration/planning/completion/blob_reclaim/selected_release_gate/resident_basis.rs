//! Live planning backing at the source-custody admission boundary.
//! This is not accumulated I/O or a ledger of earlier transient windows.

use super::super::super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use crate::orchestration::planning::selected_source_inventory::ResidentAllowance;

pub(super) fn seed(
    context: &PlanningContext,
    basis: &ResolvedPlanningBasis,
) -> Result<ResidentAllowance, Option<crate::entry::PhysicalRecoveryLimitFailure>> {
    // Final custody assembly cannot be used as the source of a second admission.
    if basis.verified_selected_checkpoint_custody.is_some()
        || basis.verified_pending_wal_release_custody.is_some()
        || basis.verified_ordered_historical_release_custody.is_some()
        || basis.verified_effective_release_heads_v14.is_some()
    {
        return Err(None);
    }
    let mut resident = ResidentAllowance::new(context.limits.recovery_memory_bytes);
    if resident
        .bytes(
            crate::orchestration::planning::resident_memory::live_bytes(context, basis)
                .unwrap_or(u64::MAX),
        )
        .is_err()
    {
        return Err(limit_failure(context, &resident));
    }
    Ok(resident)
}

pub(super) fn limit_failure(
    context: &PlanningContext,
    resident: &ResidentAllowance,
) -> Option<crate::entry::PhysicalRecoveryLimitFailure> {
    resident
        .exceeded_requirement()
        .map(|observed| crate::entry::PhysicalRecoveryLimitFailure {
            dimension: crate::entry::PhysicalRecoveryLimitDimension::RecoveryMemoryBytes,
            observed,
            admitted: context.limits.recovery_memory_bytes,
        })
}
