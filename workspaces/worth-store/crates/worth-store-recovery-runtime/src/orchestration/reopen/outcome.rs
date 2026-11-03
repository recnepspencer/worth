//! Terminal fresh-reopen receivers extract the actual namespace payload once.
use super::{custody_indeterminate, custody_rebind, denial_counters, indeterminate};
use crate::entry::{PhysicalRecoveryOutcome, PhysicalRecoveryReopenFailure};
use crate::progression::NamespaceDurablePayload;
use crate::progression::{NamespaceDurablePhysicalRecovery, ReopenedPhysicalRecovery};

#[inline(never)]
pub(super) fn completed(
    mut durable: NamespaceDurablePhysicalRecovery,
    completed: worth_store::physical_runtime::CompletedPhysicalRecoveryFreshReopen,
) -> Result<ReopenedPhysicalRecovery, PhysicalRecoveryOutcome> {
    match durable.prepare_selected_custody_rebind(&completed) {
        Ok(pending_effective) => custody_rebind::complete(durable, completed, pending_effective),
        Err(()) => custody_denied(durable),
    }
}

#[inline(never)]
fn custody_denied(
    durable: NamespaceDurablePhysicalRecovery,
) -> Result<ReopenedPhysicalRecovery, PhysicalRecoveryOutcome> {
    let NamespaceDurablePayload {
        state,
        publication_counters,
        publication_settlement,
        ..
    } = durable.into_payload();
    Err(custody_indeterminate(
        state,
        publication_counters,
        publication_settlement,
    ))
}

#[inline(never)]
pub(super) fn denied(
    durable: NamespaceDurablePhysicalRecovery,
    denial: worth_store::physical_runtime::PhysicalRecoveryFreshReopenDenial,
) -> Result<ReopenedPhysicalRecovery, PhysicalRecoveryOutcome> {
    let NamespaceDurablePayload {
        state,
        publication_counters,
        publication_settlement,
        ..
    } = durable.into_payload();
    let counters = denial_counters(&denial);
    let failure = PhysicalRecoveryReopenFailure::new(counters, denial);
    Err(indeterminate(
        state,
        publication_counters,
        publication_settlement,
        failure,
    ))
}
