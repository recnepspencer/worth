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
        // World already retired this incarnation; this entry carries no completion receipt.
        let mut sweep = PostCommitSweep::after_effect();
        state.note_custody_changes(|_, custody| {
            infallible(sweep.inspect_row());
            if custody.occurrence == occurrence {
                custody.retire(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Closed,
                    "product occurrence retired before required-output custody was consumed",
                ));
                return true;
            }
            false
        });
        if let Some(preparation) = state.source_preparations.get_mut(&occurrence) {
            preparation.retired = true;
        }
        let retired_cursors = state.take_discontinuity_cursors(occurrence);
        // Mark retained rows terminal before draining their descendants. A
        // descendant can release the final framework pin during this walk.
        for mut record in state.records.values_mut() {
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
            if unheld_retired_row(record) {
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
        drop(state);
        drop(retired_cursors);
    }
}

fn unheld_retired_row(record: &DemandRecord) -> bool {
    record.interests == 0
        && record.framework_required_count == 0
        && record.prepared_prerequisite_claims == 0
        && !record.pending_cleanup_queued
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::{key, record, record_admission};
    use super::*;
    use crate::domain_computation::primary_graph::tests::recoverable_commit_support;
    #[test]
    fn registry_product_retirement_prunes_rows_and_their_notes_together() {
        let receipt = recoverable_commit_support::committed_recoverable_application();
        let registry = WorthQueryOutputDemandRegistry::default();
        let held = key("retired-note", 1, 2);
        let wake = {
            let mut state = registry.state.lock().unwrap();
            for slot in 1..=3 {
                let key = key("retired-note", 1, slot);
                state.records.insert(
                    key.clone(),
                    record(
                        receipt.product_branch().occurrence(),
                        DemandState::Failed(WorthQueryOutputDemandDenial::new(
                            WorthQueryOutputDemandDenialKind::NoEffect,
                            "terminal debt",
                        )),
                        usize::from(slot == 2),
                    ),
                );
                let member = state.prepare_required_member(&key).unwrap();
                state.install_required_member(member);
                state.note_closed_row(&key);
            }
            Arc::clone(&state.records[&held].wake)
        };
        let before = super::super::walk_charge::sweep_contacts();
        registry.release_product_occurrence(receipt.product_branch().occurrence());
        // Four three-row passes; the owned notes are untouched by record retain.
        assert_eq!(super::super::walk_charge::sweep_contacts() - before, 12);
        {
            let state = registry.state.lock().unwrap();
            assert_eq!(state.records.len(), 1);
            assert_eq!(state.closed_cleanup.len(), 3);
            assert!(state.closed_cleanup.valid_order());
        }
        drop(WorthQueryOutputDemandInterest {
            key: held,
            requires_output: false,
            notifications: WorthQueryOutputDemandNotifications { wake },
            owner: registry.clone(),
        });
        registry
            .drain_closed_cleanup_admitted(&mut record_admission())
            .unwrap();
        let state = registry.state.lock().unwrap();
        assert!(state.records.is_empty());
        assert_eq!(state.closed_cleanup.len(), 0);
        assert!(state.closed_cleanup.head().is_none());
    }

    #[test]
    fn registry_product_retirement_sweep_names_its_registry_bound() {
        let receipts: Vec<_> = (0..5)
            .map(|_| recoverable_commit_support::committed_recoverable_application())
            .collect();
        let mut contacts = Vec::new();
        for (rows, custody_entries) in [(2, 1), (5, 1), (9, 4)] {
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
                        cleanup: Default::default(),
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
        assert_eq!(contacts, [9, 21, 40]);
    }
}
