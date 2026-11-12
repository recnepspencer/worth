//! Preserve each actual publication fate in its own terminal receiver.
use super::counters::{completed_counters, denial_counters, indeterminate_counters};
use super::state::PublicationState;
use crate::entry::{PhysicalRecoveryOutcome, PhysicalRecoveryPublicationSettlement};
use crate::progression::{NamespaceDurablePhysicalRecovery, RecoveryPublicationExpectation};
use worth_store::physical_runtime::{
    CompletedPhysicalRecoveryPublicationCommand, PhysicalRecoveryPublicationCommandDenial,
    PhysicalRecoveryPublicationCommandIndeterminate,
};

#[inline(never)]
pub(super) fn completed(
    state: PublicationState,
    expectation: RecoveryPublicationExpectation,
    planned_effects: u64,
    completed: CompletedPhysicalRecoveryPublicationCommand,
) -> Result<NamespaceDurablePhysicalRecovery, PhysicalRecoveryOutcome> {
    let counters = completed_counters(planned_effects, &completed);
    let settlement = PhysicalRecoveryPublicationSettlement::Completed(completed);
    if state.is_ready() {
        Ok(state.into_namespace(expectation, counters, settlement))
    } else {
        Err(state.indeterminate(counters, settlement))
    }
}

#[inline(never)]
pub(super) fn denied(
    state: PublicationState,
    planned_effects: u64,
    denial: PhysicalRecoveryPublicationCommandDenial,
) -> PhysicalRecoveryOutcome {
    let counters = denial_counters(planned_effects, &denial);
    let escaped = !denial.candidates().is_empty()
        || denial.candidate_materialization().is_some()
        || denial.root_protocol().is_some();
    let settlement = PhysicalRecoveryPublicationSettlement::DeniedBeforeEffect(denial);
    if escaped {
        state.indeterminate(counters, settlement)
    } else {
        state.block_settlement(counters, settlement)
    }
}

#[inline(never)]
pub(super) fn indeterminate(
    state: PublicationState,
    planned_effects: u64,
    result: PhysicalRecoveryPublicationCommandIndeterminate,
) -> PhysicalRecoveryOutcome {
    let counters = indeterminate_counters(planned_effects, &result);
    state.indeterminate(
        counters,
        PhysicalRecoveryPublicationSettlement::Indeterminate(result),
    )
}
