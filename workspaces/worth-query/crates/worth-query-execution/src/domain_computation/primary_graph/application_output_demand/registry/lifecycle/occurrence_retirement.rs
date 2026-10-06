use super::*;

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn release_product_occurrence(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for custody in state.source_custody.values_mut() {
            if custody.occurrence == occurrence {
                custody.retire(WorthQueryOutputDemandDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Closed,
                    "product occurrence retired before required-output custody was consumed",
                ));
            }
        }
        if let Some(preparation) = state.source_preparations.get_mut(&occurrence) {
            preparation.retired = true;
        }
        let retired_cursors = state.take_discontinuity_cursors(occurrence);
        // Mark retained rows terminal before draining their descendants. A
        // descendant can release the final framework pin during this walk.
        for record in state.records.values_mut() {
            if record.product_occurrence != occurrence
                || (record.interests == 0
                    && record.framework_required_count == 0
                    && record.prepared_prerequisite_claims == 0)
            {
                continue;
            }
            let denial = WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Closed,
                "product occurrence retired",
            );
            match &mut record.state {
                DemandState::Output(output) => output.stop(denial),
                _ => record.state = DemandState::Failed(denial),
            }
            record.performed_source = None;
            record.readmission_source.take();
            record.wake.notify();
        }
        state.release_matching_prerequisites(|_, record| record.product_occurrence == occurrence);
        let mut released_bytes = 0;
        let DemandRegistryState {
            records,
            required_keys,
            required_reserved_bytes,
            ..
        } = &mut *state;
        records.retain(|key, record| {
            if record.product_occurrence != occurrence {
                return true;
            }
            if record.interests == 0
                && record.framework_required_count == 0
                && record.prepared_prerequisite_claims == 0
                && !record.pending_cleanup_queued
            {
                released_bytes += record.obligation_reserved_bytes();
                if let Some(member) = required_keys.take(key) {
                    *required_reserved_bytes = required_reserved_bytes.saturating_sub(
                        super::required_members::member_bytes(member.as_ref())
                            .expect("admitted key charge fits"),
                    );
                }
                return false;
            }
            released_bytes += record.release_obligations();
            if !record.is_required() {
                if let Some(member) = required_keys.take(key) {
                    *required_reserved_bytes = required_reserved_bytes.saturating_sub(
                        super::required_members::member_bytes(member.as_ref())
                            .expect("admitted key charge fits"),
                    );
                }
            }
            true
        });
        state.obligation_reserved_bytes = state
            .obligation_reserved_bytes
            .saturating_sub(released_bytes);
        state.prune_completed_custody();
        drop(state);
        drop(retired_cursors);
    }
}
