//! Synchronous source-wide closure precedes admission-free custody readers.

use super::super::source_custody::denial;

use super::*;
use crate::domain_computation::primary_graph::application_query::WorthQueryObservedSourceEpoch;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;
use std::cmp::Ordering;
pub(in super::super) fn retire_stale_records(
    state: &mut DemandRegistryState,
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    successor: &WorthQueryObservedSourceEpoch,
) {
    // Custody retirement is source-wide: a changed query source invalidates
    // every producer record that was computed from its older epoch.

    let keys = super::super::refreshed_rejoin::source_rows(&state.records, successor)
        .filter(|(key, record)| {
            record.product_occurrence == occurrence
                && record.source_scope == Some(scope)
                && key.source.replacement_order(successor) == Some(Ordering::Less)
        })
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();
    for key in &keys {
        let mut record = state
            .records
            .get_mut(key)
            .expect("source occurrence key exists");
        crate::domain_computation::primary_graph::fact_verification_observation::registry_custody(
            crate::domain_computation::primary_graph::WorthQueryRegistryCustodyTransitionForTest::SourceRetirement,
            record.settlements.len(), record.prerequisites.len(),
        );
        let cause = denial(
            WorthQueryOutputDemandDenialKind::Superseded,
            key.producer.clone(),
        );
        match &mut record.state {
            DemandState::Output(output) => output.stop(cause),
            _ => record.state = DemandState::Failed(cause),
        }
        record.performed_source = None;
        record.wake.notify();
        record.finish();
        state.release_record_prerequisites(key);
    }
    // Remove only after every stale row has released its prerequisites: an
    // upstream row that sorts before its consumer is held until then.
    for key in keys {
        let record = state
            .records
            .get(&key)
            .expect("source occurrence key exists");
        if record.interests == 0
            && record.framework_required_count == 0
            && record.prepared_prerequisite_claims == 0
            && !record.pending_cleanup_queued
        {
            state.remove_record(&key);
            state.required_keys.take(&key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::key;
    use super::super::super::tests::record;
    use super::*;
    #[test]
    fn registry_source_retirement_sweep_names_its_source_coordinate_bound() {
        let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::
            committed_recoverable_application();
        let scope = receipt.principal_scope().scope();
        let occurrence = receipt.product_branch().occurrence();
        let receipts = super::super::super::tests::walk_bounds::receipt_batch();
        let mut contacts = Vec::new();
        for producers in [1, 3, 7] {
            let mut state = DemandRegistryState::default();
            for producer in 0..producers {
                let mut row = record(occurrence, DemandState::Admitted, 1);
                row.source_scope = Some(scope);
                state
                    .records
                    .insert(key(&format!("source-owner-{producer}"), 1, 1), row);
            }
            super::super::super::tests::walk_bounds::add_custody(&mut state, &receipts, 3);
            let before = super::super::retirement_observation::sweep_contacts();
            retire_stale_records(&mut state, occurrence, scope, &key("unused", 2, 1).source);
            contacts.push(super::super::retirement_observation::sweep_contacts() - before);
            assert!(state
                .records
                .source_occurrence_rows(&key("unused", 2, 1).source)
                .all(|(_, row)| row.terminal()));
            assert_eq!(state.source_custody.len(), receipts.len());
        }
        // One source-coordinate row per producer; no prerequisite edges.
        assert_eq!(contacts, [1, 3, 7]);
    }
}
