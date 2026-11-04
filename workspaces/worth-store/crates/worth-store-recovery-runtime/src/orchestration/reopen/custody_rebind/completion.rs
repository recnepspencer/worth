//! Owned observation follows borrowed preparation without a full-state Result.
use super::super::{completed_counters, custody_indeterminate};
use super::into_reopened_state;
use crate::entry::PhysicalRecoveryOutcome;
use crate::progression::{
    CustodyState, NamespaceDurablePayload, NamespaceDurablePhysicalRecovery, PlanningCustody,
    ReopenedPayload, ReopenedPhysicalRecovery,
};

#[inline(never)]
pub(in crate::orchestration::reopen) fn complete(
    durable: NamespaceDurablePhysicalRecovery,
    completed: worth_store::physical_runtime::CompletedPhysicalRecoveryFreshReopen,
    pending_effective: Option<worth_store_recovery_physics::VerifiedEffectiveReleaseHeadRosterV14>,
) -> Result<ReopenedPhysicalRecovery, PhysicalRecoveryOutcome> {
    let NamespaceDurablePayload {
        mut state,
        expectation,
        publication_counters,
        publication_settlement,
        reopened_backing,
    } = durable.into_payload();
    let counters = completed_counters(&completed);
    let planning = std::mem::replace(&mut state.custody, PlanningCustody::Unresolved);
    let custody = match (planning, pending_effective) {
        (PlanningCustody::NoCheckpoint, None) => CustodyState::NoCheckpoint,
        (PlanningCustody::NoRelease(claim), None) => CustodyState::NoRelease(claim),
        (PlanningCustody::SourceHeads(claim), None) => CustodyState::SourceHeads(claim),
        (PlanningCustody::PendingPrepared { claim, replay }, Some(effective_heads)) => {
            CustodyState::Pending {
                claim,
                replay: replay.into_head(),
                effective_heads,
            }
        }
        (
            PlanningCustody::OrderedCompleted {
                claim,
                effective_heads,
            },
            None,
        ) => CustodyState::OrderedCompleted {
            claim,
            effective_heads,
        },
        (planning @ PlanningCustody::NoCheckpoint, Some(_))
        | (planning @ PlanningCustody::NoRelease(_), Some(_))
        | (planning @ PlanningCustody::SourceHeads(_), Some(_))
        | (planning @ PlanningCustody::PendingPrepared { .. }, None)
        | (planning @ PlanningCustody::OrderedCompleted { .. }, Some(_))
        | (planning @ PlanningCustody::Unresolved, None)
        | (planning @ PlanningCustody::Unresolved, Some(_)) => {
            state.custody = planning;
            return Err(custody_indeterminate(
                state,
                publication_counters,
                publication_settlement,
            ));
        }
    };

    Ok(reopened_backing.finish(ReopenedPayload {
        state: into_reopened_state(state, custody),
        expectation,
        publication_counters,
        publication_settlement,
        reopened: Some(completed),
        reopen_counters: counters,
    }))
}
