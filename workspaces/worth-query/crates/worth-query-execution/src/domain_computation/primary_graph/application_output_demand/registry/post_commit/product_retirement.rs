use super::super::refreshed_rejoin::infallible;
use super::super::refreshed_rejoin::OccurrenceSearchCharge;
use super::walk_charge::PostCommitSweep;
use super::*;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn release_product_occurrence(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut sweep = PostCommitSweep::after_effect();
        for custody in state.source_custody.values_mut() {
            infallible(sweep.inspect_row());
            if custody.occurrence == occurrence {
                custody.retire(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Closed,
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
            infallible(sweep.inspect_row());
            if record.product_occurrence != occurrence
                || (record.interests == 0
                    && record.framework_required_count == 0
                    && record.prepared_prerequisite_claims == 0)
            {
                continue;
            }
            let denial = WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Closed,
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
        infallible(state.release_matching_prerequisites(
            |_, record| record.product_occurrence == occurrence,
            &mut sweep,
        ));
        let mut released_bytes = 0;
        let DemandRegistryState {
            records,
            required_keys,
            required_reserved_bytes,
            ..
        } = &mut *state;
        records.retain(|key, record| {
            infallible(sweep.inspect_row());
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
        state.pending_custody_prune = true;
        drop(state);
        drop(retired_cursors);
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::{key, record};
    use super::*;
    use crate::domain_computation::primary_graph::tests::recoverable_commit_support;
    #[test]
    fn registry_product_retirement_sweep_names_its_registry_bound() {
        let receipts: Vec<_> = (0..5)
            .map(|_| recoverable_commit_support::committed_recoverable_application())
            .collect();
        let mut contacts = Vec::new();
        for (rows, custody_entries) in [(2, 0), (5, 1), (9, 4)] {
            let registry = WorthQueryOutputDemandRegistry::default();
            let mut state = registry.state.lock().unwrap();
            for generation in 1..=rows {
                state.records.insert(
                    key("retired", generation, 1),
                    record(
                        receipts[0].product_branch().occurrence(),
                        DemandState::Admitted,
                        1,
                    ),
                );
            }
            for receipt in receipts.iter().take(custody_entries) {
                state.source_custody.insert(
                    receipt
                        .committed_product_publication()
                        .composite_commit()
                        .clone(),
                    SourceCustody {
                        occurrence: receipt.product_branch().occurrence(),
                        root_kind: PreparedOutputRootKind::Discovered(
                            crate::equality_type_id::EqualityTypeId::of::<()>(),
                        ),
                        source: None,
                        discovery: None,
                        bound_sources: None,
                        consumed_sources: Vec::new(),
                        retired_sources: Vec::new(),
                        retired: None,
                        token_count: 1,
                        completed: false,
                    },
                );
            }
            drop(state);
            let before = super::super::walk_charge::sweep_contacts();
            registry.release_product_occurrence(receipts[0].product_branch().occurrence());
            contacts.push(super::super::walk_charge::sweep_contacts() - before);
            assert!(registry
                .state
                .lock()
                .unwrap()
                .records
                .values()
                .all(|row| row.terminal()));
        }
        // Four complete record passes plus one contact per custody entry.
        assert_eq!(contacts, [8, 21, 40]);
    }
}
