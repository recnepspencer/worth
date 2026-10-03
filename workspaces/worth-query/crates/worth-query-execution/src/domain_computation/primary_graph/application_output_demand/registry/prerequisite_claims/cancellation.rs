use super::{PreparedPrerequisiteClaims, WorthQueryOutputDemandRegistry};

impl Drop for PreparedPrerequisiteClaims {
    fn drop(&mut self) {
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
            if let Some(record) = state.records.get_mut(upstream.as_ref()) {
                record.framework_required_count -= 1;
                state.remove_required_member_if_released(upstream);
            }
        }
        for upstream in &self.predecessors {
            state.defer_terminal_cleanup(upstream, 0);
        }
        state
            .records
            .get_mut(self.context.key())
            .expect("prepared demand stays pinned")
            .prepared_prerequisite_claims -= 1;
        if state.defer_terminal_cleanup(self.context.key_arc(), self.context.retained_bytes()) {
            self.context.transfer_custody_to_registry();
        }
        state.required_reserved_bytes = state
            .required_reserved_bytes
            .saturating_sub(self.predecessor_slots_bytes);
    }
}
