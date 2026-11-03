//! Terminal fresh-reopen receivers extract the actual namespace payload once.
use super::{
    completed_counters, custody_indeterminate, custody_rebind, denial_counters, indeterminate,
};
use crate::entry::{PhysicalRecoveryOutcome, PhysicalRecoveryReopenFailure};
use crate::progression::{NamespaceDurablePayload, ReopenedPayload};
use crate::progression::{NamespaceDurablePhysicalRecovery, ReopenedPhysicalRecovery};

#[inline(never)]
pub(super) fn completed(
    durable: NamespaceDurablePhysicalRecovery,
    completed: worth_store::physical_runtime::CompletedPhysicalRecoveryFreshReopen,
) -> Result<ReopenedPhysicalRecovery, PhysicalRecoveryOutcome> {
    let NamespaceDurablePayload {
        state,
        mut expectation,
        publication_counters,
        publication_settlement,
        reopened_backing,
    } = durable.into_payload();
    let counters = completed_counters(&completed);
    let state = match custody_rebind::rebind_selected_custody(
        state,
        &mut expectation,
        &publication_settlement,
        &publication_counters,
        &completed,
    ) {
        Ok(state) => state,
        Err(state) => {
            return Err(custody_indeterminate(
                state,
                publication_counters,
                publication_settlement,
            ))
        }
    };
    Ok(reopened_backing.finish(ReopenedPayload {
        state,
        expectation,
        publication_counters,
        publication_settlement,
        reopened: Some(completed),
        reopen_counters: counters,
    }))
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
