//! Terminal disposition of old WAL groups is admitted only after the V3
//! source/result verifier and the selected release/manifest gates complete.

use sha2::{Digest, Sha256};

use crate::entry::PhysicalRecoveryOutcome;
use crate::orchestration::planning::{
    context::PlanningContext, resolved_basis::ResolvedPlanningBasis,
};

pub(super) fn admit(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
) -> Result<PlanningContext, PhysicalRecoveryOutcome> {
    let selected = context.selection.root().selected();
    let selected_root_identity: [u8; 32] =
        Sha256::digest(selected.manifest().encode(selected.selector().format())).into();
    let Some(consumed) = basis.redo.admit_historical_consumed_operations(
        &basis.verified_historical_release_operations,
        selected_root_identity,
    ) else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    if consumed.operations().any(|operation| {
        let mut matching = basis.fates.operations().iter().filter(|fate| {
            fate.identity().idempotency() == operation
                && fate.fate() == worth_store_recovery_physics::RecoveryOperationFate::Indeterminate
        });
        matching.next().is_none() || matching.next().is_some()
    }) {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    basis.historical_consumed = Some(consumed);
    Ok(context)
}
