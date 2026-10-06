//! Live planning backing at the source-custody admission boundary.
//! This is not accumulated I/O or a ledger of earlier transient windows.

use super::super::super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use crate::orchestration::planning::selected_source_inventory::ResidentAllowance;
use crate::orchestration::recovery_budget::RecoveryAllowance;
use crate::progression::PlanningCustody;

pub(super) fn seed(
    context: &PlanningContext,
    basis: &ResolvedPlanningBasis,
) -> Result<ResidentAllowance, Option<crate::entry::PhysicalRecoveryLimitFailure>> {
    // Final custody assembly cannot be used as the source of a second admission.
    if !matches!(
        &basis.custody,
        PlanningCustody::Unresolved | PlanningCustody::SourceHeads(_)
    ) {
        return Err(None);
    }
    // Live bytes past every count name no limit.
    let Some(live) = crate::orchestration::planning::resident_memory::live_bytes(context, basis)
    else {
        return Err(None);
    };
    let mut resident = ResidentAllowance::new(context.limits.recovery_memory_bytes);
    if resident.bytes(live).is_err() {
        return Err(limit_failure(context, &resident));
    }
    Ok(resident)
}

pub(super) fn limit_failure(
    context: &PlanningContext,
    resident: &ResidentAllowance,
) -> Option<crate::entry::PhysicalRecoveryLimitFailure> {
    resident
        .refused_in(RecoveryAllowance::declared(
            &context.limits,
            crate::entry::PhysicalRecoveryLimitDimension::RecoveryMemoryBytes,
        ))
        .map(Into::into)
}
