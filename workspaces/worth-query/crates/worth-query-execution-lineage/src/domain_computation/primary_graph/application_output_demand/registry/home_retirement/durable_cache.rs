//! Release only quiescent process caches whose unfinished work is acknowledged DATA.
use super::*;
use crate::domain_computation::primary_graph::query_obligations::history::QueryObligationHistory;

impl WorthQueryOutputDemandRegistry {
    pub fn release_durable_home_cache(
        &mut self,
        history: &QueryObligationHistory,
    ) -> Result<(), Denial> {
        let busy = || Denial::Busy(Owner::InstalledProducers);
        let state = Arc::get_mut(&mut self.state)
            .ok_or_else(busy)?
            .get_mut()
            .map_err(|_| busy())?;
        if state.records.is_empty() && state.source_custody.is_empty() {
            return Ok(());
        }
        if state.join_pause.is_some()
            || !state.source_preparations.is_empty()
            || state.retention_reservation.is_some()
            || state.pending_cleanup_head.is_some()
            || state.custody_cleanup.head.is_some()
            || state.custody_cleanup.len != 0
            || !state.closed_cleanup.is_quiescent()
            || !state.settlement_keys.is_quiescent()
            || !state
                .records
                .values()
                .all(|row| row.can_handoff_durable(history))
        {
            return Err(busy());
        }
        let sources = history.output_sources();
        for (commit, custody) in &state.source_custody {
            let durable = commit.owner_identity().durable().map(|owner| {
                worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
                    owner,
                    commit.ordinal(),
                )
            });
            if custody.token_count != 0
                || custody.completed
                || custody.retired.is_some()
                || durable.is_none_or(|identity| !sources.iter().any(|row| row.commit == identity))
            {
                return Err(busy());
            }
        }
        for row in state.records.values() {
            if let Some(member) = &row.work_membership {
                member.set_required(false);
            }
        }
        // Dropping cached sources/producer references never completes durable membership.
        // All native owners are released before the installed handler alias check.
        let released = std::mem::take(state);
        drop(released);
        Ok(())
    }
}

impl DemandRecord {
    fn can_handoff_durable(&self, history: &QueryObligationHistory) -> bool {
        let quiescent = match &self.state {
            DemandState::Admitted | DemandState::Scheduled => true,
            DemandState::Output(output) => {
                output.checkpoint.is_none()
                    && matches!(output.advancement(), WorthQueryOutputAdvancement::Idle)
            }
            DemandState::Scheduling | DemandState::Running | DemandState::Failed(_) => false,
        };
        quiescent
            && self.interests == 0
            && self.required_interests == 0
            && self.framework_required_count == 0
            && self.claim_batch.is_none()
            && self.checkpoint_prerequisites.is_none()
            && self.prepared_prerequisite_claims == 0
            && !self.pending_cleanup_queued
            && !self.closed_cleanup.queued()
            && self.held_successor.is_none()
            && self.successor_of.is_none()
            && self.readmission_source.as_ref().is_some_and(|source| {
                source.durable.lock().is_ok_and(|held| {
                    held.as_ref().is_some_and(|membership| {
                        std::ptr::eq(membership.history.as_ref(), history)
                            && membership.pending()
                            && history.output_demand_retained(membership.position)
                    })
                })
            })
    }
}
