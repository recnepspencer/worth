//! Closing the last interest must complete without a request.
mod cancelled_prerequisites;
mod closed_claims;

use super::*;

impl WorthQueryOutputDemandRegistry {
    pub(super) fn release(&self, interest: &WorthQueryOutputDemandInterest) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.note_record_custody(&interest.key);
        let replaced = refreshed_rejoin::replaced_under_refresh(&state.records, &interest.key);
        if let Some(mut record) = state.records.get_mut(&interest.key) {
            record.interests = record.interests.saturating_sub(1);
            if interest.requires_output {
                record.required_interests = record.required_interests.saturating_sub(1);
            }
        }
        let needs_awaited = state.records.get(&interest.key).is_some_and(|row| matches!(&row.state,
            DemandState::Output(output) if matches!(output.advancement(), WorthQueryOutputAdvancement::Idle)
            && matches!(output.checkpoint, Some(WorthQueryOutputCheckpoint::Published { .. } | WorthQueryOutputCheckpoint::Delivered { .. }))));
        let awaited = needs_awaited
            && refreshed_rejoin::awaited_by_stale_owner(&state.records, &interest.key);
        let mut retirement = None;
        let remove = state
            .records
            .get_mut(&interest.key)
            .is_some_and(|mut record| {
                // Keep a ready ordinary output for an equivalent later demand.
                // Source supersession and occurrence retirement remove it; closing
                // the handle releases only the caller's interest.
                let ready = record.has_cached_ready();
                retirement = match record.idle_disposition(awaited) {
                    IdleRowDisposition::InFlight
                    | IdleRowDisposition::KeptAnswer
                    | IdleRowDisposition::HeldForStaleOwner => None,
                    IdleRowDisposition::Retired(reason) => Some(reason),
                };
                let stopped = retirement.is_some_and(RowRetirement::releases_finished_work);
                if ready && record.interests == 0 && !stopped {
                    crate::domain_computation::primary_graph::fact_verification_observation::registry_custody(
                        crate::domain_computation::primary_graph::WorthQueryRegistryCustodyTransitionForTest::CachedReadyClosed,
                        record.settlements.len(), record.prerequisites.len(),
                    );
                }
                if stopped
                    && record.interests == 0
                    && !record.pending_cleanup_queued
                    && !record.closed_cleanup.queued()
                {
                    record.release_obligations();
                }
                record.interests == 0
                    && record.framework_required_count == 0
                    && record.prepared_prerequisite_claims == 0
                    && !record.pending_cleanup_queued
                    && !record.closed_cleanup.queued()
                    && ((!record.is_required() && !ready) || stopped)
                    && !(ready && replaced)
                    && record.performed_obligations.is_empty()
                    && record.performed_source.is_none()
                    && retirement.is_some()
            });
        let terminal = retirement.is_some_and(RowRetirement::releases_finished_work);
        let queued = state
            .records
            .get(&interest.key)
            .is_some_and(|row| row.closed_cleanup.queued());
        let released_prerequisites =
            (terminal && !queued).then(|| state.release_record_prerequisites(&interest.key));
        if remove
            && let Some(record) = state.remove_record(&interest.key)
            && record.unpublished_new_key()
        {
            state.revive_replaced_ready(&interest.key);
        }
        // Cached Ready membership is occurrence-local cleanup even while its
        // claim transfer remains noted. Terminal resources stay with their debt.
        if (!queued || !terminal)
            && !state
                .records
                .get(&interest.key)
                .is_some_and(|record| record.pending_cleanup_queued)
        {
            state.remove_required_member_if_released(&interest.key);
        }
        let released_custody = state.release_closed_claim_custody(&interest.key);
        // A held successor ends with its row's work or with the last owner
        // awaiting that row; it drops after the lock.
        let mut finished = state.take_finished_successor(&interest.key);
        let unawaited = state
            .records
            .get_mut(&interest.key)
            .and_then(|mut record| record.take_unawaited_reopened());
        // A closing stale owner may have been the last one awaiting the
        // newest row of its occurrence.
        let mut retired_newest = Vec::new();
        let mut newest_claims = Vec::new();
        if terminal
            && !state
                .records
                .get(&interest.key)
                .is_some_and(|row| row.closed_cleanup.queued())
            && let Some(newest) =
                refreshed_rejoin::newest_of_occurrence(&state.records, &interest.key)
        {
            state.note_record_custody(&newest);
            state.remove_required_member_if_released(&newest);
            finished = finished.or_else(|| state.take_finished_successor(&newest));
            let awaited = refreshed_rejoin::awaited_by_stale_owner(&state.records, &newest);
            let abandoned = state.records.get(&newest).is_some_and(|row| {
                matches!(
                    row.idle_disposition(awaited),
                    IdleRowDisposition::Retired(RowRetirement::AbandonedPublication)
                ) && !super::closed_retirement::held_by_row(row)
            });
            if abandoned {
                newest_claims = state.retire_row(&newest, &mut retired_newest);
            }
        }
        drop(state);
        drop(released_prerequisites);
        drop(finished);
        drop(unawaited);
        drop(released_custody);
        drop(retired_newest);
        drop(newest_claims);
    }
}

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn release_prepared_token(
        &self,
        source_commit: &worth_runtime_world::facade::CompositeCommitIdentity,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(custody) = state.source_custody.get_mut(source_commit) {
            custody.token_count = custody.token_count.saturating_sub(1);
        }
        state.note_custody(source_commit);
    }
}
