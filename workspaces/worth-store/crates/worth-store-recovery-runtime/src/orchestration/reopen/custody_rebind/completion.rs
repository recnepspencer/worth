//! Owned observation follows borrowed preparation without a full-state Result.
use super::super::{completed_counters, custody_indeterminate};
use super::{into_reopened_state, observed_published_free_header};
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
    // Preserve Store's generation-zero custody path before selected predicates/reads.
    if matches!(&state.custody, PlanningCustody::NoCheckpoint)
        && state.verified_selected_tier_custody.is_none()
    {
        return Ok(reopened_backing.finish(ReopenedPayload {
            state: into_reopened_state(state, CustodyState::NoCheckpoint),
            expectation,
            publication_counters,
            publication_settlement,
            reopened: Some(completed),
            reopen_counters: counters,
        }));
    }
    let published = completed.root();
    let selected_free = if let Some(claim) = &state.verified_selected_tier_custody {
        let byte_limit = claim.free_header().encode(completed.format()).len() as u64;
        let (media, observed) = observed_published_free_header(
            state.authority.media,
            &mut state.integrity_trace,
            published,
            completed.format(),
            byte_limit,
        );
        state.authority.media = media;
        match observed {
            Some(header) => Some(header),
            None => {
                return Err(custody_indeterminate(
                    state,
                    publication_counters,
                    publication_settlement,
                ))
            }
        }
    } else {
        None
    };
    if let Some(claim) = &mut state.verified_selected_tier_custody
        && claim
            .rebind_published_root_and_free_header(
                &state.selection,
                published,
                selected_free
                    .as_ref()
                    .expect("tier claim requires observed free header"),
                completed.format(),
            )
            .is_err()
    {
        return Err(custody_indeterminate(
            state,
            publication_counters,
            publication_settlement,
        ));
    }
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
        (planning, _) => {
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
