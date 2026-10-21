use super::PreparedPrerequisiteClaims;
use super::WorthQueryOutputDemandRegistry;

impl PreparedPrerequisiteClaims {
    pub(in super::super) fn cancel_unpublished_prerequisites(&mut self) {
        if self.published {
            return;
        }
        let owner = WorthQueryOutputDemandRegistry::clone(self.context.registry());
        let mut state = owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.reserved_identity.take().is_some() {
            state.defer_cancelled_settlement_vacancy(
                self.reserved_cleanup
                    .take()
                    .expect("prepared vacancy has an admitted cancellation cue"),
            );
        }
        self.reserved_posting.take();
        for upstream in &self.predecessors {
            let Some(mut record) = state.records.get_mut(upstream.as_ref()) else {
                continue;
            };
            record.framework_required_count -= 1;
            let pending = record.pending_cleanup_queued;
            record.finish();
            if !pending && !state.defer_terminal_cleanup(upstream) {
                state.remove_required_member_if_released(upstream);
            }
        }
        for upstream in &self.predecessors {
            state.defer_terminal_cleanup(upstream);
        }
        state
            .records
            .get_mut(self.context.key())
            .expect("prepared demand stays pinned")
            .prepared_prerequisite_claims -= 1;
        state.defer_terminal_cleanup(self.context.key_arc());
    }
}
