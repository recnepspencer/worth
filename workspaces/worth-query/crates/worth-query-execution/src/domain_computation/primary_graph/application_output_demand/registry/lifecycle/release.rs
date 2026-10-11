//! Closing the last interest must complete without a request.
mod cancelled_prerequisites;
use super::super::refreshed_rejoin::infallible;
use super::*;

/// The private field seals construction to this release module. Its only
/// production constructor is the interest Drop release entry below.
pub(in super::super) struct CleanupWalk {
    _release: (),
}

impl WorthQueryOutputDemandRegistry {
    pub(super) fn release(&self, interest: &WorthQueryOutputDemandInterest) {
        let mut charge = CleanupWalk { _release: () };
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.note_record_custody(&interest.key);
        let mut released_bytes = 0;
        let replaced = infallible(refreshed_rejoin::replaced_under_refresh(
            &state.records,
            &interest.key,
            &mut charge,
        ));
        let remove = state.records.get_mut(&interest.key).is_some_and(|record| {
            record.interests = record.interests.saturating_sub(1);
            if interest.requires_output {
                record.required_interests = record.required_interests.saturating_sub(1);
            }
            // Keep a ready ordinary output for an equivalent later demand.
            // Source supersession and occurrence retirement remove it; closing
            // the handle releases only the caller's interest.
            let ready = record.has_cached_ready();
            let stopped = matches!(record.state, DemandState::Failed(_))
                || matches!(&record.state, DemandState::Output(output)
                    if matches!(output.advancement, WorthQueryOutputAdvancement::Stopped { .. }));
            if stopped && record.interests == 0 && !record.pending_cleanup_queued {
                released_bytes += record.release_obligations();
            }
            record.interests == 0
                && record.framework_required_count == 0
                && record.prepared_prerequisite_claims == 0
                && !record.pending_cleanup_queued
                && ((!record.is_required() && !ready) || stopped)
                && !(ready && replaced)
                && record.performed_obligations.is_empty()
                && record.performed_source.is_none()
                && match &record.state {
                    DemandState::Scheduling | DemandState::Running => false,
                    DemandState::Output(output) => {
                        matches!(
                            output.advancement,
                            WorthQueryOutputAdvancement::Stopped { .. }
                        ) || matches!(
                            output.checkpoint,
                            Some(WorthQueryOutputCheckpoint::Ready(_))
                        )
                    }
                    _ => true,
                }
        });
        let terminal = state.records.get(&interest.key).is_some_and(|record| {
            matches!(record.state, DemandState::Failed(_))
                || matches!(&record.state, DemandState::Output(output)
                    if matches!(output.advancement, WorthQueryOutputAdvancement::Stopped { .. }))
        });
        let released_prerequisites = terminal
            .then(|| infallible(state.release_record_prerequisites(&interest.key, &mut charge)));
        if remove {
            if let Some(record) = state.records.remove(&interest.key) {
                released_bytes += record.obligation_reserved_bytes();
                if record.unpublished_new_key() {
                    infallible(state.revive_replaced_ready(&interest.key, &mut charge));
                }
            }
        }
        if !state
            .records
            .get(&interest.key)
            .is_some_and(|record| record.pending_cleanup_queued)
        {
            infallible(state.remove_required_member_if_released(&interest.key, &mut charge));
        }
        // A held successor ends with its row's work or with the last owner
        // awaiting that row; it drops after the lock.
        let mut finished = state.take_finished_successor(&interest.key, &mut charge);
        let unawaited = state
            .records
            .get_mut(&interest.key)
            .and_then(DemandRecord::take_unawaited_reopened);
        // A closing stale owner may have been the last one awaiting the
        // newest row of its occurrence.
        if terminal {
            if let Some(newest) = infallible(refreshed_rejoin::newest_of_occurrence(
                &state.records,
                &interest.key,
                &mut charge,
            )) {
                state.note_record_custody(&newest);
                infallible(state.remove_required_member_if_released(&newest, &mut charge));
                finished = finished.or_else(|| state.take_finished_successor(&newest, &mut charge));
            }
        }
        state.obligation_reserved_bytes = state
            .obligation_reserved_bytes
            .saturating_sub(released_bytes);
        // The closing row, each upstream its released claims leave unheld,
        // and each row it replaced, keeps only what it can still answer for.
        let candidates = std::iter::once(interest.key.clone())
            .chain(
                released_prerequisites
                    .iter()
                    .flatten()
                    .map(|upstream| upstream.as_ref().clone()),
            )
            .chain(infallible(refreshed_rejoin::replaced_by_published(
                &state.records,
                &interest.key,
                &mut charge,
            )))
            .collect();
        // A subsequent admission-free Drop can remove the newest member.
        // Transfer closed claims before those Drops run; see sweep-debt.md.
        let (retired, released_claims) =
            infallible(state.retire_closed_rows(candidates, &mut charge));
        drop(state);
        drop(retired);
        drop(released_claims);
        drop(released_prerequisites);
        drop(finished);
        drop(unawaited);
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
