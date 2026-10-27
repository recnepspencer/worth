use super::super::ResourceRuntimeState;
use crate::data::resource::ResourceRetentionCompactionBudget;

impl ResourceRuntimeState {
    /// Expiry is diagnostic loss, not a new exact terminal classification.
    /// Live/pending requests never enter these already-pruned maps.
    pub(super) fn expire_pruned_availability(
        &mut self,
        budget: ResourceRetentionCompactionBudget,
    ) -> [u32; 3] {
        let mut expired = [0_u32; 3];
        if let Some(limit) = budget.pruned_lifecycle_availability_limit() {
            while self.pruned_in_flight_history_by_request.len() > limit as usize {
                let Some(id) = self
                    .pruned_in_flight_history_by_request
                    .keys()
                    .next()
                    .copied()
                else {
                    break;
                };
                if self.expire_lifecycle_availability(id) {
                    expired[0] = expired[0].saturating_add(1);
                }
            }
        }
        if let Some(limit) = budget.pruned_denied_availability_limit() {
            while self.pruned_denied_completions_by_id.len() > limit as usize {
                let Some(id) = self.pruned_denied_completions_by_id.keys().next().copied() else {
                    break;
                };
                if self.expire_denied_availability(id) {
                    expired[1] = expired[1].saturating_add(1);
                }
            }
        }
        if let Some(limit) = budget.pruned_retry_availability_limit() {
            while self.pruned_retry_lineage_by_ordinal.len() > limit as usize {
                let Some(id) = self.pruned_retry_lineage_by_ordinal.keys().next().copied() else {
                    break;
                };
                if self.expire_retry_availability(id) {
                    expired[2] = expired[2].saturating_add(1);
                }
            }
        }
        expired
    }

    pub(super) fn expire_lifecycle_availability(
        &mut self,
        id: crate::data::resource::ResourceRequestId,
    ) -> bool {
        let Some(value) = self.pruned_in_flight_history_by_request.remove(&id) else {
            return false;
        };
        self.absorb_released_lifecycle(value);
        true
    }

    pub(super) fn expire_denied_availability(
        &mut self,
        id: crate::data::resource::AsyncDenialId,
    ) -> bool {
        let Some(value) = self.pruned_denied_completions_by_id.remove(&id) else {
            return false;
        };
        self.absorb_released_denial(value);
        true
    }

    pub(super) fn expire_retry_availability(
        &mut self,
        id: crate::data::resource::ResourceRetryOrdinal,
    ) -> bool {
        let Some(value) = self.pruned_retry_lineage_by_ordinal.remove(&id) else {
            return false;
        };
        self.absorb_released_retry(value);
        true
    }

    pub(super) fn absorb_released_lifecycle(
        &mut self,
        value: crate::data::resource::ResourceRetainedHistoryAvailability,
    ) {
        self.expired_lifecycle_availability.absorb(
            b"resource-lifecycle",
            value.handle().request_id().get(),
            &value,
        );
    }

    pub(super) fn absorb_released_denial(
        &mut self,
        value: crate::data::resource::ResourceRetainedDeniedCompletionAvailability,
    ) {
        self.expired_denied_availability.absorb(
            b"resource-denied-completion",
            value.denial_id().get(),
            &value,
        );
    }

    pub(super) fn absorb_released_retry(
        &mut self,
        value: crate::data::resource::ResourceRetainedRetryLineageAvailability,
    ) {
        self.expired_retry_availability.absorb(
            b"resource-retry-lineage",
            value.retry_ordinal().get(),
            &value,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::ResourceRuntimeState;

    #[test]
    fn replay_digest_distinguishes_equal_width_expired_resource_histories() {
        let mut left = ResourceRuntimeState::default();
        let mut right = ResourceRuntimeState::default();
        left.expired_lifecycle_availability
            .absorb(b"resource-lifecycle", 7, &"cancelled");
        right
            .expired_lifecycle_availability
            .absorb(b"resource-lifecycle", 7, &"timed-out");
        let left_report = left.reconstruct_replay_summary_optional(None);
        let right_report = right.reconstruct_replay_summary_optional(None);
        assert_eq!(
            left_report.retained_history_unavailable_count(),
            right_report.retained_history_unavailable_count()
        );
        assert_ne!(left_report.replay_digest(), right_report.replay_digest());
    }
}
